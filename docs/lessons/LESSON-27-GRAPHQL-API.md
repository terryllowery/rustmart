# Lesson 27: GraphQL API Layer

## Overview
Build a flexible GraphQL API for RustMart using async-graphql, providing clients with precise data fetching, schema stitching across microservices, and optimized query resolution.

## Why This Matters
GraphQL provides:
- **Precise Data Fetching** - Clients request exactly what they need, no over/under-fetching
- **Single Roundtrip** - Fetch related data in one query (eliminates N+1 problem)
- **Type Safety** - Schema-first development with strong types
- **Self-Documenting** - Introspection provides automatic documentation
- **API Evolution** - Add fields without breaking existing clients

Used by: GitHub, Shopify, Netflix, Airbnb, PayPal.

## GraphQL vs REST

**REST Challenges**:
- Multiple endpoints for related data → multiple HTTP requests
- Over-fetching (getting unused fields) or under-fetching (needing additional requests)
- API versioning complexity

**GraphQL Advantages**:
- Single endpoint `/graphql`
- Client specifies exact data requirements
- Strong typing with schema
- Real-time with subscriptions

## Setting Up async-graphql

**Cargo.toml**:
```toml
[dependencies]
async-graphql = "7.0"
async-graphql-axum = "7.0"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
serde = { version = "1.0", features = ["derive"] }
```

## Complete Schema Implementation

### Query Root

```rust
use async_graphql::{Context, Object, Schema, SimpleObject, ID, InputObject};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(SimpleObject, Clone)]
#[graphql(name = "Product")]
pub struct Product {
    pub id: ID,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub inventory_count: i32,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a single product by ID
    async fn product(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Product> {
        let pool = ctx.data::<PgPool>()?;
        let uuid = Uuid::parse_str(id.as_str())?;
        
        let product = sqlx::query_as!(
            ProductRow,
            "SELECT id, name, description, price, inventory_count FROM products WHERE id = $1",
            uuid
        )
        .fetch_one(pool)
        .await?;
        
        Ok(Product {
            id: ID::from(product.id.to_string()),
            name: product.name,
            description: product.description,
            price: product.price,
            inventory_count: product.inventory_count,
        })
    }
    
    /// List products with pagination
    async fn products(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Number of items to return", default = 20)] limit: i32,
        #[graphql(desc = "Number of items to skip", default = 0)] offset: i32,
    ) -> async_graphql::Result<Vec<Product>> {
        let pool = ctx.data::<PgPool>()?;
        let limit = limit.min(100); // Max 100 items
        
        let products = sqlx::query_as!(
            ProductRow,
            "SELECT id, name, description, price, inventory_count 
             FROM products 
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
            limit as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;
        
        Ok(products
            .into_iter()
            .map(|p| Product {
                id: ID::from(p.id.to_string()),
                name: p.name,
                description: p.description,
                price: p.price,
                inventory_count: p.inventory_count,
            })
            .collect())
    }
    
    /// Search products by name
    async fn search_products(
        &self,
        ctx: &Context<'_>,
        query: String,
    ) -> async_graphql::Result<Vec<Product>> {
        let pool = ctx.data::<PgPool>()?;
        let search_pattern = format!("%{}%", query);
        
        let products = sqlx::query_as!(
            ProductRow,
            "SELECT id, name, description, price, inventory_count 
             FROM products 
             WHERE name ILIKE $1 OR description ILIKE $1
             LIMIT 50",
            search_pattern
        )
        .fetch_all(pool)
        .await?;
        
        Ok(products.into_iter().map(|p| Product::from(p)).collect())
    }
}
```

### Mutations

```rust
#[derive(InputObject)]
pub struct CreateProductInput {
    pub name: String,
    pub description: String,
    #[graphql(validator(minimum = 0.01))]
    pub price: f64,
    #[graphql(validator(minimum = 0))]
    pub inventory_count: i32,
}

#[derive(InputObject)]
pub struct UpdateProductInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub inventory_count: Option<i32>,
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new product
    async fn create_product(
        &self,
        ctx: &Context<'_>,
        input: CreateProductInput,
    ) -> async_graphql::Result<Product> {
        let pool = ctx.data::<PgPool>()?;
        let id = Uuid::new_v4();
        
        let product = sqlx::query_as!(
            ProductRow,
            "INSERT INTO products (id, name, description, price, inventory_count, created_at) 
             VALUES ($1, $2, $3, $4, $5, now()) 
             RETURNING id, name, description, price, inventory_count",
            id,
            input.name,
            input.description,
            input.price,
            input.inventory_count
        )
        .fetch_one(pool)
        .await?;
        
        Ok(Product::from(product))
    }
    
    /// Update an existing product
    async fn update_product(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateProductInput,
    ) -> async_graphql::Result<Product> {
        let pool = ctx.data::<PgPool>()?;
        let uuid = Uuid::parse_str(id.as_str())?;
        
        // Build dynamic update query
        let product = sqlx::query_as!(
            ProductRow,
            "UPDATE products 
             SET name = COALESCE($2, name),
                 description = COALESCE($3, description),
                 price = COALESCE($4, price),
                 inventory_count = COALESCE($5, inventory_count)
             WHERE id = $1
             RETURNING id, name, description, price, inventory_count",
            uuid,
            input.name,
            input.description,
            input.price,
            input.inventory_count
        )
        .fetch_one(pool)
        .await?;
        
        Ok(Product::from(product))
    }
    
    /// Delete a product
    async fn delete_product(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let pool = ctx.data::<PgPool>()?;
        let uuid = Uuid::parse_str(id.as_str())?;
        
        let result = sqlx::query!("DELETE FROM products WHERE id = $1", uuid)
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
```

### Subscriptions (Real-time)

```rust
use async_graphql::Subscription;
use tokio_stream::{Stream, StreamExt};
use futures_util::stream;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to order updates for a specific customer
    async fn order_updates(
        &self,
        customer_id: ID,
    ) -> impl Stream<Item = Order> {
        // In production, this would connect to Redis pub/sub or Kafka
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        
        tokio::spawn(async move {
            // Simulate order updates
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                // In real app: listen to message queue
                // tx.send(order).await.ok();
            }
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
```

### 3. DataLoader Pattern

#### N+1 Query Prevention
```rust
use async_graphql::dataloader::DataLoader;

struct ProductLoader {
    pool: PgPool,
}

impl Loader<ID> for ProductLoader {
    type Value = Product;
    type Error = Error;
    
    async fn load(&self, keys: &[ID]) -> Result<HashMap<ID, Product>> {
        // Batch load products by IDs
    }
}
```

#### Batching & Caching
- Automatic request batching
- Per-request caching
- Cache invalidation strategies

### 4. Schema Stitching & Federation

#### Microservice Schema Composition
- Product service schema
- Order service schema
- Inventory service schema
- Unified gateway schema

#### Apollo Federation
```rust
#[derive(SimpleObject)]
#[graphql(extends)]
struct Product {
    #[graphql(external)]
    id: ID,
    
    #[graphql(provides = "inventory_count")]
    availability: Availability,
}
```

#### Cross-Service Resolvers
- Reference resolvers
- Entity resolution
- Service-to-service queries

### 5. Authentication & Authorization

#### Context Injection
```rust
struct Context {
    user: Option<User>,
    db_pool: PgPool,
    loader: DataLoader<ProductLoader>,
}

#[Object]
impl QueryRoot {
    async fn my_orders(&self, ctx: &Context) -> Result<Vec<Order>> {
        let user = ctx.user.as_ref().ok_or("Unauthorized")?;
        // Fetch user's orders
    }
}
```

#### Field-Level Authorization
- Guard traits
- Role-based access
- Custom directives
- Permission checking

### 6. Performance Optimization

#### Query Complexity Analysis
- Limit query depth
- Calculate query cost
- Reject expensive queries
- Rate limiting by complexity

#### Persistent Queries
- Pre-registered queries
- Query whitelisting
- Reduced payload size

#### APQ (Automatic Persisted Queries)
- Client sends query hash
- Server caches full query
- Network efficiency

### 7. Error Handling

#### GraphQL Error Format
```rust
#[derive(Debug)]
enum ApiError {
    NotFound,
    Unauthorized,
    ValidationError(String),
}

impl From<ApiError> for async_graphql::Error {
    fn from(err: ApiError) -> Self {
        // Convert to GraphQL error
    }
}
```

#### Error Extensions
- Error codes
- Additional context
- Stack traces (dev only)

### 8. Testing GraphQL APIs

#### Unit Tests
- Resolver testing
- Loader testing
- Schema validation

#### Integration Tests
```rust
#[tokio::test]
async fn test_product_query() {
    let schema = create_test_schema().await;
    
    let query = r#"
        query {
            product(id: "1") {
                name
                price
            }
        }
    "#;
    
    let result = schema.execute(query).await;
    assert!(result.errors.is_empty());
}
```

#### Snapshot Testing
- Schema snapshots
- Query result snapshots

### 9. GraphQL Tooling

#### GraphQL Playground
- Interactive query IDE
- Schema exploration
- Documentation generation

#### GraphQL Code Generation
- Generate TypeScript types from schema
- Client code generation
- Type-safe queries

#### Schema Linting
- Breaking change detection
- Best practice enforcement
- Schema versioning

### 10. Subscriptions & Real-time

#### WebSocket Setup
```rust
use async_graphql::http::WebSocketProtocols;

async fn graphql_subscription(
    schema: Schema<QueryRoot, MutationRoot, SubscriptionRoot>,
    protocol: WebSocketProtocols,
) -> impl IntoResponse {
    // WebSocket handler
}
```

#### Subscription Resolvers
```rust
struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn order_updates(&self, user_id: ID) -> impl Stream<Item = Order> {
        // Stream order updates
    }
}
```

#### Pub/Sub Patterns
- Redis-backed pub/sub
- Kafka event streaming
- In-memory channels

### 11. Monitoring & Observability

#### Tracing GraphQL Operations
- Operation name tracking
- Query complexity metrics
- Resolver timing
- Error rates by field

#### Metrics
- Query execution time
- Resolver cache hit rate
- DataLoader batch efficiency
- Subscription connection count

### 12. GraphQL Best Practices

- Design schema from client perspective
- Avoid over-fetching and under-fetching
- Use connections for pagination
- Implement proper error handling
- Version schema carefully
- Document schema with descriptions
- Monitor query complexity
- Cache aggressively

## Tools & Libraries

- **async-graphql**: Rust GraphQL server library
- **async-graphql-axum**: Axum integration
- **dataloader**: Efficient batch loading
- **juniper**: Alternative GraphQL library
- **Apollo Studio**: Schema registry and monitoring
- **GraphQL Inspector**: Schema diffing and validation

## Hands-on Exercises

1. Create GraphQL schema for product catalog
2. Implement DataLoader for efficient data fetching
3. Add mutations for order creation
4. Set up schema federation across microservices
5. Implement field-level authorization
6. Add WebSocket subscriptions for real-time updates
7. Configure query complexity limits
8. Build integration tests for GraphQL API

## Best Practices

- Design schema around use cases, not database tables
- Use DataLoader to prevent N+1 queries
- Implement pagination with connections
- Provide clear error messages
- Document schema with descriptions
- Version schema carefully to avoid breaking changes
- Monitor query performance and complexity
- Use persisted queries in production

## Resources

- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/en/index.html)
- [GraphQL Specification](https://spec.graphql.org/)
- [Apollo Federation](https://www.apollographql.com/docs/federation/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)
- [DataLoader Pattern](https://github.com/graphql/dataloader)
