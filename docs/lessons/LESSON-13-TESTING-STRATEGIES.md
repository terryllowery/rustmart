# Lesson 13: Testing Strategies for Microservices

## Overview
Testing microservices is different from testing monoliths. You have distributed systems, async messaging, external dependencies, and more complexity. This lesson covers comprehensive testing strategies.

By the end of this lesson, you'll have:
- Unit tests for business logic
- Integration tests for databases
- API tests for HTTP endpoints
- Contract tests between services
- Test fixtures and mocking

## The Testing Pyramid

```
       /\
      /E2\     ← Few, slow, expensive
     /----\
    / API  \   ← Some, medium speed
   /--------\
  /  INTEG   \ ← More, faster
 /------------\
/    UNIT      \ ← Many, fast, cheap
-----------------
```

- **Unit Tests**: Test individual functions/modules in isolation
- **Integration Tests**: Test database, external APIs, file system
- **API Tests**: Test HTTP endpoints end-to-end
- **E2E Tests**: Test entire user flows across services

## Step 1: Unit Tests for Business Logic

Let's test the product repository logic.

In `product-service/src/repository.rs`, add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Helper to create test pool (requires DATABASE_URL)
    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");
        
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn test_create_product() {
        let pool = create_test_pool().await;
        let repo = ProductRepository::new(pool);

        let request = CreateProductRequest {
            name: "Test Product".to_string(),
            price: 99.99,
            inventory_count: 10,
        };

        let product = repo.create(request).await.unwrap();

        assert_eq!(product.name, "Test Product");
        assert_eq!(product.inventory_count, 10);
        
        // Cleanup
        repo.delete(product.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_nonexistent_product() {
        let pool = create_test_pool().await;
        let repo = ProductRepository::new(pool);

        let result = repo.get_by_id(Uuid::nil()).await;

        assert!(result.is_err());
        match result {
            Err(ApiError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_product() {
        let pool = create_test_pool().await;
        let repo = ProductRepository::new(pool);

        // Create
        let request = CreateProductRequest {
            name: "Delete Me".to_string(),
            price: 1.0,
            inventory_count: 1,
        };
        let product = repo.create(request).await.unwrap();

        // Delete
        let result = repo.delete(product.id).await;
        assert!(result.is_ok());

        // Verify deleted
        let get_result = repo.get_by_id(product.id).await;
        assert!(get_result.is_err());
    }
}
```

Run tests:
```bash
cd ~/code/rustmart
DATABASE_URL=postgresql://rustmart_user:rustmart_pass@localhost/rustmart cargo test -p product-service
```

## Step 2: Integration Tests with Test Database

Production and test databases should be separate. Create a test-specific setup.

Create `product-service/tests/integration_test.rs`:

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt; // For `oneshot`
use sqlx::PgPool;
use uuid::Uuid;

// Helper to setup test environment
async fn setup() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://rustmart_user:rustmart_pass@localhost/rustmart_test".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Clear existing data
    sqlx::query("TRUNCATE TABLE products")
        .execute(&pool)
        .await
        .expect("Failed to clear products table");

    pool
}

#[tokio::test]
async fn test_create_product_endpoint() {
    let pool = setup().await;
    let kafka = shared::kafka::KafkaProducer::new("localhost:9094");
    let app = product_service::create_router(pool, kafka);

    let request = Request::builder()
        .method("POST")
        .uri("/products")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "Integration Test Product",
                "price": 149.99,
                "inventory_count": 50
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let product: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(product["name"], "Integration Test Product");
    assert_eq!(product["price"], "149.99");
}

#[tokio::test]
async fn test_get_all_products() {
    let pool = setup().await;
    let kafka = shared::kafka::KafkaProducer::new("localhost:9094");
    
    // Insert test data directly
    sqlx::query(
        "INSERT INTO products (name, price, inventory_count) VALUES ($1, $2, $3)"
    )
    .bind("Product 1")
    .bind(rust_decimal::Decimal::from_f64_retain(10.0).unwrap())
    .bind(5)
    .execute(&pool)
    .await
    .unwrap();

    let app = product_service::create_router(pool, kafka);

    let request = Request::builder()
        .uri("/products")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let products: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(products.len(), 1);
    assert_eq!(products[0]["name"], "Product 1");
}
```

Run integration tests:
```bash
TEST_DATABASE_URL=postgresql://rustmart_user:rustmart_pass@localhost/rustmart_test cargo test -p product-service --test integration_test
```

## Step 3: Test Fixtures

Fixtures provide reusable test data.

Create `product-service/tests/fixtures.rs`:

```rust
use shared::models::{Product, CreateProductRequest};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ProductFixture {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
}

impl ProductFixture {
    pub fn laptop() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Test Laptop".to_string(),
            price: 999.99,
        }
    }

    pub fn mouse() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Test Mouse".to_string(),
            price: 29.99,
        }
    }

    pub async fn insert(&self, pool: &PgPool) -> Product {
        let product = sqlx::query_as::<_, Product>(
            "INSERT INTO products (id, name, price, inventory_count) VALUES ($1, $2, $3, 10) RETURNING *"
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(rust_decimal::Decimal::from_f64_retain(self.price).unwrap())
        .fetch_one(pool)
        .await
        .expect("Failed to insert fixture");

        product
    }
}

// Use in tests:
#[tokio::test]
async fn test_with_fixtures() {
    let pool = setup().await;
    
    let laptop = ProductFixture::laptop().insert(&pool).await;
    let mouse = ProductFixture::mouse().insert(&pool).await;

    // Now test with these products...
}
```

## Step 4: Mocking External Dependencies

Use `mockito` to mock HTTP services.

Add to `product-service/Cargo.toml`:

```toml
[dev-dependencies]
mockito = "1.2"
```

Test with mock inventory service:

```rust
#[tokio::test]
async fn test_check_availability_with_mock() {
    use mockito::{mock, server_url};

    // Mock the inventory service response
    let _m = mock("GET", "/inventory/check/550e8400-e29b-41d4-a716-446655440001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"available": 100}"#)
        .create();

    // Use the mock server URL
    let client = InventoryClient::new(server_url());

    let available = client
        .check_availability(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            50,
        )
        .await
        .unwrap();

    assert!(available);
}
```

## Step 5: Contract Testing

Contract tests verify that services agree on API contracts. Use `pact` for consumer-driven contract testing.

### Producer (product-service)

Create `product-service/tests/contract_test.rs`:

```rust
// This test generates a contract that consumers can verify against

#[tokio::test]
async fn test_get_product_contract() {
    let pool = setup().await;
    let kafka = shared::kafka::KafkaProducer::new("localhost:9094");
    let app = product_service::create_router(pool.clone(), kafka);

    // Insert known product
    let product_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO products (id, name, price, inventory_count) VALUES ($1, $2, $3, $4)"
    )
    .bind(product_id)
    .bind("Contract Product")
    .bind(rust_decimal::Decimal::from_f64_retain(99.99).unwrap())
    .bind(10)
    .execute(&pool)
    .await
    .unwrap();

    let request = Request::builder()
        .uri(format!("/products/{}", product_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let product: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify contract structure
    assert!(product.get("id").is_some());
    assert!(product.get("name").is_some());
    assert!(product.get("price").is_some());
    assert!(product.get("inventory_count").is_some());
    assert!(product.get("created_at").is_some());
    assert!(product.get("updated_at").is_some());
}
```

### Consumer (order-service)

Order service tests verify it can handle product-service responses:

```rust
#[tokio::test]
async fn test_order_service_expects_product_format() {
    // Order service expects these fields from product service
    let product_json = json!({
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "name": "Laptop",
        "price": "999.99",
        "inventory_count": 50,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    });

    // Deserialize to Product struct
    let product: Product = serde_json::from_value(product_json).unwrap();

    assert_eq!(product.name, "Laptop");
}
```

If product-service changes its API, consumer tests fail!

## Step 6: Test Kafka Events

Test event publishing and consuming:

```rust
use shared::events::{DomainEvent, ProductCreatedEvent};
use shared::kafka::{KafkaProducer, KafkaConsumer};

#[tokio::test]
async fn test_product_created_event() {
    let producer = KafkaProducer::new("localhost:9094");
    
    let event = DomainEvent::ProductCreated(ProductCreatedEvent {
        event_id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        product_id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        price: "99.99".to_string(),
        inventory_count: 10,
    });

    // Publish event
    producer.publish("test.events", None, &event).await.unwrap();

    // Consumer would pick this up
    // In tests, we can verify the event structure is valid
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

    match deserialized {
        DomainEvent::ProductCreated(e) => {
            assert_eq!(e.name, "Test Product");
        }
        _ => panic!("Expected ProductCreated event"),
    }
}
```

## Step 7: Performance Testing

Use `criterion` for benchmarks.

Add to `Cargo.toml`:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "product_benchmark"
harness = false
```

Create `product-service/benches/product_benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use product_service::repository::ProductRepository;

fn benchmark_get_product(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let pool = runtime.block_on(async {
        sqlx::postgres::PgPoolOptions::new()
            .connect("postgresql://rustmart_user:rustmart_pass@localhost/rustmart")
            .await
            .unwrap()
    });

    let repo = ProductRepository::new(pool);

    c.bench_function("get_product_by_id", |b| {
        b.to_async(&runtime).iter(|| async {
            repo.get_by_id(black_box(Uuid::new_v4())).await.ok();
        });
    });
}

criterion_group!(benches, benchmark_get_product);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench -p product-service
```

## Testing Best Practices

1. **Arrange-Act-Assert**: Structure tests clearly
   ```rust
   // Arrange: Setup
   let pool = setup().await;
   let repo = ProductRepository::new(pool);
   
   // Act: Execute
   let result = repo.create(request).await;
   
   // Assert: Verify
   assert!(result.is_ok());
   ```

2. **Test names describe behavior**:
   - ❌ `test_1`, `test_product`
   - ✅ `test_create_product_returns_created_status`, `test_get_nonexistent_product_returns_404`

3. **Test one thing per test**:
   - Don't test create + update + delete in one test
   - Make separate tests for each operation

4. **Use test databases**:
   - Never test against production
   - Use separate `rustmart_test` database

5. **Clean up after tests**:
   - Truncate tables between tests
   - Use transactions that rollback

6. **Fast feedback loop**:
   - Unit tests run in milliseconds
   - Integration tests in seconds
   - Don't wait for slow tests

## CI/CD Integration

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_DB: rustmart_test
          POSTGRES_USER: rustmart_user
          POSTGRES_PASSWORD: rustmart_pass
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v3
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        env:
          DATABASE_URL: postgresql://rustmart_user:rustmart_pass@localhost/rustmart_test
        run: cargo test --all
```

## Key Takeaways

1. **Test pyramid**: Many unit tests, fewer integration, fewest E2E
2. **Test databases**: Separate from production
3. **Fixtures**: Reusable test data
4. **Mocking**: Isolate external dependencies
5. **Contract tests**: Verify service agreements
6. **Fast feedback**: Run tests in CI/CD

## Challenges

1. **Add snapshot testing**: Use `insta` crate for response snapshots
2. **Add property-based testing**: Use `proptest` for random inputs
3. **Add load testing**: Use `k6` or `gatling` for stress tests
4. **Add mutation testing**: Use `cargo-mutants` to verify test quality

## Next Steps

In **Lesson 14**, you'll deploy RustMart to **Kubernetes** with proper configs, secrets, and scaling!

## Official Documentation

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tokio::test](https://docs.rs/tokio/latest/tokio/attr.test.html)
- [mockito](https://docs.rs/mockito/)
- [criterion](https://docs.rs/criterion/)
- [Pact](https://docs.pact.io/)
