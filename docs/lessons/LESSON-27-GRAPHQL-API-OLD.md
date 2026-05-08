# Lesson 27: GraphQL API Layer (Outline)

## Overview
Build a flexible GraphQL API for RustMart using async-graphql, providing clients with precise data fetching, schema stitching across microservices, and optimized query resolution.

## Core Topics

### 1. GraphQL Fundamentals
- GraphQL vs REST trade-offs
- Schema Definition Language (SDL)
- Queries, mutations, subscriptions
- Type system and introspection
- Resolvers and data loading

### 2. async-graphql Framework

#### Schema Definition
```rust
use async_graphql::{Object, Schema, SimpleObject};

#[derive(SimpleObject)]
struct Product {
    id: ID,
    name: String,
    price: f64,
    inventory_count: i32,
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn product(&self, id: ID) -> Result<Product> {
        // Fetch product
    }
    
    async fn products(&self, limit: i32) -> Result<Vec<Product>> {
        // Fetch products
    }
}
```

#### Mutations
- Create, update, delete operations
- Input validation
- Error handling
- Optimistic updates

#### Subscriptions
- Real-time updates via WebSockets
- Event streaming
- Subscription filtering
- Connection management

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
