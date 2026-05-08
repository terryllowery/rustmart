# Lesson 22: Chaos Engineering

## Overview
Test system resilience by intentionally introducing failures using chaos engineering principles. Validate fault tolerance, circuit breakers, and recovery mechanisms in RustMart.

## Why This Matters
Chaos engineering helps you:
- **Find weaknesses before customers do** - Discover failure modes in controlled conditions
- **Build confidence in resilience** - Prove your system can handle failures
- **Improve incident response** - Train teams through realistic failure scenarios
- **Validate SLOs** - Ensure your system meets availability targets under stress

Netflix pioneered this with Chaos Monkey, randomly terminating production instances to ensure services handle failures gracefully.

## Chaos Engineering Principles

### The Scientific Method for Distributed Systems

1. **Define Steady State** - Establish baseline metrics (latency, error rate, throughput)
2. **Hypothesize** - Predict system behavior during failure
3. **Introduce Chaos** - Inject controlled failures
4. **Observe & Learn** - Compare actual vs expected behavior
5. **Improve** - Fix discovered issues, repeat

### Key Principles

**Minimize Blast Radius**
- Start in non-production environments
- Limit scope (single service, small percentage of traffic)
- Use feature flags for instant rollback
- Have automated circuit breakers

**Automate Experiments**
- Chaos as code (version controlled)
- Run regularly (weekly/monthly)
- Integrate into CI/CD pipeline
- Continuous validation of resilience

## Installing Chaos Mesh on Kubernetes

### Step 1: Install Chaos Mesh

```bash
# Add Chaos Mesh Helm repository
helm repo add chaos-mesh https://charts.chaos-mesh.org
helm repo update

# Install Chaos Mesh
kubectl create namespace chaos-mesh
helm install chaos-mesh chaos-mesh/chaos-mesh \
  --namespace=chaos-mesh \
  --set chaosDaemon.runtime=containerd \
  --set chaosDaemon.socketPath=/run/containerd/containerd.sock

# Verify installation
kubectl get pods -n chaos-mesh
```

### Step 2: Access Chaos Dashboard

```bash
# Port-forward to access dashboard
kubectl port-forward -n chaos-mesh svc/chaos-dashboard 2333:2333

# Open browser to http://localhost:2333
```

## Network Chaos Experiments

### Experiment 1: Inject Network Latency

Test how RustMart handles slow network connections between services.

**Hypothesis**: "When network latency between order-service and inventory-service increases to 500ms, the circuit breaker opens within 10 seconds, and order success rate remains above 95%."

**Chaos Mesh Manifest**:
```yaml
# chaos/network-latency.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: network-delay-inventory
  namespace: rustmart
spec:
  action: delay
  mode: one  # or 'all', 'fixed', 'fixed-percent'
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: inventory-service
  delay:
    latency: "500ms"
    correlation: "25"  # 25% correlation between delays
    jitter: "100ms"    # random jitter
  duration: "5m"
  scheduler:
    cron: "@every 1h"  # Run hourly
```

**Apply Chaos**:
```bash
kubectl apply -f chaos/network-latency.yaml

# Monitor experiment
kubectl get networkchaos -n rustmart
kubectl describe networkchaos network-delay-inventory -n rustmart
```

### Experiment 2: Network Partition (Split Brain)

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind:NetworkChaos
metadata:
  name: network-partition
  namespace: rustmart
spec:
  action: partition
  mode: all
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: order-service
  direction: to
  target:
    mode: all
    selector:
      namespaces:
        - rustmart
      labelSelectors:
        app: inventory-service
  duration: "2m"
```

### Experiment 3: Packet Loss

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: packet-loss
  namespace: rustmart
spec:
  action: loss
  mode: one
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: product-service
  loss:
    loss: "25"         # 25% packet loss
    correlation: "25"
  duration: "3m"
```

## Pod Chaos Experiments

### Experiment 4: Pod Kill (Simulate Node Failure)

**Hypothesis**: "When a product-service pod is killed, Kubernetes restarts it within 30s, and the API gateway routes requests to healthy pods with <1% error rate increase."

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-kill-product-service
  namespace: rustmart
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: product-service
  duration: "30s"
  scheduler:
    cron: "@every 30m"
```

**Monitor Recovery**:
```bash
# Watch pod restarts
kubectl get pods -n rustmart -w

# Check restart count
kubectl get pods -n rustmart -l app=product-service \
  -o jsonpath='{.items[*].status.containerStatuses[*].restartCount}'

# View events
kubectl get events -n rustmart --sort-by='.lastTimestamp'
```

### Experiment 5: Pod Failure (Keep Pod Running But Unresponsive)

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-failure
  namespace: rustmart
spec:
  action: pod-failure
  mode: fixed
  value: "1"
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: order-service
  duration: "2m"
```

## Stress Testing (Resource Exhaustion)

### Experiment 6: CPU Stress

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: StressChaos
metadata:
  name: cpu-stress
  namespace: rustmart
spec:
  mode: one
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: product-service
  stressors:
    cpu:
      workers: 4
      load: 100  # 100% load per worker
  duration: "5m"
```

### Experiment 7: Memory Pressure

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: StressChaos
metadata:
  name: memory-stress
  namespace: rustmart
spec:
  mode: one
  selector:
    namespaces:
      - rustmart
    labelSelectors:
      app: inventory-service
  stressors:
    memory:
      workers: 1
      size: "1GB"  # Consume 1GB RAM
  duration: "3m"
```

## Using Toxiproxy for Network Chaos

Toxiproxy is a TCP proxy for simulating network conditions without Kubernetes.

### Setup Toxiproxy

```bash
# Run Toxiproxy server
docker run -d --name toxiproxy -p 8474:8474 -p 26379:26379 ghcr.io/shopify/toxiproxy

# Create proxy for PostgreSQL
curl -X POST http://localhost:8474/proxies \
  -d '{"name": "postgres", "listen": "0.0.0.0:26432", "upstream": "postgres:5432"}'
```

### Inject Latency

```bash
# Add 500ms latency
curl -X POST http://localhost:8474/proxies/postgres/toxics \
  -d '{"type": "latency", "attributes": {"latency": 500}}'

# Remove toxic
curl -X DELETE http://localhost:8474/proxies/postgres/toxics/latency_downstream
```

### Rust Client Integration

```rust
use toxiproxy_rust::Client;

#[tokio::test]
async fn test_database_latency_resilience() {
    let toxic_client = Client::new("http://localhost:8474");
    
    // Add 1s latency to database
    toxic_client.add_toxic("postgres", "latency", "downstream", 1.0, 
        json!({"latency": 1000})).await?;
    
    // Test application behavior
    let start = Instant::now();
    let result = app.get_products().await;
    let duration = start.elapsed();
    
    // Verify timeout kicks in
    assert!(duration < Duration::from_secs(5));
    assert!(result.is_err());
    
    // Cleanup
    toxic_client.remove_toxic("postgres", "latency_downstream").await?;
}
```

## Custom Chaos Injection in Rust Code

### Feature-Flag-Based Chaos

```rust
use rand::Rng;

#[derive(Clone)]
struct ChaosConfig {
    enabled: bool,
    error_rate: f64,      // 0.0 to 1.0
    latency_ms: Option<u64>,
}

impl ChaosConfig {
    fn from_env() -> Self {
        Self {
            enabled: std::env::var("CHAOS_ENABLED").unwrap_or_default() == "true",
            error_rate: std::env::var("CHAOS_ERROR_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
            latency_ms: std::env::var("CHAOS_LATENCY_MS")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }
    
    async fn maybe_inject(&self) -> Result<(), ChaosError> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut rng = rand::thread_rng();
        
        // Inject latency
        if let Some(ms) = self.latency_ms {
            let jitter = rng.gen_range(0..ms / 2);
            tokio::time::sleep(Duration::from_millis(ms + jitter)).await;
        }
        
        // Inject errors
        if rng.gen::<f64>() < self.error_rate {
            return Err(ChaosError::InjectedFailure);
        }
        
        Ok(())
    }
}

// Use in handlers
async fn get_product(chaos: &ChaosConfig, id: Uuid) -> Result<Product, Error> {
    chaos.maybe_inject().await?;
    
    // Normal logic
    fetch_product_from_db(id).await
}
```

### Middleware for Request Delays

```rust
use axum::middleware::Next;
use axum::http::Request;

async fn chaos_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let chaos = ChaosConfig::from_env();
    
    if let Err(_) = chaos.maybe_inject().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(next.run(req).await)
}

// Apply to router
let app = Router::new()
    .route("/products", get(list_products))
    .layer(middleware::from_fn(chaos_middleware));
```

## Observability During Chaos Experiments

### Monitor with Prometheus

```promql
# HTTP error rate
rate(http_requests_total{status=~"5.."}[5m])

# Request latency (p99)
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))

# Circuit breaker state
circuit_breaker_state{service="order-service"}
```

### Grafana Dashboard for Chaos

Create a dashboard with panels:
1. **Success Rate** - Track during experiment
2. **Latency (p50, p95, p99)** - Detect increases
3. **Active Pods** - Monitor restarts
4. **Circuit Breaker State** - Visualize opens/closes
5. **Error Rate by Service** - Identify cascading failures

### Automated Validation with Workflow

```yaml
# chaos/workflow.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: Workflow
metadata:
  name: rustmart-resilience-test
spec:
  entry: entry
  templates:
    - name: entry
      templateType: Serial
      children:
        - baseline-metrics
        - network-chaos
        - validate-recovery
    
    - name: baseline-metrics
      templateType: Task
      task:
        # Collect baseline for 2 minutes
        duration: 2m
    
    - name: network-chaos
      templateType: NetworkChaos
      networkChaos:
        action: delay
        mode: one
        selector:
          labelSelectors:
            app: inventory-service
        delay:
          latency: "500ms"
        duration: "3m"
    
    - name: validate-recovery
      templateType: Task
      task:
        # Verify metrics return to baseline
        duration: 2m
```

## GameDay Exercises

### Planning a GameDay

1. **Schedule** - Pick non-peak hours, notify team
2. **Scenario** - Define realistic failure (e.g., "database primary fails")
3. **Success Criteria** - Clear metrics (e.g., "recovery <5min, no data loss")
4. **Roles** - Assign incident commander, responders, observers
5. **Runbook** - Have recovery procedures ready

### Example GameDay Scenario

**Scenario**: PostgreSQL primary database becomes unavailable

**Steps**:
1. Inject chaos (kill postgres primary pod)
2. Monitor alert triggers
3. Team follows runbook to promote replica
4. Verify application failover
5. Document response time, issues encountered
6. Post-mortem: What went well? What needs improvement?

## Chaos in CI/CD Pipeline

### GitHub Actions Integration

```yaml
# .github/workflows/chaos-test.yml
name: Chaos Tests

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  chaos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup kubectl
        uses: azure/setup-kubectl@v3
      
      - name: Run pod kill experiment
        run: |
          kubectl apply -f chaos/pod-kill-experiment.yaml
          sleep 300  # Run for 5 minutes
      
      - name: Validate metrics
        run: |
          python scripts/validate_slo.py --experiment pod-kill
      
      - name: Cleanup
        if: always()
        run: kubectl delete -f chaos/pod-kill-experiment.yaml
```

## Testing Resilience Patterns

### Circuit Breaker Validation

```rust
#[tokio::test]
async fn test_circuit_breaker_opens_on_failures() {
    let client = TestClient::new();
    let chaos = ChaosConfig { enabled: true, error_rate: 1.0, latency_ms: None };
    
    // Make requests until circuit opens
    let mut failures = 0;
    for _ in 0..20 {
        if client.call_inventory_service().await.is_err() {
            failures += 1;
        }
    }
    
    // Circuit should open after threshold (e.g., 10 failures)
    assert!(failures >= 10);
    
    // Subsequent requests should fail fast
    let start = Instant::now();
    let result = client.call_inventory_service().await;
    assert!(start.elapsed() < Duration::from_millis(100)); // Fast fail
    assert!(result.is_err());
}
```

### Retry with Exponential Backoff Test

```rust
#[tokio::test]
async fn test_exponential_backoff() {
    let chaos = ChaosConfig { enabled: true, error_rate: 0.8, latency_ms: None };
    
    let start = Instant::now();
    let result = retry_with_backoff(|| async {
        chaos.maybe_inject().await?;
        Ok("success")
    }, 5).await;
    
    let duration = start.elapsed();
    
    // Should have retried with backoff: ~1s + 2s + 4s + 8s
    assert!(duration > Duration::from_secs(10));
}
```

## Safety and Blast Radius Control

### Progressive Rollout

```yaml
# Start with single pod
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: progressive-chaos
spec:
  mode: fixed-percent
  value: "10"  # Only 10% of pods
  selector:
    labelSelectors:
      app: product-service
```

### Automated Rollback

```rust
// Monitor metrics, rollback if SLO violated
async fn run_chaos_with_rollback(experiment: ChaosExperiment) -> Result<()> {
    let baseline_error_rate = get_current_error_rate().await?;
    
    experiment.apply().await?;
    
    // Monitor for 5 minutes
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        let current_error_rate = get_current_error_rate().await?;
        
        // Rollback if error rate > 5%
        if current_error_rate > 0.05 {
            warn!("Error rate exceeded threshold, rolling back");
            experiment.delete().await?;
            return Err(Error::SLOViolation);
        }
    }
    
    experiment.delete().await?;
    Ok(())
}
```

## Tools & Libraries

- **Chaos Mesh**: Kubernetes-native chaos engineering
- **Litmus Chaos**: CNCF chaos engineering framework
- **Toxiproxy**: Network chaos proxy
- **Pumba**: Docker chaos testing
- **Gremlin**: Commercial chaos engineering platform
- **Chaos Toolkit**: Open-source chaos automation

## Hands-on Exercises

1. Install Chaos Mesh on Kubernetes cluster
2. Run pod kill experiment on product-service
3. Inject network latency with Toxiproxy
4. Create custom chaos experiment workflow
5. Conduct GameDay exercise with team
6. Integrate chaos tests into CI/CD pipeline

## Best Practices

- Always define steady-state metrics before experiments
- Start small, increase blast radius gradually
- Document all experiments and findings
- Run experiments regularly (weekly/monthly)
- Involve entire team in GameDays
- Automate chaos experiments in CI/CD
- Use observability tools to measure impact
- Have rollback plan for every experiment

## Resources

- [Principles of Chaos Engineering](https://principlesofchaos.org/)
- [Chaos Mesh Documentation](https://chaos-mesh.org/)
- [Litmus Chaos](https://litmuschaos.io/)
- [Netflix Chaos Engineering](https://netflix.github.io/chaosmonkey/)
- [Google SRE Book: Testing for Reliability](https://sre.google/sre-book/testing-reliability/)
## Hands-on Exercises

1. **Install Chaos Mesh** - Set up on your K8s cluster
2. **Run Network Latency Experiment** - Inject 500ms delay, observe circuit breaker
3. **Pod Kill Test** - Terminate product-service pod, verify recovery
4. **Custom Chaos Injection** - Add feature-flag chaos to your Rust service
5. **GameDay Simulation** - Run full team exercise with database failover
6. **CI/CD Integration** - Automate chaos tests in GitHub Actions

## Best Practices

- **Start Small** - Begin with staging, single service, short duration
- **Define Hypothesis** - Always predict expected behavior before experiment
- **Monitor Everything** - Metrics, logs, traces during chaos
- **Document Findings** - Record what broke, what worked, improvements made
- **Automate** - Run chaos regularly, not just once
- **Team Involvement** - Include all engineers in GameDays
- **Progressive Rollout** - Increase blast radius gradually
- **Have Rollback** - Automated circuit breakers for safety

## Official Documentation

- [Principles of Chaos Engineering](https://principlesofchaos.org/)
- [Chaos Mesh Documentation](https://chaos-mesh.org/)
- [Litmus Chaos](https://litmuschaos.io/)
- [Toxiproxy](https://github.com/Shopify/toxiproxy)
- [Google SRE: Testing for Reliability](https://sre.google/sre-book/testing-reliability/)
