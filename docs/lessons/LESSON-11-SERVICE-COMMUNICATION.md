# Lesson 11: Service-to-Service Communication

## Overview
Microservices need to communicate with each other. In this lesson, you'll learn resilience patterns that prevent cascading failures and make your system robust.

By the end of this lesson, you'll have:
- HTTP client for service-to-service calls
- Circuit breaker pattern
- Retry logic with exponential backoff
- Timeout configuration
- Health checks for dependencies

## The Problem: Cascading Failures

Imagine:
1. Order service calls inventory service
2. Inventory service is down
3. Order service waits, times out
4. All order service threads blocked waiting
5. API gateway calls order service
6. API gateway threads blocked
7. **Entire system fails** because one service is down

This is called a **cascading failure**.

## Solution: Resilience Patterns

### Circuit Breaker
Like an electrical circuit breaker, it "trips" when too many failures occur:

```
CLOSED → failures → OPEN → timeout → HALF_OPEN → success → CLOSED
                     ↓
                  fail fast (don't even try)
```

States:
- **CLOSED**: Normal operation, requests go through
- **OPEN**: Too many failures, reject requests immediately
- **HALF_OPEN**: Try one request to see if service recovered

### Retry with Exponential Backoff
Don't give up on first failure, but don't spam the service:

```
Try 1: immediate
Try 2: wait 100ms
Try 3: wait 200ms
Try 4: wait 400ms
Try 5: give up
```

### Timeouts
Don't wait forever:

```rust
let response = client.get(url)
    .timeout(Duration::from_secs(5))
    .send()
    .await?;
```

## Step 1: Create Service Client Library

Let's build a reusable HTTP client in the shared crate.

Create `shared/src/client.rs`:

```rust
use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ServiceClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl ServiceClient {
    pub fn new(base_url: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            timeout,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, path);
        tracing::info!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    #[tracing::instrument(skip(self, body))]
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        tracing::info!("POST {}", url);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| ClientError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| ClientError::DeserializationError(e.to_string()))
        } else {
            Err(ClientError::HttpError(status))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error: {0}")]
    HttpError(StatusCode),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
```

Add reqwest to `shared/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
reqwest = { version = "0.11", features = ["json"] }
```

Export in `shared/src/lib.rs`:

```rust
pub mod client;
pub mod config;
pub mod error;
pub mod models;
```

## Step 2: Add Circuit Breaker

We'll use the `failsafe` crate for circuit breaker pattern.

Add to `shared/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
failsafe = "1.2"
```

Create `shared/src/circuit_breaker.rs`:

```rust
use failsafe::{CircuitBreaker, Config, Error as FailsafeError};
use std::time::Duration;

pub fn create_circuit_breaker() -> CircuitBreaker {
    let config = Config::new()
        .failure_rate_threshold(0.5) // Open at 50% failure rate
        .wait_duration_in_open_state(Duration::from_secs(30)) // Stay open for 30s
        .ring_buffer_size_in_half_open_state(10) // Test with 10 requests
        .ring_buffer_size_in_closed_state(100); // Track last 100 requests

    CircuitBreaker::new(config)
}
```

Export it:

```rust
// shared/src/lib.rs
pub mod circuit_breaker;
```

## Step 3: Add Retry Logic

Create `shared/src/retry.rs`:

```rust
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
        }
    }
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }

    #[tracing::instrument(skip(self, operation))]
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 1;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        tracing::info!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.max_attempts {
                        tracing::error!("Operation failed after {} attempts: {}", attempt, e);
                        return Err(e);
                    }

                    let delay = self.base_delay * attempt;
                    tracing::warn!(
                        "Attempt {} failed: {}. Retrying in {:?}",
                        attempt,
                        e,
                        delay
                    );

                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}
```

Export it:

```rust
// shared/src/lib.rs
pub mod retry;
```

## Step 4: Create Inventory Service Client

Now let's say product-service needs to check inventory. Create a client for it.

Create `product-service/src/inventory_client.rs`:

```rust
use shared::client::{ServiceClient, ClientError};
use shared::retry::RetryPolicy;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct InventoryClient {
    client: ServiceClient,
    retry_policy: RetryPolicy,
}

impl InventoryClient {
    pub fn new(base_url: String) -> Self {
        let client = ServiceClient::new(base_url, Duration::from_secs(5));
        let retry_policy = RetryPolicy::default();

        Self {
            client,
            retry_policy,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn check_availability(
        &self,
        product_id: Uuid,
        quantity: i32,
    ) -> Result<bool, ClientError> {
        let path = format!("/inventory/check/{}", product_id);
        
        // Use retry policy
        self.retry_policy
            .execute(|| async {
                let response: InventoryCheckResponse = self.client.get(&path).await?;
                Ok(response.available >= quantity)
            })
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn reserve(
        &self,
        product_id: Uuid,
        quantity: i32,
    ) -> Result<ReservationResponse, ClientError> {
        let path = "/inventory/reserve";
        let request = ReserveRequest {
            product_id,
            quantity,
        };

        self.retry_policy
            .execute(|| async { self.client.post(&path, &request).await })
            .await
    }
}

#[derive(Debug, Serialize)]
struct ReserveRequest {
    product_id: Uuid,
    quantity: i32,
}

#[derive(Debug, Deserialize)]
struct InventoryCheckResponse {
    available: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReservationResponse {
    pub reservation_id: String,
    pub expires_at: String,
}
```

## Step 5: Use Client in Product Service

Update `product-service/src/lib.rs` to use the inventory client:

```rust
mod inventory_client;
use inventory_client::InventoryClient;

#[derive(Clone)]
pub struct AppState {
    pub repo: ProductRepository,
    pub inventory_client: InventoryClient,
}

pub fn create_router(pool: PgPool, inventory_service_url: String) -> Router {
    let repo = ProductRepository::new(pool);
    let inventory_client = InventoryClient::new(inventory_service_url);
    
    let state = AppState {
        repo,
        inventory_client,
    };

    // ... rest of router setup
}

// New handler that checks inventory
#[tracing::instrument(skip(state))]
async fn check_product_availability(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<AvailabilityParams>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.get_by_id(id).await?;
    
    // Call inventory service
    let available = state
        .inventory_client
        .check_availability(id, params.quantity)
        .await
        .map_err(|e| ApiError::InternalError(format!("Inventory check failed: {}", e)))?;

    Ok(Json(AvailabilityResponse {
        product_id: id,
        quantity: params.quantity,
        available,
    }))
}

#[derive(serde::Deserialize)]
struct AvailabilityParams {
    quantity: i32,
}

#[derive(serde::Serialize)]
struct AvailabilityResponse {
    product_id: Uuid,
    quantity: i32,
    available: bool,
}
```

## Step 6: Add Health Check Endpoint

Services should expose health checks so other services know if they're healthy.

Add to `product-service/src/lib.rs`:

```rust
use axum::http::StatusCode;

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    database: String,
    dependencies: Vec<DependencyHealth>,
}

#[derive(serde::Serialize)]
struct DependencyHealth {
    name: String,
    status: String,
}

#[tracing::instrument(skip(state))]
async fn health_check_detailed(State(state): State<AppState>) -> impl IntoResponse {
    // Check database
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.repo.pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check inventory service (simple ping)
    let inventory_status = match state
        .inventory_client
        .check_availability(Uuid::nil(), 0)
        .await
    {
        Ok(_) | Err(_) => "healthy", // Any response means service is up
    };

    let health = HealthResponse {
        status: if db_status == "healthy" { "healthy".to_string() } else { "degraded".to_string() },
        database: db_status.to_string(),
        dependencies: vec![DependencyHealth {
            name: "inventory-service".to_string(),
            status: inventory_status.to_string(),
        }],
    };

    let status_code = if health.status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health))
}
```

Add route:

```rust
Router::new()
    .route("/health", get(health_check))
    .route("/health/detailed", get(health_check_detailed))
    // ... other routes
```

## Step 7: Implement Graceful Degradation

When a dependency fails, don't fail the entire request. Degrade gracefully:

```rust
#[tracing::instrument(skip(state))]
async fn get_product_with_availability(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.get_by_id(id).await?;
    
    // Try to get inventory, but don't fail if unavailable
    let inventory_available = state
        .inventory_client
        .check_availability(id, 1)
        .await
        .ok(); // Convert Result to Option

    Ok(Json(ProductWithAvailability {
        id: product.id,
        name: product.name,
        price: product.price,
        inventory_count: product.inventory_count,
        availability_check: inventory_available.map(|avail| AvailabilityStatus {
            available: avail,
            last_checked: chrono::Utc::now(),
        }),
    }))
}

#[derive(serde::Serialize)]
struct ProductWithAvailability {
    id: Uuid,
    name: String,
    price: rust_decimal::Decimal,
    inventory_count: i32,
    availability_check: Option<AvailabilityStatus>,
}

#[derive(serde::Serialize)]
struct AvailabilityStatus {
    available: bool,
    last_checked: chrono::DateTime<chrono::Utc>,
}
```

If inventory service is down, the product response still works, but `availability_check` is null.

## Key Takeaways

1. **Circuit breakers**: Fail fast when services are down
2. **Retries**: Don't give up on transient failures
3. **Timeouts**: Don't wait forever
4. **Health checks**: Let other services know your status
5. **Graceful degradation**: Partial functionality > no functionality

## Patterns Summary

| Pattern | When to Use | Example |
|---------|------------|---------|
| Circuit Breaker | Prevent cascading failures | Don't call down services |
| Retry | Transient failures | Network hiccups |
| Timeout | Slow services | Database query takes too long |
| Fallback | Critical availability | Return cached data |
| Health Check | Service discovery | Is this instance healthy? |

## Challenges

1. **Add caching**: Cache inventory checks for 30 seconds
2. **Add bulkhead pattern**: Limit concurrent requests to a service
3. **Add fallback**: Return cached product data if database fails
4. **Add metrics**: Track circuit breaker state changes

<details>
<summary>Challenge 3 Solution: Fallback with Cache</summary>

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ProductCache {
    cache: Arc<RwLock<HashMap<Uuid, Product>>>,
}

impl ProductCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, id: &Uuid) -> Option<Product> {
        let cache = self.cache.read().await;
        cache.get(id).cloned()
    }

    pub async fn set(&self, id: Uuid, product: Product) {
        let mut cache = self.cache.write().await;
        cache.insert(id, product);
    }
}

// In handler:
async fn get_product_with_fallback(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Try database first
    match state.repo.get_by_id(id).await {
        Ok(product) => {
            // Update cache
            state.cache.set(id, product.clone()).await;
            Ok(Json(product))
        }
        Err(_) => {
            // Fallback to cache
            if let Some(cached) = state.cache.get(&id).await {
                tracing::warn!("Database failed, serving from cache");
                Ok(Json(cached))
            } else {
                Err(ApiError::NotFound(format!("Product {} not found", id)))
            }
        }
    }
}
```

</details>

## Next Steps

In **Lesson 12**, you'll add **Kafka** for asynchronous messaging. This lets services communicate without waiting for responses!

## Official Documentation

- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [failsafe crate](https://docs.rs/failsafe/)
- [reqwest](https://docs.rs/reqwest/)
- [Microservice Patterns](https://microservices.io/patterns/index.html)
- [Release It! (book on resilience)](https://pragprog.com/titles/mnee2/release-it-second-edition/)
