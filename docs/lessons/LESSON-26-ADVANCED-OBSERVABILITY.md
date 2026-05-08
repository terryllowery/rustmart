# Lesson 26: Advanced Observability

## Overview
Implement sophisticated observability patterns including custom metrics, SLOs/SLIs, distributed tracing correlation, and advanced Instana integration for production-grade monitoring of RustMart.

## Why This Matters
Advanced observability enables:
- **Proactive Problem Detection** - Find issues before customers report them
- **Data-Driven SLOs** - Set and track service level objectives
- **Faster Incident Response** - Correlate metrics, logs, and traces
- **Business Insights** - Track business metrics alongside technical metrics

Essential for SRE practices and demonstrates production readiness for IBM Tiger Team work.

## Service Level Objectives (SLOs) & SLIs

### Defining SLIs (Service Level Indicators)

**Key SLIs for RustMart**:
```rust
// Request success rate
let success_rate = successful_requests / total_requests;

// Request latency percentiles
let p50_latency = calculate_percentile(&latencies, 0.50);
let p95_latency = calculate_percentile(&latencies, 0.95);
let p99_latency = calculate_percentile(&latencies, 0.99);

// Availability
let availability = uptime_seconds / total_seconds;

// Throughput
let throughput = requests_per_second;
```

### Setting SLOs

**Example SLOs**:
```yaml
product_service_slos:
  availability:
    target: 99.9%  # "three nines"
    window: 30d
  
  latency_p95:
    target: 200ms
    window: 7d
  
  error_rate:
    target: 0.1%   # <0.1% errors
    window: 30d
```

### Error Budget Calculation

```rust
use chrono::{DateTime, Duration, Utc};

#[derive(Debug)]
struct ErrorBudget {
    slo_target: f64,        // e.g., 0.999 for 99.9%
    window_duration: Duration,
    total_requests: u64,
    failed_requests: u64,
}

impl ErrorBudget {
    fn availability(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        let successful = self.total_requests - self.failed_requests;
        successful as f64 / self.total_requests as f64
    }
    
    fn budget_remaining(&self) -> f64 {
        let actual_availability = self.availability();
        let allowed_failure_rate = 1.0 - self.slo_target;
        let actual_failure_rate = 1.0 - actual_availability;
        
        // % of budget remaining
        ((allowed_failure_rate - actual_failure_rate) / allowed_failure_rate) * 100.0
    }
    
    fn is_violated(&self) -> bool {
        self.availability() < self.slo_target
    }
}

// Usage
async fn check_error_budget(prometheus: &PrometheusClient) -> Result<ErrorBudget, Error> {
    let query = r#"
        sum(rate(http_requests_total{status=~"2.."}[30d])) / 
        sum(rate(http_requests_total[30d]))
    "#;
    
    let availability = prometheus.query(query).await?;
    
    Ok(ErrorBudget {
        slo_target: 0.999,
        window_duration: Duration::days(30),
        total_requests: 1_000_000,
        failed_requests: 500,  // Calculated from Prometheus
    })
}
```

### Burn Rate Alerts

```yaml
# alerting_rules.yml
groups:
  - name: slo_burn_rate
    rules:
      # Fast burn: 2% budget consumed in 1 hour
      - alert: ErrorBudgetFastBurn
        expr: |
          (
            1 - (sum(rate(http_requests_total{status=~"2.."}[1h])) /
                 sum(rate(http_requests_total[1h])))
          ) > (14.4 * (1 - 0.999))
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Fast burn rate detected - error budget exhausted in <6 hours"
      
      # Slow burn: 5% budget consumed in 6 hours
      - alert: ErrorBudgetSlowBurn
        expr: |
          (
            1 - (sum(rate(http_requests_total{status=~"2.."}[6h])) /
                 sum(rate(http_requests_total[6h])))
          ) > (6 * (1 - 0.999))
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Slow burn rate - error budget exhausted in <30 hours"
```

## Custom Business Metrics

### Domain-Specific Metrics

```rust
use prometheus::{
    register_counter_vec, register_histogram_vec, register_gauge,
    CounterVec, HistogramVec, Gauge,
};
use lazy_static::lazy_static;

lazy_static! {
    // Order metrics
    static ref ORDERS_TOTAL: CounterVec = register_counter_vec!(
        "rustmart_orders_total",
        "Total orders by status",
        &["status", "payment_method"]
    ).unwrap();
    
    static ref ORDER_VALUE: HistogramVec = register_histogram_vec!(
        "rustmart_order_value_dollars",
        "Order value distribution",
        &["customer_tier"],
        vec![10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]
    ).unwrap();
    
    static ref ACTIVE_CARTS: Gauge = register_gauge!(
        "rustmart_active_carts",
        "Number of active shopping carts"
    ).unwrap();
    
    static ref REVENUE_TOTAL: CounterVec = register_counter_vec!(
        "rustmart_revenue_dollars_total",
        "Total revenue in dollars",
        &["product_category"]
    ).unwrap();
    
    // Conversion funnel
    static ref FUNNEL_STEP: CounterVec = register_counter_vec!(
        "rustmart_conversion_funnel_total",
        "Conversion funnel steps",
        &["step"]
    ).unwrap();
}

// Track order completion
pub fn record_order_completed(order: &Order) {
    ORDERS_TOTAL
        .with_label_values(&["completed", &order.payment_method])
        .inc();
    
    ORDER_VALUE
        .with_label_values(&[&order.customer_tier])
        .observe(order.total);
    
    REVENUE_TOTAL
        .with_label_values(&[&order.category])
        .inc_by(order.total);
}

// Track conversion funnel
pub fn track_funnel(step: &str) {
    FUNNEL_STEP.with_label_values(&[step]).inc();
}

// Usage in handlers
async fn add_to_cart(/* ... */) -> Result<(), Error> {
    track_funnel("add_to_cart");
    // ... rest of logic
}

async fn checkout(/* ... */) -> Result<(), Error> {
    track_funnel("checkout_initiated");
    // ... payment processing
    track_funnel("checkout_completed");
    Ok(())
}
```

### Grafana Dashboards for Business Metrics

```json
{
  "dashboard": {
    "title": "RustMart Business Metrics",
    "panels": [
      {
        "title": "Orders Per Minute",
        "targets": [{
          "expr": "rate(rustmart_orders_total{status='completed'}[5m]) * 60"
        }]
      },
      {
        "title": "Revenue (Last 24h)",
        "targets": [{
          "expr": "increase(rustmart_revenue_dollars_total[24h])"
        }]
      },
      {
        "title": "Conversion Rate",
        "targets": [{
          "expr": "(rate(rustmart_conversion_funnel_total{step='checkout_completed'}[5m]) / rate(rustmart_conversion_funnel_total{step='add_to_cart'}[5m])) * 100"
        }]
      },
      {
        "title": "Average Order Value",
        "targets": [{
          "expr": "sum(rate(rustmart_order_value_dollars_sum[5m])) / sum(rate(rustmart_order_value_dollars_count[5m]))"
        }]
      }
    ]
  }
}
```

## Advanced Distributed Tracing

### Trace Context Propagation with Baggage

```rust
use opentelemetry::{
    global,
    trace::{Span, SpanKind, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

async fn create_order_with_context(
    customer_id: Uuid,
    items: Vec<OrderItem>,
) -> Result<Order, Error> {
    let tracer = global::tracer("order-service");
    
    let mut span = tracer
        .span_builder("create_order")
        .with_kind(SpanKind::Server)
        .start(&tracer);
    
    // Add attributes
    span.set_attribute(KeyValue::new("customer.id", customer_id.to_string()));
    span.set_attribute(KeyValue::new("order.item_count", items.len() as i64));
    
    // Set baggage for cross-service context
    let cx = Context::current_with_span(span);
    let cx = cx.with_baggage(vec![
        KeyValue::new("customer.tier", "premium"),
        KeyValue::new("experiment.variant", "checkout_v2"),
    ]);
    
    // Make downstream call with context
    let result = inventory_client
        .check_availability(&items)
        .with_context(cx.clone())
        .await?;
    
    Ok(result)
}
```

### Trace Sampling Strategies

```rust
use opentelemetry::sdk::trace::{Sampler, SamplerDecision};

// Custom sampler: Sample all errors, 10% of successes
struct ErrorAwareSampler;

impl Sampler for ErrorAwareSampler {
    fn should_sample(
        &self,
        parent_context: &Context,
        trace_id: opentelemetry::trace::TraceId,
        name: &str,
        span_kind: &SpanKind,
        attributes: &[KeyValue],
        links: &[opentelemetry::trace::Link],
    ) -> SamplerDecision {
        // Check if this is an error span
        if attributes.iter().any(|kv| {
            kv.key.as_str() == "error" && kv.value.as_str() == "true"
        }) {
            return SamplerDecision {
                decision: opentelemetry::sdk::trace::Decision::RecordAndSample,
                attributes: vec![],
                trace_state: Default::default(),
            };
        }
        
        // Sample 10% of normal requests
        if trace_id.to_bytes()[0] < 26 {  // ~10% (256 * 0.1 ≈ 26)
            SamplerDecision {
                decision: opentelemetry::sdk::trace::Decision::RecordAndSample,
                attributes: vec![],
                trace_state: Default::default(),
            }
        } else {
            SamplerDecision {
                decision: opentelemetry::sdk::trace::Decision::Drop,
                attributes: vec![],
                trace_state: Default::default(),
            }
        }
    }
}
```

## Log Correlation with Traces

### Structured Logging with Trace IDs

```rust
use tracing::{info, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

#[instrument(skip(pool), fields(trace_id, span_id))]
async fn process_order(pool: &PgPool, order_id: Uuid) -> Result<(), Error> {
    let span = tracing::Span::current();
    
    // Extract trace context
    let context = span.context();
    let trace_id = format!("{:x}", context.span().span_context().trace_id());
    let span_id = format!("{:x}", context.span().span_context().span_id());
    
    // Add to span
    span.record("trace_id", &trace_id.as_str());
    span.record("span_id", &span_id.as_str());
    
    info!(
        order_id = %order_id,
        trace_id = %trace_id,
        span_id = %span_id,
        "Processing order"
    );
    
    // Process...
    
    Ok(())
}

// Setup tracing with OpenTelemetry
fn init_tracing() -> Result<(), Error> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    let telemetry = OpenTelemetryLayer::new(tracer);
    
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
        );
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

## Continuous Profiling

### CPU Profiling with pprof

```rust
use pprof::ProfilerGuard;

// Start profiler
let guard = ProfilerGuard::new(100)?;  // Sample at 100Hz

// Run application...

// Generate flamegraph
if let Ok(report) = guard.report().build() {
    let file = std::fs::File::create("flamegraph.svg")?;
    report.flamegraph(file)?;
}
```

### Integration with Handlers

```rust
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone)]
struct ProfilingState {
    guard: Arc<Mutex<Option<ProfilerGuard<'static>>>>,
}

async fn start_profiling(
    State(state): State<ProfilingState>,
) -> Result<StatusCode, StatusCode> {
    let guard = ProfilerGuard::new(100)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    *state.guard.lock() = Some(guard);
    Ok(StatusCode::OK)
}

async fn stop_profiling_and_get_flamegraph(
    State(state): State<ProfilingState>,
) -> Result<Vec<u8>, StatusCode> {
    let guard = state.guard.lock().take()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let report = guard.report().build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut buf = Vec::new();
    report.flamegraph(&mut buf)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(buf)
}
```

## Instana Advanced Integration

### Custom Spans and Tags

```rust
use instana::Tracer;

async fn process_payment_with_instana(
    order: &Order,
    tracer: &Tracer,
) -> Result<PaymentResult, Error> {
    let mut span = tracer.start_span("payment.process");
    
    // Add custom tags for Instana
    span.set_tag("payment.method", &order.payment_method);
    span.set_tag("payment.amount", order.total);
    span.set_tag("customer.id", &order.customer_id.to_string());
    span.set_tag("order.id", &order.id.to_string());
    
    let result = match payment_gateway.charge(order).await {
        Ok(result) => {
            span.set_tag("payment.status", "success");
            span.set_tag("payment.transaction_id", &result.transaction_id);
            Ok(result)
        }
        Err(e) => {
            span.set_tag("payment.status", "failed");
            span.set_tag("payment.error", &e.to_string());
            span.set_error();
            Err(e)
        }
    };
    
    span.finish();
    result
}
```

## Alerting Strategies

### Multi-Level Alert Routing

```yaml
# alertmanager.yml
route:
  receiver: 'default'
  group_by: ['alertname', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  
  routes:
    # Critical: Page immediately
    - match:
        severity: critical
      receiver: pagerduty
      continue: true
    
    # Warning: Slack only
    - match:
        severity: warning
      receiver: slack
    
    # Business metrics: Different channel
    - match:
        team: business
      receiver: slack-business

receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR_KEY'
  
  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
  
  - name: 'slack-business'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#business-metrics'
```

### Runbook Automation

```rust
// Alert webhook handler
#[derive(Deserialize)]
struct AlertWebhook {
    alerts: Vec<Alert>,
}

#[derive(Deserialize)]
struct Alert {
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    status: String,
}

async fn handle_alert_webhook(
    Json(webhook): Json<AlertWebhook>,
) -> Result<StatusCode, StatusCode> {
    for alert in webhook.alerts {
        if alert.status == "firing" {
            // Auto-remediation
            match alert.labels.get("alertname").map(|s| s.as_str()) {
                Some("HighMemoryUsage") => {
                    trigger_pod_restart(&alert).await?;
                }
                Some("DatabaseConnectionPoolExhausted") => {
                    increase_connection_pool(&alert).await?;
                }
                _ => {}
            }
            
            // Create incident ticket
            create_incident_ticket(&alert).await?;
        }
    }
    
    Ok(StatusCode::OK)
}
```

## Best Practices

- **Start with SLOs** - Define objectives before implementing alerts
- **Keep Cardinality Low** - Avoid high-cardinality labels (use <100 unique values)
- **Correlate Everything** - Use consistent trace IDs across metrics, logs, traces
- **Monitor the Monitors** - Track Prometheus/Grafana availability
- **Test in CI/CD** - Validate metrics collection in pipelines
- **Document Runbooks** - Link alerts to resolution procedures
- **Review Regularly** - Tune alerts monthly, update SLOs quarterly

## Official Documentation

- [Google SRE: Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/)
- [Implementing SLOs](https://sre.google/workbook/implementing-slos/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Instana Rust SDK](https://www.ibm.com/docs/en/instana-observability)
