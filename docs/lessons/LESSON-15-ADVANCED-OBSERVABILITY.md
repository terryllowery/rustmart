# Lesson 15: Advanced Observability - Metrics, Dashboards, and Alerting

## Overview
You've instrumented tracing (Lesson 7). Now add the other pillars of observability: **metrics** and **logs**. Together with tracing, this gives you complete visibility into RustMart's health and performance.

By the end of this lesson, you'll have:
- Prometheus metrics exported from services
- Grafana dashboards for visualization
- Alerting rules for critical issues
- Structured logging with correlation IDs
- The complete observability stack for IBM Instana integration

## The Three Pillars of Observability

1. **Logs**: Discrete events ("User login failed")
2. **Metrics**: Numerical measurements over time (CPU usage, request rate)
3. **Traces**: Request flows through distributed systems

Together, they answer:
- **Logs**: What happened?
- **Metrics**: How much/how often?
- **Traces**: Where did it go?

## Step 1: Add Prometheus Metrics

Use `prometheus` crate to expose metrics.

Add to `product-service/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
prometheus = { version = "0.13", features = ["process"] }
lazy_static = "1.4"
```

Create `product-service/src/metrics.rs`:

```rust
use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, HistogramVec,
    IntCounterVec, IntGauge, TextEncoder, Encoder,
};

lazy_static! {
    // Request count by endpoint and status
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests",
        &["method", "endpoint", "status"]
    )
    .unwrap();

    // Request duration histogram
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latency in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0]
    )
    .unwrap();

    // Database connection pool size
    pub static ref DB_POOL_SIZE: IntGauge = register_int_gauge!(
        "db_pool_connections",
        "Number of database connections in pool"
    )
    .unwrap();

    // Business metrics
    pub static ref PRODUCTS_CREATED: IntCounterVec = register_int_counter_vec!(
        "products_created_total",
        "Total products created",
        &["status"]
    )
    .unwrap();
}

pub fn encode_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

## Step 2: Metrics Middleware

Create middleware to automatically track HTTP metrics.

Create `product-service/src/middleware/metrics.rs`:

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;

use crate::metrics::{HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION};

pub async fn track_metrics(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let timer = Instant::now();

    let response = next.run(request).await;

    let duration = timer.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Record metrics
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path])
        .observe(duration);

    response
}
```

## Step 3: Expose Metrics Endpoint

Add to `product-service/src/lib.rs`:

```rust
mod metrics;
mod middleware;

async fn metrics_handler() -> impl IntoResponse {
    metrics::encode_metrics()
}

pub fn create_router(pool: PgPool, kafka: KafkaProducer) -> Router {
    // ... existing setup ...

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))  // Prometheus endpoint
        .route("/products", get(get_products).post(create_product))
        .route("/products/:id", get(get_product_by_id).delete(delete_product))
        .layer(middleware::from_fn(middleware::metrics::track_metrics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

Test metrics:
```bash
curl http://localhost:8001/metrics
```

You'll see Prometheus format metrics:
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",endpoint="/products",status="200"} 42
```

## Step 4: Business Metrics

Track domain-specific metrics in handlers:

```rust
use crate::metrics::PRODUCTS_CREATED;

#[tracing::instrument(skip(state))]
async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.create(req).await?;

    // Track business metric
    PRODUCTS_CREATED.with_label_values(&["success"]).inc();

    // ... publish event ...

    Ok((axum::http::StatusCode::CREATED, Json(product)))
}
```

## Step 5: Deploy Prometheus

Create `k8s/base/prometheus.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s

    scrape_configs:
      - job_name: 'product-service'
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - rustmart
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            action: keep
            regex: product-service
          - source_labels: [__meta_kubernetes_pod_ip]
            target_label: __address__
            replacement: $1:8001
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prometheus
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:latest
        ports:
        - containerPort: 9090
        volumeMounts:
        - name: config
          mountPath: /etc/prometheus
        args:
          - '--config.file=/etc/prometheus/prometheus.yml'
          - '--storage.tsdb.path=/prometheus'
      volumes:
      - name: config
        configMap:
          name: prometheus-config
---
apiVersion: v1
kind: Service
metadata:
  name: prometheus
spec:
  selector:
    app: prometheus
  ports:
  - port: 9090
    targetPort: 9090
```

Deploy:
```bash
kubectl apply -f k8s/base/prometheus.yaml -n rustmart
kubectl port-forward service/prometheus 9090:9090 -n rustmart
```

Open http://localhost:9090

## Step 6: Deploy Grafana

Create `k8s/base/grafana.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: grafana
spec:
  replicas: 1
  selector:
    matchLabels:
      app: grafana
  template:
    metadata:
      labels:
        app: grafana
    spec:
      containers:
      - name: grafana
        image: grafana/grafana:latest
        ports:
        - containerPort: 3000
        env:
        - name: GF_SECURITY_ADMIN_PASSWORD
          value: admin
---
apiVersion: v1
kind: Service
metadata:
  name: grafana
spec:
  selector:
    app: grafana
  ports:
  - port: 3000
    targetPort: 3000
```

Deploy:
```bash
kubectl apply -f k8s/base/grafana.yaml -n rustmart
kubectl port-forward service/grafana 3000:3000 -n rustmart
```

Open http://localhost:3000 (admin/admin)

## Step 7: Create Grafana Dashboard

1. Add Prometheus data source:
   - Configuration → Data Sources → Add Prometheus
   - URL: `http://prometheus:9090`

2. Create dashboard:
   - Create → Dashboard → Add Panel

### Key Panels

**Request Rate:**
```promql
rate(http_requests_total[5m])
```

**Error Rate:**
```promql
rate(http_requests_total{status=~"5.."}[5m])
```

**P95 Latency:**
```promql
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

**Products Created (Business Metric):**
```promql
rate(products_created_total[5m])
```

Save dashboard as "RustMart Product Service".

## Step 8: Alerting Rules

Create `k8s/base/prometheus-rules.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-rules
data:
  alerts.yml: |
    groups:
      - name: rustmart-alerts
        interval: 30s
        rules:
          - alert: HighErrorRate
            expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
            for: 5m
            labels:
              severity: critical
            annotations:
              summary: "High error rate on {{ $labels.endpoint }}"
              description: "Error rate is {{ $value }} req/s"

          - alert: HighLatency
            expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1.0
            for: 5m
            labels:
              severity: warning
            annotations:
              summary: "High latency on {{ $labels.endpoint }}"
              description: "P95 latency is {{ $value }}s"

          - alert: ServiceDown
            expr: up{job="product-service"} == 0
            for: 1m
            labels:
              severity: critical
            annotations:
              summary: "Product service is down"

          - alert: DatabaseConnectionPoolLow
            expr: db_pool_connections < 2
            for: 5m
            labels:
              severity: warning
            annotations:
              summary: "Low database connection pool"
```

## Step 9: Structured Logging with Correlation IDs

Add correlation IDs to trace requests across logs, metrics, and traces.

Create `shared/src/correlation.rs`:

```rust
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_header(value: &str) -> Self {
        Self(value.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Middleware to extract/generate correlation ID:

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id = request
        .headers()
        .get("X-Correlation-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| CorrelationId::from_header(s))
        .unwrap_or_else(CorrelationId::new);

    // Store in request extensions
    request.extensions_mut().insert(correlation_id.clone());

    // Add to tracing span
    tracing::Span::current().record("correlation_id", correlation_id.as_str());

    let mut response = next.run(request).await;

    // Add to response headers
    response.headers_mut().insert(
        "X-Correlation-ID",
        correlation_id.as_str().parse().unwrap(),
    );

    response
}
```

Use in handlers:

```rust
#[tracing::instrument(skip(state), fields(correlation_id))]
async fn get_products(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Fetching all products");
    // ... 
}
```

Now logs, traces, and metrics all share the same correlation ID!

## Step 10: Connecting to Instana

For your IBM demo, configure Instana:

1. **Get Instana agent key** from IBM Cloud

2. **Deploy Instana agent** on Kubernetes:

```bash
kubectl create namespace instana-agent
kubectl create secret generic instana-agent-secret \
  --from-literal=key=YOUR_INSTANA_KEY \
  -n instana-agent

kubectl apply -f https://github.com/instana/instana-agent-operator/releases/latest/download/instana-agent-operator.yaml

kubectl apply -f - <<EOF
apiVersion: instana.io/v1
kind: InstanaAgent
metadata:
  name: instana-agent
  namespace: instana-agent
spec:
  agent:
    key: YOUR_INSTANA_KEY
    endpointHost: ingress-red-saas.instana.io
    endpointPort: "443"
  zone:
    name: rustmart-production
  cluster:
    name: rustmart
EOF
```

3. **Update services** to export to Instana:

```yaml
env:
- name: OTEL_EXPORTER_OTLP_ENDPOINT
  value: "http://instana-agent.instana-agent:4317"
```

Instana will automatically:
- Discover all services
- Map service dependencies
- Collect traces, metrics, logs
- Provide AI-powered anomaly detection
- Alert on issues

## The Complete Observability Stack

```
┌─────────────────────────────────────────────────────┐
│                    Instana (IBM)                    │
│  (Production monitoring, alerting, AI analytics)    │
└─────────────────────────────────────────────────────┘
                          ▲
                          │ OTLP/Prometheus
                          │
┌─────────────────────────────────────────────────────┐
│              RustMart Microservices                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ Product  │  │   API    │  │  Order   │         │
│  │ Service  │  │ Gateway  │  │ Service  │         │
│  └──────────┘  └──────────┘  └──────────┘         │
│       │             │              │                │
│       ├─ Traces (OpenTelemetry)───┘                │
│       ├─ Metrics (Prometheus) ─────────────┐       │
│       └─ Logs (structured JSON) ──────┐    │       │
└───────────────────────────────────────│────│───────┘
                                        │    │
                        ┌───────────────┘    │
                        ▼                    ▼
                  ┌──────────┐        ┌──────────┐
                  │  Jaeger  │        │Prometheus│
                  │   (dev)  │        │ Grafana  │
                  └──────────┘        └──────────┘
```

## Golden Signals

Monitor these four critical metrics:

1. **Latency**: How long requests take
   - P50, P95, P99 response times
   - Alert if P95 > 1 second

2. **Traffic**: Request volume
   - Requests per second
   - Alert if sudden drop (service down)

3. **Errors**: Failure rate
   - 4xx and 5xx responses
   - Alert if error rate > 5%

4. **Saturation**: Resource usage
   - CPU, memory, disk
   - Alert if CPU > 80%

## Key Takeaways

1. **Three pillars**: Logs + Metrics + Traces = Complete observability
2. **Golden signals**: Latency, traffic, errors, saturation
3. **Correlation IDs**: Connect data across systems
4. **Business metrics**: Track domain events, not just infra
5. **Alerting**: Proactive detection before users complain
6. **Instana integration**: Enterprise-grade observability for IBM demo

## Production Readiness Checklist

- [x] OpenTelemetry tracing
- [x] Prometheus metrics
- [x] Structured logging
- [x] Health checks
- [x] Readiness probes
- [x] Resource limits
- [x] Auto-scaling (HPA)
- [x] Alerting rules
- [x] Grafana dashboards
- [x] Correlation IDs
- [x] Circuit breakers
- [x] Retries and timeouts
- [x] Database migrations
- [x] Secrets management
- [x] CI/CD pipeline
- [x] Documentation

**Congratulations!** RustMart is production-ready and demo-ready for IBM/Instana! 🎉

## Challenges

1. **Add custom Grafana dashboards** for each service
2. **Implement distributed tracing sampling** (trace 10% of requests)
3. **Add log aggregation** with Loki or ELK stack
4. **Create SLO dashboards** (Service Level Objectives)
5. **Add cost tracking** metrics for cloud resources

## Next Steps

You've completed all 15 lessons! Here's what to do next:

1. **Build out other services**: order-service, inventory-service, payment-service
2. **Add authentication service**: OAuth2/OIDC with Keycloak
3. **Implement CQRS**: Separate read and write models
4. **Add caching layer**: Redis for hot data
5. **Performance optimization**: Profiling with flamegraphs
6. **Security hardening**: OWASP Top 10, penetration testing
7. **Deploy to production**: AWS EKS, GCP GKE, or Azure AKS

## Official Documentation

- [Prometheus](https://prometheus.io/docs/)
- [Grafana](https://grafana.com/docs/)
- [OpenTelemetry](https://opentelemetry.io/docs/)
- [Instana](https://www.ibm.com/docs/en/instana-observability)
- [SRE Book (Google)](https://sre.google/books/)
- [Observability Engineering](https://www.honeycomb.io/observability-engineering-book)

---

**You did it!** You've built a production-grade Rust microservices platform with complete observability. This positions you perfectly for:
- Band 10 promotion at IBM
- Rust microservices expert role
- Cloud-native architecture leadership
- Instana demo that showcases real-world distributed tracing

Keep building, keep learning, and good luck with your career goals! 🚀
