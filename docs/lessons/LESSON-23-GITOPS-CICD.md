# Lesson 23: GitOps and CI/CD

## Overview
Automate testing, building, and deployment of RustMart microservices using GitHub Actions for CI/CD and ArgoCD for GitOps-driven Kubernetes deployments.

## GitHub Actions CI/CD Pipeline

### Basic Workflow Structure
```yaml
# .github/workflows/ci.yml
name: CI Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --workspace --all-features
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/test
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Security audit
        run: cargo audit
```

### Multi-Service Build Pipeline
```yaml
# .github/workflows/build.yml
name: Build and Push

on:
  push:
    branches: [main]
    tags:
      - 'v*'

jobs:
  build-matrix:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [product-service, order-service, inventory-service, api-gateway]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Docker meta
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ github.repository }}/${{ matrix.service }}
          tags: |
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=sha,prefix={{branch}}-
      
      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./services/${{ matrix.service }}/Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Integration Tests in CI
```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests

on:
  pull_request:
    branches: [main]

jobs:
  integration:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Start services
        run: docker-compose up -d
      
      - name: Wait for services
        run: |
          timeout 60 bash -c 'until curl -f http://localhost:8000/health; do sleep 2; done'
      
      - name: Run integration tests
        run: |
          cargo test --test integration_tests -- --test-threads=1
      
      - name: Collect logs
        if: failure()
        run: docker-compose logs
      
      - name: Cleanup
        if: always()
        run: docker-compose down -v
```

## ArgoCD GitOps Setup

### Installation
```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Access ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Get initial admin password
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

### Application Manifest
```yaml
# argocd/product-service-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: product-service
  namespace: argocd
spec:
  project: rustmart
  
  source:
    repoURL: https://github.com/your-org/rustmart
    targetRevision: main
    path: k8s/product-service
    
    kustomize:
      images:
        - ghcr.io/your-org/rustmart/product-service:latest
  
  destination:
    server: https://kubernetes.default.svc
    namespace: rustmart
  
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
    
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

### Kustomize Structure
```yaml
# k8s/product-service/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: rustmart

resources:
  - deployment.yaml
  - service.yaml
  - configmap.yaml

images:
  - name: product-service
    newName: ghcr.io/your-org/rustmart/product-service
    newTag: latest

configMapGenerator:
  - name: product-service-config
    literals:
      - LOG_LEVEL=info
      - RUST_LOG=product_service=debug
```

## Deployment Strategies

### Blue-Green Deployment
```yaml
# k8s/product-service/deployment-blue-green.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: product-service
spec:
  replicas: 3
  strategy:
    blueGreen:
      activeService: product-service
      previewService: product-service-preview
      autoPromotionEnabled: false
      scaleDownDelaySeconds: 30
  
  selector:
    matchLabels:
      app: product-service
  
  template:
    metadata:
      labels:
        app: product-service
    spec:
      containers:
        - name: product-service
          image: ghcr.io/your-org/rustmart/product-service:latest
          ports:
            - containerPort: 8001
```

### Canary Deployment
```yaml
# k8s/product-service/rollout-canary.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: product-service
spec:
  replicas: 5
  strategy:
    canary:
      steps:
        - setWeight: 20
        - pause: {duration: 5m}
        - setWeight: 40
        - pause: {duration: 5m}
        - setWeight: 60
        - pause: {duration: 5m}
        - setWeight: 80
        - pause: {duration: 5m}
      
      analysis:
        templates:
          - templateName: success-rate
        startingStep: 2
  
  selector:
    matchLabels:
      app: product-service
  
  template:
    metadata:
      labels:
        app: product-service
    spec:
      containers:
        - name: product-service
          image: ghcr.io/your-org/rustmart/product-service:latest
```

### Analysis Template (for Canary)
```yaml
# k8s/analysis-template.yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  metrics:
    - name: success-rate
      interval: 1m
      successCondition: result >= 0.95
      failureLimit: 3
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{status=~"2.."}[5m])) /
            sum(rate(http_requests_total[5m]))
```

## Automated Testing in CD

### Smoke Tests
```rust
// tests/smoke_tests.rs
use reqwest::Client;

#[tokio::test]
async fn smoke_test_product_service() {
    let client = Client::new();
    let base_url = std::env::var("SERVICE_URL").unwrap_or("http://localhost:8001".to_string());
    
    // Health check
    let health = client.get(&format!("{}/health", base_url))
        .send()
        .await
        .expect("Health check failed");
    assert_eq!(health.status(), 200);
    
    // Basic functionality
    let products = client.get(&format!("{}/products", base_url))
        .send()
        .await
        .expect("List products failed");
    assert_eq!(products.status(), 200);
}
```

### Post-Deployment Verification
```yaml
# .github/workflows/post-deploy.yml
name: Post-Deployment Tests

on:
  deployment_status:

jobs:
  verify:
    if: github.event.deployment_status.state == 'success'
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Run smoke tests
        run: |
          export SERVICE_URL=${{ github.event.deployment_status.target_url }}
          cargo test --test smoke_tests
      
      - name: Load test
        run: |
          ./load-test-tool \
            --url ${{ github.event.deployment_status.target_url }} \
            --duration 60 \
            --rps 100
```

## Environment Management

### Multi-Environment Setup
```yaml
# argocd/projects/rustmart.yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: rustmart
  namespace: argocd
spec:
  description: RustMart microservices
  
  sourceRepos:
    - 'https://github.com/your-org/rustmart'
  
  destinations:
    - namespace: rustmart-dev
      server: https://kubernetes.default.svc
    - namespace: rustmart-staging
      server: https://kubernetes.default.svc
    - namespace: rustmart-prod
      server: https://kubernetes.default.svc
  
  clusterResourceWhitelist:
    - group: '*'
      kind: '*'
```

### Environment-Specific Overlays
```yaml
# k8s/overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: rustmart-prod

bases:
  - ../../base

replicas:
  - name: product-service
    count: 5

images:
  - name: product-service
    newTag: v1.2.3  # Specific version for prod

configMapGenerator:
  - name: product-service-config
    behavior: merge
    literals:
      - LOG_LEVEL=warn
      - ENVIRONMENT=production
```

## Secrets Management in GitOps

### Sealed Secrets
```bash
# Install sealed-secrets controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml

# Create sealed secret
echo -n 'my-secret-password' | kubectl create secret generic db-password \
  --dry-run=client \
  --from-file=password=/dev/stdin \
  -o yaml | \
kubeseal -o yaml > sealed-secret.yaml
```

```yaml
# k8s/product-service/sealed-secret.yaml
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: db-password
  namespace: rustmart
spec:
  encryptedData:
    password: AgBHR8... # Encrypted value
```

## Monitoring Deployments

### Prometheus Metrics
```rust
// Add deployment metrics
use prometheus::{register_counter, Counter};

lazy_static! {
    static ref DEPLOYMENT_COUNTER: Counter = register_counter!(
        "app_deployment_total",
        "Total number of deployments"
    ).unwrap();
}

pub fn record_deployment() {
    DEPLOYMENT_COUNTER.inc();
}
```

### Grafana Dashboard for Deployments
```json
{
  "dashboard": {
    "title": "RustMart Deployments",
    "panels": [
      {
        "title": "Deployment Frequency",
        "targets": [
          {
            "expr": "rate(app_deployment_total[1h])"
          }
        ]
      },
      {
        "title": "Deployment Success Rate",
        "targets": [
          {
            "expr": "sum(rate(deployment_status{status='success'}[1h])) / sum(rate(deployment_status[1h]))"
          }
        ]
      }
    ]
  }
}
```

## Best Practices

1. **Git as Source of Truth**: All manifests in Git
2. **Immutable Images**: Use specific tags, never `:latest` in prod
3. **Progressive Rollouts**: Start with canary/blue-green
4. **Automated Rollbacks**: Set failure thresholds
5. **Environment Parity**: Keep dev/staging/prod similar
6. **Secret Management**: Never commit secrets to Git
7. **PR Reviews**: Require approvals for production changes
8. **Monitoring**: Track deployment metrics and success rates

## Official Documentation

- [GitHub Actions](https://docs.github.com/en/actions)
- [ArgoCD](https://argo-cd.readthedocs.io/)
- [Argo Rollouts](https://argoproj.github.io/argo-rollouts/)
- [Kustomize](https://kustomize.io/)
- [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets)
