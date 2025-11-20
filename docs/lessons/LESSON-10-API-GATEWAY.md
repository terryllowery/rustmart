# Lesson 10: API Gateway Service

## Overview
In microservices architectures, an **API Gateway** is the single entry point for clients. Instead of calling services directly, clients hit the gateway, which routes requests to the appropriate backend service.

By the end of this lesson, you'll have:
- A new api-gateway service
- Request routing to product-service
- JWT authentication middleware
- Request/response logging
- Load balancing basics

## Why an API Gateway?

Without a gateway:
```
Client → product-service (port 8001)
Client → order-service (port 8002)
Client → inventory-service (port 8003)
```

Clients need to know all service addresses, handle auth separately, etc.

With a gateway:
```
Client → api-gateway (port 8000)
  └─> Routes to product-service
  └─> Routes to order-service
  └─> Routes to inventory-service
```

Benefits:
- **Single entry point**: Clients only need one URL
- **Authentication**: Centralized auth logic
- **Rate limiting**: Protect backend services
- **Request transformation**: Modify requests/responses
- **Observability**: Log all traffic in one place

## Step 1: Initialize the API Gateway Service

```bash
cd ~/code/rustmart
mkdir api-gateway/src
touch api-gateway/src/lib.rs
touch api-gateway/src/main.rs
```

Create `api-gateway/Cargo.toml`:

```toml
[package]
name = "api-gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
axum.workspace = true
tokio = { workspace = true, features = ["full"] }
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
tower-http = { version = "0.5", features = ["trace", "cors"] }

# HTTP client for proxying
reqwest = { version = "0.11", features = ["json"] }

# JWT auth
jsonwebtoken = "9.2"

# OpenTelemetry (for tracing)
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.23"
```

## Step 2: Create Configuration

Create `api-gateway/src/config.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub services: ServicesConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServicesConfig {
    pub product_service_url: String,
    pub order_service_url: String,
    pub inventory_service_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig {
                host: std::env::var("GATEWAY_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("GATEWAY_PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .parse()
                    .expect("GATEWAY_PORT must be a valid port number"),
            },
            services: ServicesConfig {
                product_service_url: std::env::var("PRODUCT_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8001".to_string()),
                order_service_url: std::env::var("ORDER_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8002".to_string()),
                inventory_service_url: std::env::var("INVENTORY_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8003".to_string()),
            },
            auth: AuthConfig {
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            },
        }
    }
}
```

## Step 3: Create JWT Authentication Middleware

Create `api-gateway/src/auth.rs`:

```rust
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub email: String,
    pub exp: usize,   // Expiry time
}

#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check "Bearer <token>" format
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    // Validate JWT
    let claims = decode_jwt(token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add claims to request extensions so handlers can access them
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub fn encode_jwt(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
```

## Step 4: Create Proxy Handler

Create `api-gateway/src/proxy.rs`:

```rust
use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use reqwest::Client;

#[derive(Clone)]
pub struct ProxyState {
    pub client: Client,
    pub product_service_url: String,
}

#[tracing::instrument(skip(state, request))]
pub async fn proxy_to_product_service(
    State(state): State<ProxyState>,
    request: Request,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let query = request.uri().query().unwrap_or("");
    
    // Build target URL
    let target_url = format!(
        "{}{}{}",
        state.product_service_url,
        path,
        if query.is_empty() { String::new() } else { format!("?{}", query) }
    );

    tracing::info!("Proxying to: {}", target_url);

    // Forward request
    let method = request.method().clone();
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = state
        .client
        .request(method, &target_url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    // Convert reqwest response to axum response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut axum_response = Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap();

    *axum_response.headers_mut() = headers;

    Ok(axum_response)
}
```

## Step 5: Build the Router

Create `api-gateway/src/lib.rs`:

```rust
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod config;
pub mod proxy;

use auth::AuthState;
use proxy::ProxyState;

pub fn create_router(config: config::Config) -> Router {
    let auth_state = AuthState {
        jwt_secret: config.auth.jwt_secret.clone(),
    };

    let proxy_state = ProxyState {
        client: reqwest::Client::new(),
        product_service_url: config.services.product_service_url.clone(),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/products", get(proxy::proxy_to_product_service).post(proxy::proxy_to_product_service))
        .route("/products/:id", get(proxy::proxy_to_product_service))
        .with_state(proxy_state.clone())
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth::auth_middleware,
        ));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> &'static str {
    "API Gateway OK"
}

#[tracing::instrument]
async fn login(
    axum::extract::Json(payload): axum::extract::Json<LoginRequest>,
) -> Result<axum::Json<LoginResponse>, axum::http::StatusCode> {
    // In a real app, verify credentials against database
    // For now, accept any email/password for demo
    
    let claims = auth::Claims {
        sub: "user123".to_string(),
        email: payload.email.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = auth::encode_jwt(&claims, "dev-secret-change-in-production")
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(axum::Json(LoginResponse { token }))
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
}
```

## Step 6: Create main.rs

Create `api-gateway/src/main.rs`:

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod auth;
mod config;
mod proxy;

#[tokio::main]
async fn main() {
    // Initialize OpenTelemetry
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "api-gateway"),
                KeyValue::new("service.version", "0.1.0"),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize OTLP tracer");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    tracing::info!("Starting API Gateway...");

    // Load configuration
    let config = config::Config::from_env();
    tracing::info!("Configuration loaded: {:?}", config);

    // Create router
    let app = api_gateway::create_router(config.clone());

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("API Gateway listening on {}", addr);

    axum::serve(listener, app).await.unwrap();

    global::shutdown_tracer_provider();
}
```

## Step 7: Add chrono Dependency

The login handler uses `chrono`, so add it to `api-gateway/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
chrono = "0.4"
```

## Step 8: Update docker-compose.yml

Add the API gateway to `docker-compose.yml`:

```yaml
  api-gateway:
    build:
      context: .
      dockerfile: api-gateway/Dockerfile
    container_name: rustmart-api-gateway
    environment:
      GATEWAY_HOST: 0.0.0.0
      GATEWAY_PORT: 8000
      PRODUCT_SERVICE_URL: http://product-service:8001
      JWT_SECRET: ${JWT_SECRET:-dev-secret-change-in-production}
      RUST_LOG: info
      OTEL_EXPORTER_OTLP_ENDPOINT: http://jaeger:4317
    ports:
      - "8000:8000"
    depends_on:
      product-service:
        condition: service_started
```

Create `api-gateway/Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY api-gateway ./api-gateway

# Build the application
WORKDIR /app/api-gateway
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api-gateway /app/api-gateway

EXPOSE 8000

CMD ["/app/api-gateway"]
```

## Step 9: Test the Gateway

Start the stack:

```bash
cd ~/code/rustmart
docker-compose up --build
```

**Test health check:**
```bash
curl http://localhost:8000/health
```

**Get a JWT token:**
```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password"
  }'
```

Save the token from the response.

**Call protected endpoint with token:**
```bash
curl http://localhost:8000/products \
  -H "Authorization: Bearer <YOUR_TOKEN>"
```

**Try without token (should fail):**
```bash
curl http://localhost:8000/products
# Returns 401 Unauthorized
```

## Step 10: View Distributed Traces

Open Jaeger UI at http://localhost:16686

Select "api-gateway" from the service dropdown and click "Find Traces".

You'll see:
1. Request hits api-gateway
2. Auth middleware validates JWT
3. Request proxied to product-service
4. Product-service queries database
5. Response returned through gateway

This is **distributed tracing across services**!

## Key Takeaways

1. **API Gateway pattern**: Single entry point for all client requests
2. **JWT authentication**: Stateless auth that scales
3. **Request proxying**: Forward requests to backend services
4. **Middleware**: Reusable auth logic
5. **Distributed tracing**: See requests flow through multiple services

## Challenges

1. **Add rate limiting**: Limit requests per user/IP
2. **Add CORS**: Allow browser clients from different origins
3. **Add response caching**: Cache GET requests with Redis
4. **Add request validation**: Validate request schemas before proxying
5. **Add circuit breaker**: Stop proxying to unhealthy services

<details>
<summary>Challenge 2 Solution: Add CORS</summary>

In `lib.rs`:

```rust
use tower_http::cors::{CorsLayer, Any};

pub fn create_router(config: config::Config) -> Router {
    // ... existing code ...

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
}
```

Note: `allow_origin(Any)` is for development only! In production, specify allowed origins.

</details>

## Next Steps

In **Lesson 11**, you'll implement service-to-service communication patterns like circuit breakers and retries, making your microservices resilient!

## Official Documentation

- [API Gateway Pattern](https://microservices.io/patterns/apigateway.html)
- [JWT.io](https://jwt.io/)
- [Axum Middleware](https://docs.rs/axum/latest/axum/middleware/)
- [reqwest](https://docs.rs/reqwest/)
- [tower-http](https://docs.rs/tower-http/)
