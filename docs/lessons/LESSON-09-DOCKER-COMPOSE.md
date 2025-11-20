# Lesson 9: Docker Compose - Containerizing RustMart

## Overview
Now you'll containerize product-service and run the entire stack (service + PostgreSQL + Jaeger) with **Docker Compose**. This is essential for:
- **Reproducible environments**: Same setup on any machine
- **Local development**: Run all dependencies with one command
- **Cloud deployment**: Containers are the foundation for Kubernetes
- **Testing observability**: Jaeger UI lets you visualize traces before connecting to Instana

By the end of this lesson, you'll have:
- Dockerfile for product-service
- docker-compose.yml with all services
- Jaeger UI running locally
- Complete observability stack

## What is Docker?

**Docker** packages applications into **containers**: isolated, portable units that include everything needed to run (code, runtime, dependencies, OS libraries).

Key concepts:
- **Image**: Template for containers (like a class in OOP)
- **Container**: Running instance of an image (like an object)
- **Dockerfile**: Recipe for building an image
- **Docker Compose**: Tool to run multi-container applications

## Step 1: Create Dockerfile for Product Service

Create `product-service/Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY product-service ./product-service

# Build the application
WORKDIR /app/product-service
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/product-service /app/product-service

# Copy migrations
COPY product-service/migrations /app/migrations

EXPOSE 8001

CMD ["/app/product-service"]
```

**What's happening:**
1. **Multi-stage build**: Build in rust image, run in slim Debian (smaller final image)
2. **Workspace aware**: Copies both shared and product-service
3. **Release build**: Optimized binary
4. **Migrations included**: Needed for startup
5. **Minimal runtime**: Only essential dependencies

## Step 2: Create .dockerignore

Create `product-service/.dockerignore`:

```
target/
Cargo.lock
.env
*.log
.git/
.github/
docs/
```

This speeds up builds by excluding unnecessary files.

## Step 3: Test Building the Image

```bash
cd ~/code/rustmart
docker build -t rustmart/product-service:latest -f product-service/Dockerfile .
```

**Note**: This takes a few minutes the first time (downloading crates, compiling).

Verify the image:
```bash
docker images | grep product-service
```

## Step 4: Create docker-compose.yml

Create `~/code/rustmart/docker-compose.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    container_name: rustmart-postgres
    environment:
      POSTGRES_DB: rustmart
      POSTGRES_USER: rustmart_user
      POSTGRES_PASSWORD: rustmart_pass
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rustmart_user -d rustmart"]
      interval: 10s
      timeout: 5s
      retries: 5

  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: rustmart-jaeger
    environment:
      COLLECTOR_OTLP_ENABLED: "true"
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:16686"]
      interval: 10s
      timeout: 5s
      retries: 5

  product-service:
    build:
      context: .
      dockerfile: product-service/Dockerfile
    container_name: rustmart-product-service
    environment:
      DATABASE_URL: postgresql://rustmart_user:rustmart_pass@postgres/rustmart
      RUST_LOG: info
      OTEL_EXPORTER_OTLP_ENDPOINT: http://jaeger:4317
    ports:
      - "8001:8001"
    depends_on:
      postgres:
        condition: service_healthy
      jaeger:
        condition: service_healthy

volumes:
  postgres_data:
```

**Key features:**
- **postgres**: Database with health check
- **jaeger**: All-in-one Jaeger (collector + UI + storage)
- **product-service**: Your microservice
- **depends_on**: Service starts only when dependencies are healthy
- **volumes**: Persistent PostgreSQL data

## Step 5: Update Product Service to Use OTLP

We need to switch from stdout exporter to OTLP for Jaeger.

Update `product-service/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
```

Update `product-service/src/main.rs`:

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod db;

#[tokio::main]
async fn main() {
    // Get OTLP endpoint from environment (defaults to Jaeger)
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    // Initialize OpenTelemetry with OTLP exporter
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "product-service"),
                KeyValue::new("service.version", "0.1.0"),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize OTLP tracer");

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize tracing with both console and OpenTelemetry layers
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8001")
        .await
        .unwrap();
    
    tracing::info!("Server listening on http://0.0.0.0:8001");
    
    axum::serve(listener, app).await.unwrap();

    global::shutdown_tracer_provider();
}
```

**Changes:**
- `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable
- `opentelemetry_otlp::new_pipeline()` instead of stdout
- `install_batch()` for better performance (batches spans)
- Bind to `0.0.0.0` instead of `127.0.0.1` (Docker networking)

## Step 6: Start the Stack

```bash
cd ~/code/rustmart
docker-compose up --build
```

**What happens:**
1. PostgreSQL starts and initializes
2. Jaeger starts
3. product-service builds (if needed)
4. product-service runs migrations
5. product-service starts listening

You'll see logs from all three services interleaved.

## Step 7: Verify Everything Works

**Check services are running:**
```bash
docker-compose ps
```

All should show "Up" status.

**Test the API:**
```bash
# Health check
curl http://localhost:8001/health

# Create a product
curl -X POST http://localhost:8001/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Docker Laptop",
    "price": 1299.99,
    "inventory_count": 10
  }'

# Get all products
curl http://localhost:8001/products
```

**Open Jaeger UI:**

Navigate to http://localhost:16686

1. Select "product-service" from the Service dropdown
2. Click "Find Traces"
3. You should see traces for your API calls!
4. Click on a trace to see the span details:
   - HTTP request span
   - Repository method spans
   - SQL query timings

This is **distributed tracing in action**!

## Step 8: Understanding the Trace View

In Jaeger, each trace shows:
- **Trace ID**: Unique identifier for the request
- **Spans**: Timeline of operations
- **Service**: Which service handled each span
- **Duration**: How long each operation took
- **Tags**: Metadata (HTTP method, status code, etc.)
- **Logs**: Events within spans

This is exactly what you'll see in Instana when you connect it!

## Step 9: Add Seed Data (Optional)

Let's add some initial products automatically.

Create `product-service/migrations/<timestamp>_seed_products.sql`:

```bash
cd ~/code/rustmart/product-service
sqlx migrate add seed_products
```

Edit the new migration file:

```sql
-- Seed initial products
INSERT INTO products (id, name, price, inventory_count) VALUES
    ('550e8400-e29b-41d4-a716-446655440001', 'Laptop Pro', 1999.99, 25),
    ('550e8400-e29b-41d4-a716-446655440002', 'Wireless Mouse', 49.99, 100),
    ('550e8400-e29b-41d4-a716-446655440003', 'Mechanical Keyboard', 149.99, 50),
    ('550e8400-e29b-41d4-a716-446655440004', 'USB-C Hub', 79.99, 75),
    ('550e8400-e29b-41d4-a716-446655440005', '4K Monitor', 599.99, 30)
ON CONFLICT (id) DO NOTHING;
```

Restart the stack:
```bash
docker-compose down
docker-compose up --build
```

Now you'll have sample products on startup!

## Step 10: Managing the Stack

**Start in background:**
```bash
docker-compose up -d
```

**View logs:**
```bash
docker-compose logs -f product-service
```

**Stop everything:**
```bash
docker-compose down
```

**Stop and remove volumes (fresh database):**
```bash
docker-compose down -v
```

**Rebuild a specific service:**
```bash
docker-compose up --build product-service
```

## Docker Compose Commands Reference

| Command | What it does |
|---------|--------------|
| `docker-compose up` | Start all services |
| `docker-compose up -d` | Start in detached mode (background) |
| `docker-compose down` | Stop and remove containers |
| `docker-compose down -v` | Stop and remove volumes |
| `docker-compose ps` | List running services |
| `docker-compose logs -f <service>` | Follow logs for service |
| `docker-compose exec <service> <cmd>` | Run command in service |
| `docker-compose restart <service>` | Restart a service |
| `docker-compose build` | Rebuild images |

## Key Takeaways

1. **Multi-stage builds**: Smaller production images
2. **Health checks**: Services wait for dependencies
3. **Environment variables**: Configure services
4. **Docker networking**: Services communicate by name
5. **Jaeger UI**: Visualize traces locally
6. **Reproducible setup**: One command to run everything

## Challenges

1. **Add Redis**: Add a Redis container for caching
2. **Production Dockerfile**: Create a separate prod Dockerfile with optimizations
3. **Environment files**: Use `.env` file for Docker Compose variables
4. **Multi-service**: When you build other services (order-service, etc.), add them to docker-compose.yml

<details>
<summary>Challenge 1 Solution: Add Redis</summary>

Add to `docker-compose.yml`:

```yaml
  redis:
    image: redis:7-alpine
    container_name: rustmart-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

Then in product-service, add `redis` crate and implement caching layer!

</details>

<details>
<summary>Challenge 3 Solution: Environment File</summary>

Create `.env`:
```
POSTGRES_DB=rustmart
POSTGRES_USER=rustmart_user
POSTGRES_PASSWORD=rustmart_pass
RUST_LOG=info
```

Update `docker-compose.yml`:
```yaml
services:
  postgres:
    env_file: .env
    # ...
```

Don't forget to add `.env` to `.gitignore`!

</details>

## Troubleshooting

**"port is already allocated"**: Another service using that port, stop it or change port in docker-compose.yml

**"database connection refused"**: Wait for health check, or check DATABASE_URL

**"no space left on device"**: Clean up Docker: `docker system prune -a --volumes`

**Traces not showing in Jaeger**: Check OTLP endpoint, verify Jaeger is healthy

## Connecting to Instana

When you're ready to connect to Instana:

1. Get your Instana backend OTLP endpoint from IBM Cloud
2. Update `docker-compose.yml`:
   ```yaml
   product-service:
     environment:
       OTEL_EXPORTER_OTLP_ENDPOINT: https://your-instana-endpoint:4317
       OTEL_EXPORTER_OTLP_HEADERS: "authorization=Bearer YOUR_TOKEN"
   ```
3. Restart: `docker-compose up -d product-service`
4. Your traces now flow to Instana instead of local Jaeger!

## Next Steps

Now that you have containerized infrastructure, you're ready to:
- **Lesson 10**: Build the API Gateway service
- **Lesson 11**: Implement service-to-service communication
- **Lesson 12**: Add Kafka for async messaging

## Official Documentation

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)
- [Multi-stage builds](https://docs.docker.com/build/building/multi-stage/)

---

**Congrats!** You now have a fully observable, containerized microservice ready for your IBM demo! 🎉
