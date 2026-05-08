# Lesson 7: OpenTelemetry Basics - Tracing Your Microservice

## Overview
In this lesson, you'll add **OpenTelemetry** (OTel) tracing to the product-service. This is **critical** for your IBM demo because it lets you hook RustMart into **Instana** for observability.

By the end of this lesson, you'll have:
- OpenTelemetry SDK configured in product-service
- Automatic HTTP request tracing
- Custom spans for business logic
- Traces exported to console (and ready for Instana)

## What is OpenTelemetry?

OpenTelemetry is an **observability framework** that provides:
- **Traces**: Request flows through your system
- **Metrics**: Performance measurements
- **Logs**: Structured event records

For microservices, tracing is essential because a single user request can touch multiple services. OTel traces show you the complete journey.

## Why This Matters for IBM/Instana

Instana is IBM's **automatic application performance monitoring (APM)** tool. It natively supports OpenTelemetry, so by instrumenting RustMart with OTel:
- Instana automatically discovers your services
- You get distributed tracing across all microservices
- You can demo real-world observability to your team
- This positions you as a Rust + Cloud-Native expert at IBM

## Key Concepts

### Traces and Spans
- **Trace**: The complete journey of a request through your system
- **Span**: A single operation within that trace (e.g., "handle GET /products")
- Spans have: name, start time, duration, attributes, parent span

### Instrumentation
- **Automatic**: Frameworks add spans for you (HTTP requests, DB queries)
- **Manual**: You create custom spans for business logic

### Exporters
- Console: Print traces to stdout (development)
- OTLP: Send traces to collectors like Instana, Jaeger, or Tempo
- We'll start with console, then you can switch to OTLP for Instana

## Step 1: Add OpenTelemetry Dependencies to Workspace

**Note**: `cargo add` doesn't have a direct way to add workspace dependencies. You need to manually edit the `Cargo.toml` files.

### Step 1a: Update Workspace Dependencies

Edit the **workspace** `Cargo.toml` at the project root (`~/code/rustmart/Cargo.toml`):

```toml
[workspace.dependencies]
# ... existing dependencies ...

# Observability (update existing entries)
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
opentelemetry-stdout = { version = "0.3", features = ["trace"] }
opentelemetry-jaeger = { version = "0.21", features = ["rt-tokio"] }
tracing-opentelemetry = "0.23"
```

**Why workspace dependencies?**
- All services share the same versions (avoid version conflicts)
- Easier to upgrade dependencies across the project
- Smaller disk usage (shared build artifacts)
- Standard practice for Cargo workspaces

### Step 1b: Update Service Dependencies

Then, update `product-service/Cargo.toml` (`~/code/rustmart/product-service/Cargo.toml`):

```toml
[dependencies]
shared = { path = "../shared" }
axum.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
tower-http.workspace = true

# OpenTelemetry - all from workspace
opentelemetry.workspace = true
opentelemetry_sdk.workspace = true
opentelemetry-otlp.workspace = true
opentelemetry-stdout.workspace = true
opentelemetry-jaeger.workspace = true
tracing-opentelemetry.workspace = true
```

**What these do:**
- `opentelemetry`: Core OTel API
- `opentelemetry_sdk`: Implementation with runtime support
- `opentelemetry-otlp`: OTLP exporter (Jaeger, Instana)
- `opentelemetry-stdout`: Console exporter for debugging
- `opentelemetry-jaeger`: Jaeger Thrift protocol (for Instana Option A)
- `tracing-opentelemetry`: Bridge between Rust's `tracing` crate and OTel

## Step 2: Initialize OpenTelemetry with Configurable Backend

We'll support three tracing backends via the `TRACING_BACKEND` environment variable:
- `console`: Print traces to stdout (development)
- `jaeger`: Send to Jaeger (with Prometheus/Grafana)
- `instana`: Send to Instana backend

Replace your `product-service/src/main.rs` with this:

```rust
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _; // Trait for .tracer() method
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig; // Trait for .with_endpoint()
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // Get backend configuration
    let backend = std::env::var("TRACING_BACKEND")
        .unwrap_or_else(|_| "console".to_string());

    // Initialize tracing based on backend (case-insensitive)
    match backend.as_str() {
        "jaeger" | "JAEGER" | "Jaeger" => {
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(
                            std::env::var("JAEGER_ENDPOINT")
                                .unwrap_or_else(|_| "http://localhost:4317".to_string())
                        )
                )
                .with_trace_config(
                    opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "product-service"),
                        KeyValue::new("service.version", "0.1.0"),
                    ]))
                )
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .expect("Failed to initialize Jaeger tracer");

            // Initialize tracing subscriber with OpenTelemetry
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
                ))
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        }
        "console" | "CONSOLE" | "Console" | _ => {
            // Console with OpenTelemetry stdout exporter (prints JSON traces)
            let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .with_config(
                    opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "product-service"),
                        KeyValue::new("service.version", "0.1.0"),
                    ]))
                )
                .build();

            let tracer = tracer.tracer("product-service");

            // Initialize tracing subscriber with OpenTelemetry
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
                ))
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        }
    }

    tracing::info!(backend = %backend, "Starting product-service with OpenTelemetry");

    // Create the Axum router from lib.rs
    let app = product_service::create_router();

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .expect("Failed to bind to port 8001");
    
    tracing::info!("Server listening on http://127.0.0.1:8001");
    
    axum::serve(listener, app)
        .await
        .expect("Server error");

    // Shutdown tracer on exit
    global::shutdown_tracer_provider();
}
```

**What changed:**
1. **Configurable backend**: Use `TRACING_BACKEND` env var to switch exporters
2. **Match backend**: Supports `console`, `jaeger`, or `instana` backends
3. **OTLP exporters**: Both Jaeger and Instana use OTLP protocol with gRPC
4. **Configurable endpoints**: `JAEGER_ENDPOINT` and `INSTANA_ENDPOINT` env vars
5. **Batch exporter**: Uses Tokio runtime for async span batching
6. **Graceful shutdown**: Flushes all pending spans on exit

### ⚠️ Important: Connection Timeout Issue

**Problem**: If Jaeger/Instana isn't running, the OTLP exporter will **block trying to connect**, causing:
- Slow service startup (10-30 seconds)
- API request timeouts
- Service appears hung

**Why this happens**: `install_batch()` attempts to establish a gRPC connection immediately and retries on failure.

**Solutions**:

**Option 1: Only enable when backend is available** (Simplest)
```bash
# Use console when Jaeger isn't running
TRACING_BACKEND=console cargo run -p product-service

# Or omit the variable (defaults to console)
cargo run -p product-service
```

**Option 2: Add health check before enabling** (Production-ready)

Add this helper function before `main()`:

```rust
use std::time::Duration;
use std::net::TcpStream;

/// Quick TCP health check to see if endpoint is reachable
fn is_endpoint_reachable(endpoint: &str, timeout_ms: u64) -> bool {
    // Parse "http://localhost:4317" -> "localhost:4317"
    let addr = endpoint
        .replace("http://", "")
        .replace("https://", "");
    
    match addr.parse::<std::net::SocketAddr>() {
        Ok(socket_addr) => {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms))
                .is_ok()
        }
        Err(_) => false,
    }
}
```

Then modify the Jaeger backend case:

```rust
"jaeger" | "JAEGER" | "Jaeger" => {
    let endpoint = std::env::var("JAEGER_ENDPOINT")
        .unwrap_or_else(|| "http://localhost:4317".to_string());
    
    // Quick health check (500ms timeout)
    if is_endpoint_reachable(&endpoint, 500) {
        // Jaeger is reachable, proceed with OTLP
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            // ... rest of config
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Failed to initialize Jaeger tracer");
        
        tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
        
        tracing::info!("✓ Jaeger tracing enabled");
    } else {
        // Fall back to console logging
        eprintln!("⚠ Jaeger not reachable at {}, using console logging", endpoint);
        
        tracing_subscriber::fmt()
            .with_env_filter(
                std::env::var("RUST_LOG").unwrap_or_else(|| "info".into())
            )
            .init();
    }
}
```

**Benefits of Option 2**:
- ✅ No timeouts or hangs
- ✅ Service starts immediately
- ✅ Automatic fallback
- ✅ Works in dev and prod

**For production**: Always use Option 2 with health checks to ensure your service is resilient to observability backend failures.

## Step 3: Add Automatic HTTP Tracing

We need to add middleware that automatically creates spans for HTTP requests.

Update `product-service/src/lib.rs`:

```rust
use axum::{
    extract::Path,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use shared::models::Product;
use shared::error::ApiError;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/products", get(get_products))
        .route("/products/:id", get(get_product_by_id))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

async fn health_check() -> &'static str {
    "OK"
}

#[tracing::instrument]
async fn get_products() -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Fetching all products");
    
    let products = vec![
        Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            price: 999.99,
            inventory_count: 50,
        },
        Product {
            id: "2".to_string(),
            name: "Mouse".to_string(),
            price: 29.99,
            inventory_count: 200,
        },
    ];

    Ok(Json(products))
}

#[tracing::instrument]
async fn get_product_by_id(Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(product_id = %id, "Fetching product by ID");

    if id == "1" {
        let product = Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            price: 999.99,
            inventory_count: 50,
        };
        Ok(Json(product))
    } else {
        Err(ApiError::NotFound(format!("Product {} not found", id)))
    }
}
```

**New additions:**
1. **tower_http::trace::TraceLayer**: Axum middleware that auto-creates spans for HTTP requests
2. **#[tracing::instrument]**: Macro that creates a span for the function
3. **Structured logging**: `product_id = %id` adds attributes to spans

**Note**: `tower-http`, `opentelemetry-otlp`, and `opentelemetry-jaeger` are already included from Step 1's workspace dependencies, so no additional changes needed!

## Step 4: Test with Console Backend (Default)

Run the service with console output:

```bash
cd ~/code/rustmart
RUST_LOG=info TRACING_BACKEND=console cargo run -p product-service
```

In another terminal, make requests:

```bash
curl http://localhost:8001/health
curl http://localhost:8001/products
curl http://localhost:8001/products/1
curl http://localhost:8001/products/999
```

**What to look for in the output:**

You'll see JSON traces printed to stdout like:

```json
{
  "resourceSpans": [{
    "resource": {
      "attributes": [{
        "key": "service.name",
        "value": { "stringValue": "product-service" }
      }]
    },
    "scopeSpans": [{
      "spans": [{
        "name": "GET /products",
        "spanId": "...",
        "traceId": "...",
        "startTimeUnixNano": "...",
        "endTimeUnixNano": "...",
        "attributes": [...]
      }]
    }]
  }]
}
```

This proves OpenTelemetry is working with the console backend!

## Step 5: Set Up Jaeger with Docker Compose

Now let's run Jaeger, Prometheus, and Grafana locally so you can visualize traces.

Create `docker-compose.observability.yml` in your project root:

```yaml
version: '3.8'

services:
  # Jaeger all-in-one (includes collector, query, UI)
  jaeger:
    image: jaegertracing/all-in-one:1.52
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC receiver
      - "4318:4318"    # OTLP HTTP receiver
      - "14268:14268"  # Jaeger collector HTTP
    environment:
      - COLLECTOR_OTLP_ENABLED=true
      - METRICS_STORAGE_TYPE=prometheus
      - PROMETHEUS_SERVER_URL=http://prometheus:9090
    networks:
      - observability

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:v2.48.0
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - observability

  # Grafana for dashboards
  grafana:
    image: grafana/grafana:10.2.2
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-storage:/var/lib/grafana
      - ./config/grafana/datasources.yml:/etc/grafana/provisioning/datasources/datasources.yml
    networks:
      - observability

networks:
  observability:
    driver: bridge

volumes:
  grafana-storage:
```

Create `config/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'jaeger'
    static_configs:
      - targets: ['jaeger:14269']

  # Add your services here later
  - job_name: 'product-service'
    static_configs:
      - targets: ['host.docker.internal:8001']
```

Create `config/grafana/datasources.yml`:

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true

  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
    jsonData:
      tracesToLogs:
        datasourceUid: 'loki'
```

Start the observability stack:

```bash
docker-compose -f docker-compose.observability.yml up -d
```

Verify all services are running:

```bash
docker-compose -f docker-compose.observability.yml ps
```

Access the UIs:
- **Jaeger UI**: http://localhost:16686
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

## Step 6: Test with Jaeger Backend

Now run your service with Jaeger:

```bash
RUST_LOG=info TRACING_BACKEND=jaeger cargo run -p product-service
```

Make some requests:

```bash
curl http://localhost:8001/products
curl http://localhost:8001/products/1
curl http://localhost:8001/products/999
```

Open **Jaeger UI** at http://localhost:16686:
1. Select **product-service** from the Service dropdown
2. Click **Find Traces**
3. Click on a trace to see the full span tree

You'll see:
- HTTP request spans with method, path, status code
- Function spans from `#[tracing::instrument]`
- Nested span relationships (parent/child)
- Timing information for each operation

## Step 7: Understanding Trace IDs

Each request gets a unique **trace ID**. If you have multiple services, they all use the same trace ID to link spans together. This is how distributed tracing works.

When you call another service, you propagate the trace context via HTTP headers:
- `traceparent`: W3C standard header with trace ID and span ID
- Services automatically extract this and continue the trace

## Step 8: Set Up Instana Integration (Master Both Approaches)

To become **the go-to expert on Rust instrumentation with Instana at IBM**, you need to master **both integration paths**. Each has different use cases in enterprise environments.

### Option A: Direct Jaeger Protocol (Production-Ready, Simple)

Instana agents natively support the **Jaeger Thrift protocol**:

```
RustMart Service → Instana Agent (Jaeger endpoint) → Instana Backend
```

**When to use:**
- Greenfield Rust projects with Instana
- Simple microservice deployments
- Lower latency requirements
- Direct agent-to-service communication

### Option B: Via OpenTelemetry Collector (Enterprise-Grade, Flexible)

Using an intermediary OTel Collector:

```
RustMart Service → OTel Collector → Instana Agent → Instana Backend
```

**When to use:**
- Multi-vendor observability (Instana + Datadog + Splunk)
- Advanced sampling, filtering, or trace enrichment
- Complex enterprise pipelines with compliance requirements
- Need to correlate traces with logs (e.g., tail-based sampling)
- Migrating from another observability platform to Instana

---

**You'll implement BOTH** so you can recommend the right approach for any scenario at IBM.

### Update Rust Code for Both Options

**Note**: The Jaeger exporter was already added in Step 1 via workspace dependencies.

Update `init_tracer()` in `main.rs` to support Jaeger protocol for Instana:

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::{TracerProvider, Config};
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;

/// Initialize OpenTelemetry with configurable backend
fn init_tracer() -> TracerProvider {
    let backend = std::env::var("TRACING_BACKEND")
        .unwrap_or_else(|_| "console".to_string())
        .to_lowercase();

    let resource = Resource::new(vec![
        KeyValue::new("service.name", "product-service"),
        KeyValue::new("service.version", "0.1.0"),
    ]);

    let config = Config::default().with_resource(resource);

    match backend.as_str() {
        "jaeger" => {
            tracing::info!("Initializing Jaeger exporter");
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(
                    std::env::var("JAEGER_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:4317".to_string())
                );

            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(config)
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .expect("Failed to initialize Jaeger tracer")
        }
        "instana" | "instana-jaeger" => {
            tracing::info!("Initializing Instana exporter (Jaeger protocol)");
            
            // Instana agents expose Jaeger Thrift endpoint on port 6831 (UDP) or 14268 (HTTP)
            let agent_endpoint = std::env::var("INSTANA_AGENT_HOST")
                .unwrap_or_else(|_| "localhost".to_string());
            
            opentelemetry_jaeger::new_agent_pipeline()
                .with_service_name("product-service")
                .with_endpoint(format!("{}:6831", agent_endpoint))
                .with_trace_config(config)
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .expect("Failed to initialize Instana tracer")
        }
        "instana-otlp" => {
            tracing::info!("Initializing Instana exporter (via OTel Collector)");
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(
                    std::env::var("OTEL_COLLECTOR_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:4315".to_string())
                );

            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(config)
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .expect("Failed to initialize Instana OTLP tracer")
        }
        _ => {
            tracing::info!("Initializing console exporter");
            let exporter = opentelemetry_stdout::SpanExporter::default();
            TracerProvider::builder()
                .with_simple_exporter(exporter)
                .with_config(config)
                .build()
        }
    }
}
```

### Docker Compose with Optional OTel Collector

Update `docker-compose.observability.yml`:

```yaml
version: '3.8'

services:
  # Jaeger all-in-one (for local development)
  jaeger:
    image: jaegertracing/all-in-one:1.52
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC receiver
      - "4318:4318"    # OTLP HTTP receiver
    environment:
      - COLLECTOR_OTLP_ENABLED=true
      - METRICS_STORAGE_TYPE=prometheus
      - PROMETHEUS_SERVER_URL=http://prometheus:9090
    networks:
      - observability

  # OpenTelemetry Collector (for Instana integration)
  otel-collector:
    image: otel/opentelemetry-collector-contrib:0.91.0
    ports:
      - "4315:4317"    # OTLP gRPC (different port to avoid conflict with Jaeger)
      - "4316:4318"    # OTLP HTTP
      - "8888:8888"    # Prometheus metrics
      - "13133:13133"  # Health check
    volumes:
      - ./config/otel-collector-config.yaml:/etc/otel-collector-config.yaml
    command: ["--config=/etc/otel-collector-config.yaml"]
    networks:
      - observability

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:v2.48.0
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - observability

  # Grafana for dashboards
  grafana:
    image: grafana/grafana:10.2.2
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-storage:/var/lib/grafana
      - ./config/grafana/datasources.yml:/etc/grafana/provisioning/datasources/datasources.yml
    networks:
      - observability

networks:
  observability:
    driver: bridge

volumes:
  grafana-storage:
```

### OTel Collector Config (Optional - for Option B)

If using `TRACING_BACKEND=instana-otlp`, create `config/otel-collector-config.yaml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024
  
  # Add resource attributes for Instana
  resource:
    attributes:
      - key: instana.zone
        value: "rustmart-dev"
        action: upsert

  # Memory limiter to prevent OOM
  memory_limiter:
    check_interval: 1s
    limit_mib: 512

exporters:
  # Debug exporter (console output)
  logging:
    loglevel: info
  
  # Instana exporter via OTLP
  otlp/instana:
    endpoint: "${INSTANA_AGENT_HOST}:4317"
    tls:
      insecure: true
    headers:
      # Instana agent key if required
      x-instana-key: "${INSTANA_AGENT_KEY}"
  
  # Also export to Jaeger for local testing
  otlp/jaeger:
    endpoint: "jaeger:4317"
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [logging, otlp/instana]
    
    # Optional: also send to Jaeger for local visualization
    # traces/jaeger:
    #   receivers: [otlp]
    #   processors: [batch]
    #   exporters: [otlp/jaeger]
  
  telemetry:
    logs:
      level: info
    metrics:
      address: 0.0.0.0:8888
```

### Run with Instana - Option A (Jaeger Protocol - Recommended)

**Simplest approach** - Instana agent already has Jaeger support:

```bash
# Run your service pointing directly to Instana agent
RUST_LOG=info TRACING_BACKEND=instana INSTANA_AGENT_HOST=your-instana-agent cargo run -p product-service
```

The Instana agent exposes Jaeger endpoints:
- **UDP Thrift**: Port `6831` (default, used by the Jaeger agent protocol)
- **HTTP Thrift**: Port `14268` (Jaeger collector HTTP)

Traces flow: `product-service → Instana Agent (Jaeger endpoint) → Instana Backend`

### Run with Instana - Option B (via OTel Collector)

For advanced use cases (filtering, routing, dual-export):

```bash
# Set Instana agent details
export INSTANA_AGENT_HOST=your-instana-agent-host
export INSTANA_AGENT_KEY=your-instana-key

# Start observability stack
docker-compose -f docker-compose.observability.yml up -d

# Run service pointing to OTel Collector
RUST_LOG=info TRACING_BACKEND=instana-otlp OTEL_COLLECTOR_ENDPOINT=http://localhost:4315 cargo run -p product-service
```

Traces flow: `product-service → OTel Collector → Instana Agent → Instana Backend`

### Verify Instana Integration

1. **Check traces are flowing**:
   ```bash
   # Make requests
   curl http://localhost:8001/products
   curl http://localhost:8001/products/1
   ```

2. **View in Instana UI**:
   - Navigate to Applications → Services
   - Find `product-service`
   - Click on a trace to see the full span tree

3. **Instana automatically provides**:
   - Service discovery and dependency maps
   - Distributed tracing across all microservices
   - Performance anomaly detection
   - Infrastructure correlation

### Decision Matrix: Which Approach for Which Scenario?

| Feature | Option A: Jaeger Protocol | Option B: OTel Collector |
|---------|--------------------------|-------------------------|
| **Simplicity** | ✅ Simple, direct connection | ⚠️ Requires collector setup |
| **Latency** | ✅ Lower (one less hop) | ⚠️ Slightly higher |
| **Setup** | ✅ Just point to agent | ⚠️ Need collector config |
| **Filtering/Sampling** | ❌ Limited | ✅ Advanced processors |
| **Multi-backend** | ❌ Single destination | ✅ Fan-out to multiple |
| **Resource enrichment** | ⚠️ Basic | ✅ Rich attribute manipulation |
| **PII redaction** | ❌ None | ✅ Processor-based scrubbing |
| **Cost control** | ⚠️ Agent-level only | ✅ Sampling before export |
| **Best for** | Startups, single vendor | Enterprises, multi-vendor |

### Real-World IBM Scenarios:

**Scenario 1: New Rust microservice in existing Instana deployment**
→ **Use Option A** - Fast to implement, agent already deployed

**Scenario 2: Multi-cloud deployment with Instana + CloudWatch**
→ **Use Option B** - OTel Collector fans out to both backends

**Scenario 3: High-throughput service (1M+ req/min) with cost constraints**
→ **Use Option B** - Tail-based sampling keeps only interesting traces

**Scenario 4: Regulated industry with PII in traces**
→ **Use Option B** - Collector processors redact sensitive data before export

**Scenario 5: Demo/POC for IBM client**
→ **Show both!** - Demonstrate flexibility and deep expertise

### Your Learning Path:

1. ✅ **Today**: Implement both options in RustMart
2. 🎯 **Demo prep**: Practice switching between them on-the-fly
3. 🚀 **IBM presentation**: Show decision matrix to architects
4. 💪 **Positioning**: "I know when to use each approach and why"

### Hands-On Exercise: Master Both Approaches

**Challenge**: Configure product-service to work with BOTH Instana integration methods and demonstrate switching between them.

**Step 1: Test Option A (Jaeger Protocol)**
```bash
# Terminal 1: Simulate Instana agent with Jaeger endpoint
# (In production, this would be your actual Instana agent)
docker run -d --name instana-agent-sim \
  -p 6831:6831/udp \
  -p 14268:14268 \
  jaegertracing/all-in-one:1.52

# Terminal 2: Run your service with Jaeger protocol
RUST_LOG=info TRACING_BACKEND=instana INSTANA_AGENT_HOST=localhost cargo run -p product-service

# Terminal 3: Generate traces
curl http://localhost:8001/products
curl http://localhost:8001/products/1

# View traces at http://localhost:16686
```

**Step 2: Test Option B (OTel Collector)**
```bash
# Terminal 1: Start full observability stack with OTel Collector
export INSTANA_AGENT_HOST=localhost
export INSTANA_AGENT_KEY=test-key
docker-compose -f docker-compose.observability.yml up -d

# Terminal 2: Run service pointing to OTel Collector
RUST_LOG=info TRACING_BACKEND=instana-otlp OTEL_COLLECTOR_ENDPOINT=http://localhost:4315 cargo run -p product-service

# Terminal 3: Generate traces
curl http://localhost:8001/products

# Check OTel Collector logs
docker logs otel-collector
```

**Step 3: Compare the Approaches**

Create a comparison table in your notes:

| Metric | Option A | Option B |
|--------|----------|----------|
| Time to first trace | _____ | _____ |
| Startup time | _____ | _____ |
| Lines of config | _____ | _____ |
| Dependencies | _____ | _____ |
| Your preference | _____ | _____ |

**Bonus**: Add a third backend option (`TRACING_BACKEND=instana-http`) that uses the HTTP Thrift endpoint (port 14268) instead of UDP.

## Step 9: Configure All Services

As you add more services (order-service, user-service), use the same pattern:

1. Add OpenTelemetry dependencies
2. Copy the `init_tracer()` function
3. Change `service.name` to match the service
4. Run with `TRACING_BACKEND=jaeger` or `TRACING_BACKEND=instana`

All services will share the same trace IDs, giving you **distributed tracing** across your entire system.

## Key Takeaways

### Technical Skills Acquired:
1. **Multi-backend architecture**: Switch between Jaeger, Instana (2 methods), or console with env vars
2. **Jaeger Thrift protocol**: Direct Instana agent integration for simplicity
3. **OTel Collector pipeline**: Enterprise-grade trace processing and routing
4. **Docker Compose observability stack**: Jaeger + Prometheus + Grafana + OTel Collector
5. **Automatic instrumentation**: Middleware handles HTTP spans automatically
6. **Manual spans**: Use `#[tracing::instrument]` for business logic
7. **Distributed tracing**: W3C trace context propagation across microservices
8. **Production-ready patterns**: Batch exporters, resource attributes, graceful shutdown

### Your IBM/Instana Expertise Positioning:

💪 **"I know both Instana integration approaches for Rust and when to use each"**

- ✅ **Jaeger Protocol**: Fast setup, low latency, production-ready for single-vendor
- ✅ **OTel Collector**: Enterprise patterns, multi-vendor, compliance-ready
- ✅ **Decision framework**: Can advise architects on the right approach
- ✅ **Hands-on experience**: Implemented both in RustMart demo project
- ✅ **Cost awareness**: Understand sampling strategies and their trade-offs

**Your value proposition**: "Most developers only know one way to integrate Instana. I know both, understand the trade-offs, and can architect the right solution for each scenario."

### Band 10 Demonstration Points:

1. 🎯 **Technical depth**: Implemented dual integration paths with environment-based switching
2. 🛠️ **Enterprise thinking**: Considered PII redaction, cost control, multi-vendor scenarios
3. 📊 **Decision making**: Built comparison matrix for different use cases
4. 🚀 **Innovation**: Rust + Instana is cutting-edge (not many have done this)
5. 📚 **Knowledge sharing**: Can teach others via RustMart lessons

## Challenges

1. **Add a custom span**: Create a function that "processes" a product (e.g., calculates discount) and instrument it with a span
2. **Add span attributes**: In `get_product_by_id`, add the product name as a span attribute
3. **Simulate an error**: Add a span that records an error event when product not found
4. **Nested spans**: Create a function that calls another instrumented function and observe the parent-child relationship

<details>
<summary>Challenge 1 Solution: Custom Span</summary>

```rust
#[tracing::instrument]
async fn calculate_discount(product: &Product) -> f64 {
    tracing::info!("Calculating discount for product");
    
    // Simulate some business logic
    if product.price > 500.0 {
        product.price * 0.10 // 10% discount
    } else {
        0.0
    }
}

#[tracing::instrument]
async fn get_product_by_id(Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(product_id = %id, "Fetching product by ID");

    if id == "1" {
        let product = Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            price: 999.99,
            inventory_count: 50,
        };
        
        let discount = calculate_discount(&product).await;
        tracing::info!(discount = %discount, "Discount calculated");
        
        Ok(Json(product))
    } else {
        Err(ApiError::NotFound(format!("Product {} not found", id)))
    }
}
```

You'll see nested spans: `get_product_by_id` contains `calculate_discount`.

</details>

<details>
<summary>Challenge 2 Solution: Span Attributes</summary>

```rust
#[tracing::instrument(fields(product_name))]
async fn get_product_by_id(Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    tracing::info!(product_id = %id, "Fetching product by ID");

    if id == "1" {
        let product = Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            price: 999.99,
            inventory_count: 50,
        };
        
        // Record attribute on the span
        tracing::Span::current().record("product_name", &product.name);
        
        Ok(Json(product))
    } else {
        Err(ApiError::NotFound(format!("Product {} not found", id)))
    }
}
```

The span will have `product_name: "Laptop"` as an attribute.

</details>

## Next Steps

In **Lesson 8**, you'll integrate PostgreSQL with SQLx, and you'll see how database queries automatically create spans when instrumented!

After that, **Lesson 9** covers Docker Compose, where you'll run:
- PostgreSQL
- Your microservices
- Jaeger UI for viewing traces (before moving to Instana)

## Official Documentation

- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry/)
- [Instana OpenTelemetry](https://www.ibm.com/docs/en/instana-observability/current?topic=apis-opentelemetry)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)

---

**Important for IBM Demo**: Once you have Docker Compose running (Lesson 9), you can point the OTLP exporter at your Instana backend. The service.name will show up automatically in Instana, and you'll have full distributed tracing across all RustMart services!
