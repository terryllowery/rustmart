# Lesson 8: Database Integration with SQLx


## Overview
Real microservices need persistent storage. In this lesson, you'll integrate **PostgreSQL** into product-service using **SQLx**, a compile-time checked SQL library for Rust.

By the end of this lesson, you'll have:
- PostgreSQL running locally
- Database migrations
- Connection pooling
- CRUD operations for products
- Automatic query tracing (yes, SQLx integrates with OpenTelemetry!)

## What is SQLx?

SQLx is an **async SQL toolkit** for Rust with these killer features:
- **Compile-time query verification**: Queries are checked against your actual database schema at compile time
- **Async/await native**: Built for Tokio
- **Type-safe**: Results mapped to Rust structs automatically
- **Migration support**: Built-in migration runner
- **Multiple databases**: PostgreSQL, MySQL, SQLite

## Why PostgreSQL?

- Industry standard for web applications
- ACID compliant (reliable transactions)
- Rich data types (JSON, arrays, etc.)
- Excellent performance
- Great tooling and ecosystem

## Step 1: Install PostgreSQL

**On macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**On Linux:**
```bash
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

Verify it's running:
```bash
psql --version
```

## Step 2: Create the Database

```bash
# Connect as postgres user
psql postgres

# In the psql shell:
CREATE DATABASE rustmart;
CREATE USER rustmart_user WITH PASSWORD 'rustmart_pass';
GRANT ALL PRIVILEGES ON DATABASE rustmart TO rustmart_user;
\q
```

Test the connection:
```bash
psql -h localhost -U rustmart_user -d rustmart
# Enter password: rustmart_pass
\q
```

## Step 3: Add SQLx Dependencies

Update `product-service/Cargo.toml`:

```toml
[dependencies]
shared = { path = "../shared" }
axum.workspace = true
tokio = { workspace = true, features = ["full"] }
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-stdout = { version = "0.3", features = ["trace"] }
tracing-opentelemetry = "0.23"
tower-http = { version = "0.5", features = ["trace"] }

# SQLx dependencies
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Feature flags explained:**
- `runtime-tokio`: Use Tokio async runtime
- `postgres`: PostgreSQL driver
- `chrono`: Date/time types
- `uuid`: UUID support (common for primary keys)
- `migrate`: Built-in migration support

## Step 4: Install SQLx CLI

This tool helps with migrations and compile-time checking:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Verify:
```bash
sqlx --version
```

## Step 5: Create Database Migrations

Set the database URL environment variable (add to `~/.zshrc` or `~/.bashrc`):

```bash
export DATABASE_URL="postgresql://rustmart_user:rustmart_pass@localhost/rustmart"
```

Then reload your shell or run:
```bash
source ~/.zshrc  # or ~/.bashrc
```

Create the migrations directory:
```bash
cd ~/code/rustmart/product-service
mkdir migrations
```

Create the first migration:
```bash
sqlx migrate add create_products_table
```

This creates `migrations/<timestamp>_create_products_table.sql`. Edit it:

```sql
-- Create products table
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    inventory_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on name for faster searches
CREATE INDEX idx_products_name ON products(name);

-- Create a trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

Run the migration:
```bash
sqlx migrate run
```

Verify:
```bash
psql $DATABASE_URL -c "SELECT * FROM products;"
```

You should see an empty table with the columns.

## Step 6: Update the Product Model

We need to update the `Product` model to match the database schema.

Update `shared/src/models.rs`:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub price: rust_decimal::Decimal,
    pub inventory_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub price: f64,
    pub inventory_count: i32,
}

// ... keep User, Order, OrderItem as-is ...
```

**Key changes:**
- `sqlx::FromRow`: Derive macro that maps SQL rows to structs
- `uuid::Uuid`: Proper UUID type instead of String
- `rust_decimal::Decimal`: Precise decimal type for money (not f64!)
- `created_at`, `updated_at`: Timestamps for auditing

Add dependencies to `shared/Cargo.toml`:

```toml
[dependencies]
serde.workspace = true
thiserror.workspace = true
axum.workspace = true
sqlx = { version = "0.7", features = ["uuid", "chrono", "decimal"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.33", features = ["serde"] }
```

## Step 7: Create a Database Module

Create `product-service/src/db.rs`:

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}
```

This creates a **connection pool** so you don't open/close connections on every request.

## Step 8: Create a Repository Layer

Create `product-service/src/repository.rs`:

```rust
use shared::models::{Product, CreateProductRequest};
use shared::error::ApiError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all(&self) -> Result<Vec<Product>, ApiError> {
        let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        Ok(products)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound(format!("Product {} not found", id)))?;

        Ok(product)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, req: CreateProductRequest) -> Result<Product, ApiError> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (name, price, inventory_count)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(req.name)
        .bind(rust_decimal::Decimal::from_f64_retain(req.price).unwrap())
        .bind(req.inventory_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

        Ok(product)
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Product {} not found", id)));
        }

        Ok(())
    }
}
```

**Key patterns:**
- `query_as::<_, Product>`: Maps SQL results to Product struct
- `bind()`: Parameterized queries prevent SQL injection
- `fetch_all()` / `fetch_one()` / `fetch_optional()`: Different fetch modes
- `#[tracing::instrument(skip(self))]`: Traces queries (skip self to avoid logging entire pool)

## Step 9: Update the Router with Real Data

Update `product-service/src/lib.rs`:

```rust
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use shared::models::{Product, CreateProductRequest};
use shared::error::ApiError;
use sqlx::PgPool;
use uuid::Uuid;

mod repository;
use repository::ProductRepository;

pub mod db;

#[derive(Clone)]
pub struct AppState {
    pub repo: ProductRepository,
}

pub fn create_router(pool: PgPool) -> Router {
    let repo = ProductRepository::new(pool);
    let state = AppState { repo };

    Router::new()
        .route("/health", get(health_check))
        .route("/products", get(get_products).post(create_product))
        .route("/products/:id", get(get_product_by_id).delete(delete_product))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

#[tracing::instrument(skip(state))]
async fn get_products(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let products = state.repo.get_all().await?;
    Ok(Json(products))
}

#[tracing::instrument(skip(state))]
async fn get_product_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.get_by_id(id).await?;
    Ok(Json(product))
}

#[tracing::instrument(skip(state))]
async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.create(req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(product)))
}

#[tracing::instrument(skip(state))]
async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.repo.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
```

**New patterns:**
- `State`: Axum extractor for shared state
- `with_state()`: Attach state to router
- Repository pattern: Separates DB logic from HTTP handlers

Declare the modules in `product-service/src/main.rs`:

```rust
mod db;
mod repository;
```

## Step 10: Update main.rs

Update `product-service/src/main.rs` to create the pool:

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod db;

#[tokio::main]
async fn main() {
    // Initialize OpenTelemetry tracer
    let tracer = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(tracer)
        .with_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "product-service"),
                KeyValue::new("service.version", "0.1.0"),
            ])),
        )
        .build();
    
    global::set_tracer_provider(provider.clone());

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(provider.tracer("product-service"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    tracing::info!("Starting product-service with OpenTelemetry...");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create database connection pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");

    // Create the Axum router with database pool
    let app = product_service::create_router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();
    
    tracing::info!("Server listening on http://127.0.0.1:8001");
    
    axum::serve(listener, app).await.unwrap();

    global::shutdown_tracer_provider();
}
```

**New additions:**
- `DATABASE_URL` from environment
- `create_pool()` creates connection pool
- `sqlx::migrate!()` runs migrations automatically on startup

## Step 11: Test the Full Stack

Make sure `DATABASE_URL` is set, then run:

```bash
cd ~/code/rustmart
RUST_LOG=info cargo run -p product-service
```

**Create a product:**
```bash
curl -X POST http://localhost:8001/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Mechanical Keyboard",
    "price": 149.99,
    "inventory_count": 25
  }'
```

**Get all products:**
```bash
curl http://localhost:8001/products
```

**Get by ID (copy UUID from previous response):**
```bash
curl http://localhost:8001/products/<UUID>
```

**Delete a product:**
```bash
curl -X DELETE http://localhost:8001/products/<UUID>
```

## Step 12: Verify Query Tracing

Look at your console output when making requests. You should see:
- HTTP request spans from tower-http
- Repository method spans from `#[tracing::instrument]`
- SQL query information (not full SQL, but timing/result counts)

SQLx automatically integrates with `tracing`, so your database operations are visible in traces!

## Key Takeaways

1. **SQLx is compile-time safe**: Queries checked against real schema
2. **Connection pooling**: Reuse connections for performance
3. **Migrations are code**: Version controlled, automated
4. **Repository pattern**: Clean separation of concerns
5. **Async all the way**: No blocking IO in your web service
6. **Automatic tracing**: SQLx queries show up in OTel traces

## Challenges

1. **Add an update endpoint**: `PUT /products/:id` to update product details
2. **Add pagination**: Modify `get_all()` to accept `limit` and `offset` query params
3. **Add filtering**: Allow filtering products by price range
4. **Add seed data**: Create a migration that inserts sample products

<details>
<summary>Challenge 1 Solution: Update Endpoint</summary>

In `repository.rs`:
```rust
pub async fn update(&self, id: Uuid, req: CreateProductRequest) -> Result<Product, ApiError> {
    let product = sqlx::query_as::<_, Product>(
        r#"
        UPDATE products
        SET name = $1, price = $2, inventory_count = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(req.name)
    .bind(rust_decimal::Decimal::from_f64_retain(req.price).unwrap())
    .bind(req.inventory_count)
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| ApiError::InternalError(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Product {} not found", id)))?;

    Ok(product)
}
```

In `lib.rs`:
```rust
use axum::routing::put;

// Add to router:
.route("/products/:id", get(get_product_by_id).put(update_product).delete(delete_product))

async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.update(id, req).await?;
    Ok(Json(product))
}
```

</details>

## Troubleshooting

**"database does not exist"**: Run the CREATE DATABASE command again

**"password authentication failed"**: Check your DATABASE_URL credentials

**"no such table products"**: Run `sqlx migrate run`

**Compile error about missing types**: Make sure shared/Cargo.toml has sqlx deps

## Next Steps

In **Lesson 9**, you'll create Docker containers for your services and PostgreSQL, then use Docker Compose to run everything together. This sets you up for deploying to Kubernetes later!

## Official Documentation

- [SQLx Documentation](https://docs.rs/sqlx/)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [PostgreSQL Tutorial](https://www.postgresql.org/docs/current/tutorial.html)
- [Rust Decimal](https://docs.rs/rust_decimal/)
