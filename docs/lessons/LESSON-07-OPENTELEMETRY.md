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

## Step 1: Add OpenTelemetry Dependencies

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

# OpenTelemetry dependencies
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-stdout = { version = "0.3", features = ["trace"] }
tracing-opentelemetry = "0.23"
```

**What these do:**
- `opentelemetry`: Core OTel API
- `opentelemetry_sdk`: Implementation with runtime support
- `opentelemetry-stdout`: Exporter that prints traces to console
- `tracing-opentelemetry`: Bridge between Rust's `tracing` crate and OTel

## Step 2: Initialize OpenTelemetry in main.rs

Replace your `product-service/src/main.rs` with this:

```rust
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(provider.tracer("product-service"));

    // Initialize tracing with both console and OpenTelemetry layers
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    tracing::info!("Starting product-service with OpenTelemetry...");

    // Create the Axum router from lib.rs
    let app = product_service::create_router();

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();
    
    tracing::info!("Server listening on http://127.0.0.1:8001");
    
    axum::serve(listener, app).await.unwrap();

    // Shutdown tracer on exit
    global::shutdown_tracer_provider();
}
```

**What changed:**
1. **TracerProvider**: Configures OTel SDK with service name and exporter
2. **Resource attributes**: Identify your service (service.name, version)
3. **tracing_opentelemetry layer**: Connects tracing macros to OTel
4. **Layered subscriber**: Combines console logs + OTel traces
5. **global::shutdown_tracer_provider()**: Flushes traces on exit

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

You need to add tower-http to `product-service/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
tower-http = { version = "0.5", features = ["trace"] }
```

## Step 4: Test Your Instrumentation

Run the service:

```bash
cd ~/code/rustmart
RUST_LOG=info cargo run -p product-service
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

This proves OpenTelemetry is working!

## Step 5: Understanding Trace IDs

Each request gets a unique **trace ID**. If you have multiple services, they all use the same trace ID to link spans together. This is how distributed tracing works.

When you call another service, you propagate the trace context via HTTP headers:
- `traceparent`: W3C standard header with trace ID and span ID
- Services automatically extract this and continue the trace

## Step 6: Preparing for Instana Integration

To send traces to Instana instead of stdout, you'll change the exporter to OTLP.

Add to `product-service/Cargo.toml`:

```toml
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
```

Then in `main.rs`, replace `opentelemetry_stdout::SpanExporter` with:

```rust
use opentelemetry_otlp::WithExportConfig;

let exporter = opentelemetry_otlp::new_exporter()
    .tonic()
    .with_endpoint("http://localhost:4317"); // Instana collector endpoint

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(exporter)
    .with_trace_config(
        opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
            KeyValue::new("service.name", "product-service"),
            KeyValue::new("service.version", "0.1.0"),
        ])),
    )
    .install_batch(opentelemetry_sdk::runtime::Tokio)
    .expect("Failed to initialize OTLP tracer");
```

**Note:** Don't do this yet! We'll set up Docker Compose in Lesson 9 with a full observability stack. For now, stdout is perfect.

## Key Takeaways

1. **OpenTelemetry is vendor-neutral**: Works with Instana, Jaeger, Datadog, etc.
2. **tracing crate integrates seamlessly**: Your existing `tracing::info!` logs become span events
3. **Automatic instrumentation**: Middleware handles HTTP spans for you
4. **Manual spans**: Use `#[tracing::instrument]` for custom spans
5. **Trace context propagation**: HTTP headers carry trace IDs between services

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
