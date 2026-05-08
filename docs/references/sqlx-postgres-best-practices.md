# SQLx and PostgreSQL Best Practices

**Reference Guide for Production-Grade Database Integration**

Last Updated: 2025-11-23

---

## Table of Contents
1. [Connection Management](#connection-management)
2. [Query Patterns](#query-patterns)
3. [Transactions](#transactions)
4. [Error Handling](#error-handling)
5. [Performance Optimization](#performance-optimization)
6. [Security](#security)
7. [Testing](#testing)
8. [Migrations](#migrations)
9. [Production Considerations](#production-considerations)

---

## Connection Management

### Use Connection Pools

**Always use PgPool, never single connections in production:**

```rust
use sqlx::postgres::PgPoolOptions;

// Good: Connection pool with proper configuration
let pool = PgPoolOptions::new()
    .max_connections(5)  // Adjust based on load
    .min_connections(2)  // Keep minimum connections alive
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;

// Bad: Single connection (no concurrency)
let conn = PgConnection::connect(&database_url).await?;
```

### Pool Sizing Guidelines

**Conservative formula:**
```
max_connections = (number_of_cores * 2) + number_of_disk_spindles
```

**For most web services:**
- Development: 5-10 connections
- Production (small): 10-20 connections
- Production (large): 20-50 connections
- **Never exceed PostgreSQL's `max_connections` setting**

**Why limit connections?**
- PostgreSQL uses processes (not threads) - each connection = overhead
- More connections ≠ better performance
- Too many connections = context switching overhead
- Better to queue requests than overwhelm the database

### Connection Lifecycle

```rust
// Always clone the pool, never move it
async fn handler(pool: PgPool) -> Result<(), Error> {
    // PgPool is cheap to clone (Arc internally)
    let cloned_pool = pool.clone();
    
    // Acquire connection from pool
    let mut conn = pool.acquire().await?;
    
    // Use connection
    let result = sqlx::query("SELECT * FROM products")
        .fetch_all(&mut *conn)
        .await?;
    
    // Connection automatically returned to pool on drop
    Ok(())
}
```

---

## Query Patterns

### Compile-Time Checked Queries

**Use `query!` macro for type safety:**

```rust
// Good: Compile-time checked (verifies against actual database)
let products = sqlx::query!(
    r#"
    SELECT id, name, price, inventory_count
    FROM products
    WHERE price > $1
    "#,
    min_price
)
.fetch_all(&pool)
.await?;

// Access fields with type safety
for product in products {
    println!("{}: ${}", product.name, product.price);
}
```

**Benefits:**
- Catches SQL syntax errors at compile time
- Type-checks column names and types
- Auto-generates Rust struct from query
- Prevents runtime SQL errors

### Runtime Queries (When Needed)

```rust
// Use query() for dynamic queries
let table_name = get_table_name(); // Dynamic table selection

let products = sqlx::query_as::<_, Product>(&format!(
    "SELECT * FROM {} WHERE active = $1",
    table_name  // Dynamic, but validate/sanitize!
))
.bind(true)
.fetch_all(&pool)
.await?;
```

**⚠️ Warning:** Only use dynamic queries when absolutely necessary. Always validate/sanitize dynamic inputs.

### Query Builder Pattern

```rust
struct ProductQuery {
    min_price: Option<f64>,
    category: Option<String>,
    in_stock: bool,
}

impl ProductQuery {
    async fn execute(&self, pool: &PgPool) -> Result<Vec<Product>, Error> {
        let mut query = QueryBuilder::new(
            "SELECT id, name, price FROM products WHERE 1=1"
        );
        
        if let Some(price) = self.min_price {
            query.push(" AND price >= ");
            query.push_bind(price);
        }
        
        if let Some(ref cat) = self.category {
            query.push(" AND category = ");
            query.push_bind(cat);
        }
        
        if self.in_stock {
            query.push(" AND inventory_count > 0");
        }
        
        query.build_query_as::<Product>()
            .fetch_all(pool)
            .await
    }
}
```

---

## Transactions

### Basic Transaction Pattern

```rust
async fn transfer_inventory(
    pool: &PgPool,
    from_product_id: Uuid,
    to_product_id: Uuid,
    quantity: i32,
) -> Result<(), Error> {
    // Begin transaction
    let mut tx = pool.begin().await?;
    
    // Deduct from source
    sqlx::query!(
        "UPDATE products SET inventory_count = inventory_count - $1 WHERE id = $2",
        quantity,
        from_product_id
    )
    .execute(&mut *tx)
    .await?;
    
    // Add to destination
    sqlx::query!(
        "UPDATE products SET inventory_count = inventory_count + $1 WHERE id = $2",
        quantity,
        to_product_id
    )
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction (or automatically rollback on error)
    tx.commit().await?;
    
    Ok(())
}
```

### Transaction Best Practices

**DO:**
- Keep transactions short (minimize time between BEGIN and COMMIT)
- Use transactions for multiple related operations
- Always commit or rollback explicitly
- Use appropriate isolation levels

**DON'T:**
- Perform expensive computations inside transactions
- Make external API calls during transactions
- Keep transactions open while waiting for user input
- Nest transactions (use savepoints instead)

### Savepoints (Nested Transactions)

```rust
let mut tx = pool.begin().await?;

// Do some work
update_product(&mut tx, product_id).await?;

// Create savepoint
sqlx::query("SAVEPOINT my_savepoint")
    .execute(&mut *tx)
    .await?;

// Try risky operation
match risky_operation(&mut tx).await {
    Ok(_) => {
        // Success: continue
    }
    Err(e) => {
        // Rollback to savepoint
        sqlx::query("ROLLBACK TO SAVEPOINT my_savepoint")
            .execute(&mut *tx)
            .await?;
    }
}

// Commit entire transaction
tx.commit().await?;
```

---

## Error Handling

### Match Specific Database Errors

```rust
use sqlx::error::DatabaseError;

match sqlx::query!("INSERT INTO products (id, name) VALUES ($1, $2)", id, name)
    .execute(&pool)
    .await
{
    Ok(_) => println!("Inserted successfully"),
    Err(sqlx::Error::Database(db_err)) => {
        // PostgreSQL error codes: https://www.postgresql.org/docs/current/errcodes-appendix.html
        if db_err.code() == Some(Cow::from("23505")) {
            // Unique violation
            return Err(ApiError::Conflict("Product already exists"));
        }
        if db_err.code() == Some(Cow::from("23503")) {
            // Foreign key violation
            return Err(ApiError::BadRequest("Referenced entity doesn't exist"));
        }
        Err(ApiError::DatabaseError(db_err.to_string()))
    }
    Err(sqlx::Error::RowNotFound) => {
        Err(ApiError::NotFound("Product not found"))
    }
    Err(e) => Err(ApiError::Internal(e.to_string())),
}
```

### Common PostgreSQL Error Codes

| Code | Meaning |
|------|---------|
| `23505` | Unique constraint violation |
| `23503` | Foreign key violation |
| `23502` | Not null violation |
| `42P01` | Undefined table |
| `42703` | Undefined column |

---

## Performance Optimization

### Use Prepared Statements

```rust
// SQLx automatically uses prepared statements with query!() and bind()
// This query is prepared once and reused:
let stmt = sqlx::query!(
    "SELECT * FROM products WHERE category = $1"
)
.bind(&category)
.fetch_all(&pool)
.await?;
```

### Batch Operations

```rust
// Good: Batch insert (single query)
let mut query_builder = QueryBuilder::new(
    "INSERT INTO products (name, price)"
);

query_builder.push_values(products.iter(), |mut b, product| {
    b.push_bind(&product.name)
     .push_bind(product.price);
});

query_builder.build().execute(&pool).await?;

// Bad: Loop with individual inserts (N queries)
for product in products {
    sqlx::query!("INSERT INTO products (name, price) VALUES ($1, $2)", 
        product.name, product.price)
        .execute(&pool)
        .await?;
}
```

### Use Indexes Wisely

```sql
-- Index frequently queried columns
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_price ON products(price);

-- Composite index for common query patterns
CREATE INDEX idx_products_category_price ON products(category, price);

-- Partial index for common filter
CREATE INDEX idx_products_in_stock ON products(inventory_count) 
WHERE inventory_count > 0;

-- Check index usage
EXPLAIN ANALYZE SELECT * FROM products WHERE category = 'electronics';
```

### Pagination Best Practices

```rust
// Good: Cursor-based pagination (keyset pagination)
async fn get_products_after_cursor(
    pool: &PgPool,
    cursor_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Product>, Error> {
    if let Some(cursor) = cursor_id {
        sqlx::query_as!(
            Product,
            r#"
            SELECT id, name, price, inventory_count, created_at, updated_at
            FROM products
            WHERE id > $1
            ORDER BY id
            LIMIT $2
            "#,
            cursor,
            limit
        )
        .fetch_all(pool)
        .await
    } else {
        // First page
        sqlx::query_as!(
            Product,
            "SELECT * FROM products ORDER BY id LIMIT $1",
            limit
        )
        .fetch_all(pool)
        .await
    }
}

// Avoid: OFFSET-based pagination for large tables (slow!)
// SELECT * FROM products OFFSET 10000 LIMIT 100  // Scans 10,000 rows to skip them
```

### Connection Timing

```rust
// Acquire connection as late as possible
async fn handler(pool: PgPool) -> Result<Response, Error> {
    // Do non-DB work first
    let validated_data = validate_input()?;
    let transformed = transform_data(validated_data)?;
    
    // Only acquire connection when needed
    let result = sqlx::query!("INSERT INTO products ...")
        .execute(&pool)  // Connection acquired here
        .await?;
    
    // Connection returned immediately after query
    
    Ok(Response::new(result))
}
```

---

## Security

### SQL Injection Prevention

```rust
// Good: Parameterized queries (SQLx prevents SQL injection)
let user_input = "'; DROP TABLE products; --";
let products = sqlx::query!(
    "SELECT * FROM products WHERE name = $1",
    user_input  // Safe: treated as literal string
)
.fetch_all(&pool)
.await?;

// Bad: String concatenation (NEVER DO THIS)
let query = format!("SELECT * FROM products WHERE name = '{}'", user_input);
// This would execute: SELECT * FROM products WHERE name = ''; DROP TABLE products; --'
```

### Use Least Privilege

```sql
-- Create role with minimal permissions
CREATE ROLE app_user WITH LOGIN PASSWORD 'secure_password';

-- Grant only what's needed
GRANT SELECT, INSERT, UPDATE, DELETE ON products TO app_user;

-- Don't grant
-- GRANT ALL PRIVILEGES  -- Too permissive
-- GRANT SUPERUSER       -- Never for app
```

### Secrets Management

```rust
// Good: Load from environment or secrets manager
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

// Bad: Hardcoded credentials
let database_url = "postgresql://user:password@localhost/db";  // NEVER!
```

---

## Testing

### Use Test Transactions

```rust
#[sqlx::test]
async fn test_create_product(pool: PgPool) -> sqlx::Result<()> {
    // Automatically runs in transaction and rolls back after test
    let product = create_product(&pool, "Test Product", 99.99).await?;
    assert_eq!(product.name, "Test Product");
    Ok(())
    // Rollback happens automatically
}
```

### Test Database Setup

```rust
// Use separate test database
#[cfg(test)]
async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or("postgresql://test_user:test_pass@localhost/rustmart_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(1)  // Test isolation
        .connect(&database_url)
        .await
        .expect("Failed to create test pool")
}
```

---

## Migrations

### Migration Best Practices

**DO:**
- Use descriptive migration names: `20231122_add_product_category_index.sql`
- Write both UP and DOWN migrations
- Test migrations on staging before production
- Keep migrations small and focused
- Version control all migrations

**DON'T:**
- Edit existing migrations (create new ones instead)
- Perform data migrations and schema changes in same migration
- Add NOT NULL without default value (breaks existing data)

### Safe Migration Patterns

```sql
-- Good: Add column with default
ALTER TABLE products 
ADD COLUMN category VARCHAR(100) DEFAULT 'uncategorized';

-- Then remove default in next migration (optional)
ALTER TABLE products 
ALTER COLUMN category DROP DEFAULT;

-- Bad: Add NOT NULL without default (fails if data exists)
ALTER TABLE products 
ADD COLUMN category VARCHAR(100) NOT NULL;
```

### Zero-Downtime Migrations

```sql
-- Step 1: Add new column (nullable)
ALTER TABLE products ADD COLUMN new_price DECIMAL(10,2);

-- Step 2: Backfill data (in batches, not in migration)
-- Do this in application code with LIMIT/OFFSET

-- Step 3: Add NOT NULL constraint (after backfill complete)
ALTER TABLE products ALTER COLUMN new_price SET NOT NULL;

-- Step 4: Drop old column (in next migration)
ALTER TABLE products DROP COLUMN old_price;
```

---

## Production Considerations

### Health Checks

```rust
pub async fn database_health_check(pool: &PgPool) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    Ok(())
}
```

### Graceful Shutdown

```rust
pub async fn shutdown_database(pool: PgPool) {
    pool.close().await;
    tracing::info!("Database connections closed gracefully");
}
```

### Monitoring Queries

```rust
use tracing::instrument;

#[instrument(skip(pool))]
async fn get_product(pool: &PgPool, id: Uuid) -> Result<Product, Error> {
    let start = Instant::now();
    
    let product = sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id)
        .fetch_one(pool)
        .await?;
    
    let duration = start.elapsed();
    if duration > Duration::from_millis(100) {
        tracing::warn!("Slow query detected: {:?}", duration);
    }
    
    Ok(product)
}
```

### Connection Pool Monitoring

```rust
pub fn log_pool_metrics(pool: &PgPool) {
    tracing::info!(
        "Pool metrics - Size: {}, Idle: {}, Active: {}",
        pool.size(),
        pool.num_idle(),
        pool.size() - pool.num_idle()
    );
}
```

---

## Additional Resources

**Official Documentation:**
- [SQLx Book](https://github.com/launchbadge/sqlx)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)

**Performance:**
- [Use The Index, Luke](https://use-the-index-luke.com/) - SQL indexing guide
- [Postgres Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)

**Security:**
- [OWASP SQL Injection Prevention](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [PostgreSQL Security Best Practices](https://www.postgresql.org/docs/current/security.html)

---

**Remember:** These are guidelines, not absolute rules. Understand the tradeoffs and adjust based on your specific requirements.
