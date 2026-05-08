# Lesson 21: Advanced Database Patterns

## Overview
Implement advanced database patterns including read replicas, CQRS, event sourcing, sharding, and materialized views for scalable data architecture in RustMart.

## Why This Matters
As RustMart scales, simple database patterns won't suffice. You'll need:
- **Read replicas** to handle increased query load
- **CQRS** to optimize read and write models independently
- **Event sourcing** for auditability and temporal queries
- **Sharding** for horizontal scalability
- **Materialized views** for fast reporting

## Read Replicas & Connection Pooling

### Setting Up PostgreSQL Replication

**Primary Server Configuration** (`postgresql.conf`):
```conf
wal_level = replica
max_wal_senders = 3
max_replication_slots = 3
```

**Create Replication User**:
```sql
CREATE ROLE replicator WITH REPLICATION PASSWORD 'secure_password' LOGIN;
```

**Replica Server Configuration**:
```conf
primary_conninfo = 'host=primary-db port=5432 user=replicator password=secure_password'
primary_slot_name = 'replica_1'
```

### Read/Write Splitting in Rust

```rust
use sqlx::postgres::{PgPoolOptions, PgPool};
use std::sync::Arc;

pub struct DatabasePools {
    write_pool: PgPool,
    read_pools: Vec<PgPool>,
    current_read: Arc<AtomicUsize>,
}

impl DatabasePools {
    pub async fn new(
        write_url: &str,
        read_urls: Vec<&str>,
    ) -> Result<Self, sqlx::Error> {
        let write_pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(write_url)
            .await?;
        
        let mut read_pools = Vec::new();
        for url in read_urls {
            let pool = PgPoolOptions::new()
                .max_connections(50)
                .connect(url)
                .await?;
            read_pools.push(pool);
        }
        
        Ok(Self {
            write_pool,
            read_pools,
            current_read: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    pub fn write(&self) -> &PgPool {
        &self.write_pool
    }
    
    // Round-robin load balancing
    pub fn read(&self) -> &PgPool {
        let idx = self.current_read.fetch_add(1, Ordering::Relaxed);
        &self.read_pools[idx % self.read_pools.len()]
    }
}
```

### Using Separate Pools

```rust
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    db: Arc<DatabasePools>,
}

async fn list_products(
    State(state): State<AppState>,
) -> Result<Json<Vec<Product>>, ApiError> {
    // Use read replica for queries
    let products = sqlx::query_as::<_, Product>(
        "SELECT id, name, price FROM products"
    )
    .fetch_all(state.db.read())
    .await?;
    
    Ok(Json(products))
}

async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, ApiError> {
    // Use primary for writes
    let product = sqlx::query_as::<_, Product>(
        "INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *"
    )
    .bind(&req.name)
    .bind(&req.price)
    .fetch_one(state.db.write())
    .await?;
    
    Ok(Json(product))
}
```

### Handling Replication Lag

```rust
use chrono::{DateTime, Utc};

// Check replication lag
async fn check_replication_lag(pool: &PgPool) -> Result<Duration, sqlx::Error> {
    let lag: Option<f64> = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(Duration::from_secs_f64(lag.unwrap_or(0.0)))
}

// Read-after-write consistency
async fn create_and_read_product(
    db: &DatabasePools,
    req: CreateProductRequest,
) -> Result<Product, Error> {
    // Write to primary
    let product = sqlx::query_as::<_, Product>(
        "INSERT INTO products (name, price) VALUES ($1, $2) RETURNING *"
    )
    .bind(&req.name)
    .bind(&req.price)
    .fetch_one(db.write())
    .await?;
    
    // Force read from primary to avoid stale data
    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1"
    )
    .bind(&product.id)
    .fetch_one(db.write())
    .await?;
    
    Ok(product)
}
```

## CQRS (Command Query Responsibility Segregation)

### Command Side - Write Model

```rust
use async_trait::async_trait;
use uuid::Uuid;

// Commands
pub struct CreateOrderCommand {
    pub customer_id: Uuid,
    pub items: Vec<OrderItem>,
}

pub struct CancelOrderCommand {
    pub order_id: Uuid,
}

// Command handler trait
#[async_trait]
trait CommandHandler<C> {
    type Result;
    async fn handle(&self, command: C) -> Result<Self::Result, Error>;
}

// Command handler implementation
struct OrderCommandHandler {
    db: PgPool,
    event_bus: Arc<EventBus>,
}

#[async_trait]
impl CommandHandler<CreateOrderCommand> for OrderCommandHandler {
    type Result = Uuid;
    
    async fn handle(&self, cmd: CreateOrderCommand) -> Result<Uuid, Error> {
        let mut tx = self.db.begin().await?;
        
        // Validate business rules
        self.validate_customer(&cmd.customer_id, &mut tx).await?;
        self.validate_inventory(&cmd.items, &mut tx).await?;
        
        // Create order
        let order_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO orders (id, customer_id, status) VALUES ($1, $2, 'pending')"
        )
        .bind(&order_id)
        .bind(&cmd.customer_id)
        .execute(&mut *tx)
        .await?;
        
        // Insert order items
        for item in &cmd.items {
            sqlx::query(
                "INSERT INTO order_items (order_id, product_id, quantity, price) 
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(&order_id)
            .bind(&item.product_id)
            .bind(&item.quantity)
            .bind(&item.price)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        
        // Publish domain event
        self.event_bus.publish(OrderCreatedEvent {
            order_id,
            customer_id: cmd.customer_id,
            items: cmd.items,
        }).await?;
        
        Ok(order_id)
    }
}
```

### Query Side - Read Model

```rust
// Denormalized read model
#[derive(Serialize, Deserialize)]
pub struct OrderView {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub status: String,
    pub total: Decimal,
    pub items: Vec<OrderItemView>,
    pub created_at: DateTime<Utc>,
}

// Query
pub struct OrderListQuery {
    pub customer_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: i32,
    pub per_page: i32,
}

// Query handler
struct OrderQueryHandler {
    read_db: PgPool,
}

impl OrderQueryHandler {
    async fn handle(&self, query: OrderListQuery) -> Result<Vec<OrderView>, Error> {
        let orders = sqlx::query_as::<_, OrderView>(
            r#"
            SELECT 
                o.id,
                o.customer_id,
                c.name as customer_name,
                o.status,
                o.total,
                o.created_at,
                json_agg(oi.*) as items
            FROM orders_view o
            JOIN customers c ON o.customer_id = c.id
            LEFT JOIN order_items_view oi ON o.id = oi.order_id
            WHERE ($1::uuid IS NULL OR o.customer_id = $1)
              AND ($2::text IS NULL OR o.status = $2)
            GROUP BY o.id, c.name
            ORDER BY o.created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(query.customer_id)
        .bind(query.status)
        .bind(query.per_page)
        .bind(query.page * query.per_page)
        .fetch_all(&self.read_db)
        .await?;
        
        Ok(orders)
    }
}
```

### Event-Driven Synchronization

```rust
// Event handler to update read model
struct OrderViewUpdater {
    read_db: PgPool,
}

impl OrderViewUpdater {
    async fn on_order_created(&self, event: OrderCreatedEvent) -> Result<(), Error> {
        // Update denormalized read model
        sqlx::query(
            "INSERT INTO orders_view (id, customer_id, status, total, created_at)
             SELECT $1, $2, 'pending', SUM(quantity * price), now()
             FROM unnest($3::uuid[], $4::int[], $5::numeric[]) 
             AS items(product_id, quantity, price)"
        )
        .bind(&event.order_id)
        .bind(&event.customer_id)
        .bind(&event.items.iter().map(|i| i.product_id).collect::<Vec<_>>())
        .bind(&event.items.iter().map(|i| i.quantity).collect::<Vec<_>>())
        .bind(&event.items.iter().map(|i| i.price).collect::<Vec<_>>())
        .execute(&self.read_db)
        .await?;
        
        Ok(())
    }
}
```

## Event Sourcing

Event sourcing stores all state changes as an append-only sequence of events. The current state is derived by replaying events. This enables perfect audit, temporal queries, and easy debugging.

### Event Store Schema (PostgreSQL)

```sql
CREATE TABLE IF NOT EXISTS order_events (
    event_id      UUID PRIMARY KEY,
    aggregate_id  UUID NOT NULL,             -- e.g., order_id
    aggregate_type TEXT NOT NULL,            -- 'order'
    version       BIGINT NOT NULL,           -- event sequence per aggregate
    event_type    TEXT NOT NULL,             -- e.g., 'OrderCreated'
    payload       JSONB NOT NULL,            -- event data
    metadata      JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (aggregate_id, version)
);

CREATE INDEX IF NOT EXISTS idx_order_events_agg ON order_events(aggregate_id, version);
CREATE INDEX IF NOT EXISTS idx_order_events_type ON order_events(event_type);
```

### Appending Events (Idempotent)

```rust
#[derive(Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated { order_id: Uuid, items: Vec<OrderItem> },
    PaymentCaptured { order_id: Uuid, amount: Decimal },
    Shipped { order_id: Uuid, tracking: String },
}

async fn append_event(
    pool: &PgPool,
    aggregate_id: Uuid,
    aggregate_type: &str,
    expected_version: i64,
    event: &OrderEvent,
    metadata: serde_json::Value,
) -> Result<i64, Error> {
    let mut tx = pool.begin().await?;

    // Next version
    let next_version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM order_events WHERE aggregate_id = $1"
    )
    .bind(aggregate_id)
    .fetch_one(&mut *tx)
    .await?;

    if next_version != expected_version + 1 {
        return Err(Error::ConcurrencyViolation);
    }

    let event_type = match event {
        OrderEvent::OrderCreated { .. } => "OrderCreated",
        OrderEvent::PaymentCaptured { .. } => "PaymentCaptured",
        OrderEvent::Shipped { .. } => "Shipped",
    };

    sqlx::query(
        "INSERT INTO order_events (event_id, aggregate_id, aggregate_type, version, event_type, payload, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(Uuid::new_v4())
    .bind(aggregate_id)
    .bind(aggregate_type)
    .bind(next_version)
    .bind(event_type)
    .bind(serde_json::to_value(event)?)
    .bind(metadata)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(next_version)
}
```

### Rebuilding State and Snapshots

```rust
#[derive(Default)]
struct OrderAggregate { /* fields... */ version: i64 }

impl OrderAggregate {
    fn apply(&mut self, evt: &OrderEvent) {
        match evt {
            OrderEvent::OrderCreated { .. } => { /* initialize */ }
            OrderEvent::PaymentCaptured { .. } => { /* update */ }
            OrderEvent::Shipped { .. } => { /* update */ }
        }
        self.version += 1;
    }
}

async fn load_order(pool: &PgPool, order_id: Uuid) -> Result<OrderAggregate, Error> {
    // Optional: load snapshot first, then replay newer events
    let mut agg = OrderAggregate::default();
    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT event_type, payload FROM order_events WHERE aggregate_id = $1 ORDER BY version"
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?;

    for (etype, payload) in events {
        let evt: OrderEvent = serde_json::from_value(payload)?;
        agg.apply(&evt);
    }
    Ok(agg)
}
```

### Projections (Read Models)

```rust
// Consume events and update denormalized read models
async fn project_order_created(pool: &PgPool, e: &OrderCreatedEvent) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO orders_view (id, customer_id, status, total, created_at)
         VALUES ($1, $2, 'pending', $3, now())"
    )
    .bind(e.order_id)
    .bind(e.customer_id)
    .bind(e.items.iter().map(|i| i.price * Decimal::from(i.quantity)).sum::<Decimal>())
    .execute(pool)
    .await?;
    Ok(())
}
```

## Database Sharding

Sharding spreads data across multiple databases to scale horizontally.

### Shard Key Selection
- Prefer immutable, uniformly distributed keys (e.g., `tenant_id`, `customer_id`)
- Preserve locality for common queries
- Avoid cross-shard transactions when possible

### Routing Layer in Application

```rust
struct ShardRouter {
    shards: HashMap<u16, PgPool>, // shard_id -> pool
}

impl ShardRouter {
    fn shard_for_tenant(&self, tenant_id: Uuid) -> &PgPool {
        let shard_id = (xxhash_rust::xxh3::xxh3_64(tenant_id.as_bytes()) % self.shards.len() as u64) as u16;
        self.shards.get(&shard_id).unwrap()
    }
}

async fn get_customer_orders(router: &ShardRouter, tenant_id: Uuid, customer_id: Uuid) -> Result<Vec<Order>, Error> {
    let pool = router.shard_for_tenant(tenant_id);
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE tenant_id = $1 AND customer_id = $2"
    )
    .bind(tenant_id)
    .bind(customer_id)
    .fetch_all(pool)
    .await?;
    Ok(orders)
}
```

### PostgreSQL Options
- Native partitioning + separate schemas per shard
- Citus (distributed PostgreSQL) for automatic sharding and parallel queries
- Foreign Data Wrappers (FDW) for limited cross-shard reads

### Resharding Strategy
- Dual-write to old and new shard during migration
- Backfill data in batches with idempotent jobs
- Cutover when lag is zero, then disable old writes

## Materialized Views

### Creating Materialized Views

```sql
CREATE MATERIALIZED VIEW order_summary AS
SELECT 
    o.customer_id,
    c.name as customer_name,
    COUNT(*) as total_orders,
    SUM(o.total) as total_spent,
    MAX(o.created_at) as last_order_date
FROM orders o
JOIN customers c ON o.customer_id = c.id
GROUP BY o.customer_id, c.name;

CREATE INDEX ON order_summary (customer_id);
```

### Refresh Strategies

```rust
// Full refresh
async fn refresh_materialized_view(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("REFRESH MATERIALIZED VIEW order_summary")
        .execute(pool)
        .await?;
    Ok(())
}

// Concurrent refresh (doesn't block reads)
async fn refresh_concurrent(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY order_summary")
        .execute(pool)
        .await?;
    Ok(())
}

// Scheduled refresh with Tokio
use tokio::time::{interval, Duration};

async fn schedule_refresh(pool: PgPool) {
    let mut ticker = interval(Duration::from_secs(300)); // 5 minutes
    
    loop {
        ticker.tick().await;
        if let Err(e) = refresh_concurrent(&pool).await {
            error!("Failed to refresh materialized view: {}", e);
        }
    }
}
```

## Zero-Downtime Migrations

### Strategy: Expand-Contract
1. Expand: Add new schema elements (nullable columns, new tables)
2. Backfill: Migrate data gradually with idempotent jobs
3. Switch: Update code to write to the new schema
4. Contract: Remove old schema elements when unused

```sql
-- Migration 1: Add new column as nullable
ALTER TABLE products ADD COLUMN IF NOT EXISTS description TEXT;

-- Backfill in batches (via job)
-- UPDATE products SET description = 'Default' WHERE description IS NULL LIMIT 1000;

-- Migration 2: Enforce NOT NULL only when backfill done
ALTER TABLE products ALTER COLUMN description SET NOT NULL;
```

### SQLx Migrations in Rust

```rust
use sqlx::migrate::Migrator;
static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    MIGRATOR.run(pool).await
}
```

## Optimistic vs Pessimistic Locking

### Optimistic Locking with Version Column

```rust
#[derive(sqlx::FromRow)]
struct Product {
    id: Uuid,
    name: String,
    price: Decimal,
    inventory_count: i32,
    version: i32,
}

async fn update_product_optimistic(
    pool: &PgPool,
    id: Uuid,
    new_price: Decimal,
    current_version: i32,
) -> Result<Product, Error> {
    let result = sqlx::query_as::<_, Product>(
        "UPDATE products 
         SET price = $1, version = version + 1
         WHERE id = $2 AND version = $3
         RETURNING *"
    )
    .bind(new_price)
    .bind(id)
    .bind(current_version)
    .fetch_optional(pool)
    .await?;
    
    match result {
        Some(product) => Ok(product),
        None => Err(Error::OptimisticLockError),
    }
}
```

### Pessimistic Locking with SELECT FOR UPDATE

```rust
async fn update_product_pessimistic(
    pool: &PgPool,
    id: Uuid,
    new_price: Decimal,
) -> Result<Product, Error> {
    let mut tx = pool.begin().await?;

    let _product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 FOR UPDATE"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, Product>(
        "UPDATE products SET price = $1 WHERE id = $2 RETURNING *"
    )
    .bind(new_price)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}
```

## Database Performance Optimization

### Query Plan Analysis

```rust
async fn explain_query(pool: &PgPool, query: &str) -> Result<Vec<String>, sqlx::Error> {
    let explained = sqlx::query_scalar::<_, String>(&format!("EXPLAIN ANALYZE {}", query))
        .fetch_all(pool)
        .await?;
    Ok(explained)
}
```

### Index Strategies

```sql
-- B-tree index (default)
CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);

-- Partial index
CREATE INDEX IF NOT EXISTS idx_active_products ON products(created_at) 
WHERE deleted_at IS NULL;

-- Covering index
CREATE INDEX IF NOT EXISTS idx_products_covering ON products(name) 
INCLUDE (price, inventory_count);

-- GIN index (JSON/arrays/search)
CREATE INDEX IF NOT EXISTS idx_products_tags ON products USING GIN(tags);

-- BRIN index (large sequential data)
CREATE INDEX IF NOT EXISTS idx_orders_date ON orders USING BRIN(created_at);
```

### Preventing N+1 Queries

```rust
// Batch queries or use JOIN + JSON aggregation
let rows = sqlx::query(
    r#"
    SELECT 
        o.id as order_id,
        o.customer_id,
        o.total,
        json_agg(oi.*) as items
    FROM orders o
    LEFT JOIN order_items oi ON o.id = oi.order_id
    GROUP BY o.id
    "#,
)
.fetch_all(pool)
.await?;
```

## Data Archiving & Retention

### Native Partitioning

```sql
-- Partition orders by month
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL,
    -- ...
) PARTITION BY RANGE (created_at);

CREATE TABLE IF NOT EXISTS orders_2025_01 PARTITION OF orders
FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### Retention Jobs
- Drop or move old partitions to cheaper storage
- Keep hot data small for faster queries

## Database Monitoring

### pg_stat_statements

```sql
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
SELECT query, calls, total_time FROM pg_stat_statements ORDER BY total_time DESC LIMIT 20;
```

### Useful Checks
- Replication lag: `SELECT now() - pg_last_xact_replay_timestamp();`
- Connection utilization per pool
- Slow query log analysis

## Tools & Libraries

- **SQLx**: Compile-time checked queries, migrations
- **Diesel**: ORM with type-safe query builder
- **SeaORM**: Async ORM with relations
- **pgx**: PostgreSQL extensions in Rust
- **PostgreSQL**: Primary database
- **TimescaleDB**: Time-series extension
- **Citus**: Distributed PostgreSQL for sharding

## Hands-on Exercises

1. Implement read replica routing
2. Build CQRS pattern for order management
3. Create event-sourced product catalog
4. Design sharding strategy for multi-tenant system
5. Optimize slow queries with indexes and materialized views
6. Implement zero-downtime migration

## Best Practices

- Monitor replication lag and handle stale reads
- Use transactions judiciously (minimize lock duration)
- Design for idempotency in event handlers
- Test migrations on production-like data
- Document shard key selection rationale
- Regularly analyze and vacuum databases

## Resources

- [PostgreSQL High Availability](https://www.postgresql.org/docs/current/high-availability.html)
- [CQRS Pattern](https://martinfowler.com/bliki/CQRS.html)
- [Event Sourcing by Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Database Sharding](https://www.digitalocean.com/community/tutorials/understanding-database-sharding)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
