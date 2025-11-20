# Lesson 14: Kubernetes Deployment

## Overview
You've built microservices. Now deploy them to **Kubernetes** (K8s), the industry-standard container orchestration platform. K8s handles scaling, self-healing, load balancing, and rolling updates automatically.

By the end of this lesson, you'll have:
- Kubernetes manifests for all services
- Secrets and ConfigMaps
- Auto-scaling with HPA
- Health checks and readiness probes
- Service mesh basics with Linkerd

## What is Kubernetes?

Kubernetes orchestrates containers at scale:
- **Pods**: Smallest unit (one or more containers)
- **Deployments**: Manage replicas of pods
- **Services**: Load balance traffic to pods
- **Ingress**: HTTP routing from outside
- **ConfigMaps**: Configuration data
- **Secrets**: Sensitive data (passwords, tokens)

## Prerequisites

Install tools:
```bash
# Kubernetes CLI
brew install kubectl

# Local K8s cluster (choose one)
brew install minikube  # OR
brew install kind      # Kubernetes in Docker
```

Start local cluster:
```bash
minikube start
# OR
kind create cluster --name rustmart
```

## Step 1: Create Kubernetes Manifests

Create `k8s/` directory structure:
```bash
cd ~/code/rustmart
mkdir -p k8s/{base,overlays/{dev,prod}}
```

### Database Deployment

Create `k8s/base/postgres.yaml`:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: rustmart
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
```

### Create Secrets

```bash
kubectl create secret generic postgres-secret \
  --from-literal=username=rustmart_user \
  --from-literal=password=rustmart_pass

kubectl create secret generic jwt-secret \
  --from-literal=secret=your-production-secret-here
```

### Product Service Deployment

Create `k8s/base/product-service.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: product-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: product-service
  template:
    metadata:
      labels:
        app: product-service
        version: v1
    spec:
      containers:
      - name: product-service
        image: rustmart/product-service:latest
        ports:
        - containerPort: 8001
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: database_url
        - name: KAFKA_BROKERS
          value: "kafka:9092"
        - name: RUST_LOG
          value: "info"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://jaeger:4317"
        livenessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 10
          periodSeconds: 5
        readinessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 5
          periodSeconds: 3
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: product-service
spec:
  selector:
    app: product-service
  ports:
  - port: 8001
    targetPort: 8001
  type: ClusterIP
```

**Key features:**
- **replicas: 3**: Three instances for high availability
- **livenessProbe**: Restart if unhealthy
- **readinessProbe**: Don't route traffic until ready
- **resources**: CPU/memory requests and limits

## Step 2: Horizontal Pod Autoscaler

Scale based on CPU usage:

Create `k8s/base/product-service-hpa.yaml`:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: product-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: product-service
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

This automatically scales pods when CPU > 70% or memory > 80%.

## Step 3: ConfigMap for Configuration

Create `k8s/base/configmap.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rustmart-config
data:
  RUST_LOG: "info"
  KAFKA_BROKERS: "kafka:9092"
  PRODUCT_SERVICE_URL: "http://product-service:8001"
  ORDER_SERVICE_URL: "http://order-service:8002"
  OTEL_EXPORTER_OTLP_ENDPOINT: "http://jaeger:4317"
```

Reference in deployments:

```yaml
envFrom:
- configMapRef:
    name: rustmart-config
```

## Step 4: Ingress for External Access

Create `k8s/base/ingress.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rustmart-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  ingressClassName: nginx
  rules:
  - host: rustmart.local
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 8000
      - path: /products
        pathType: Prefix
        backend:
          service:
            name: product-service
            port:
              number: 8001
```

Add to `/etc/hosts`:
```
127.0.0.1 rustmart.local
```

## Step 5: Deploy to Kubernetes

Apply all manifests:

```bash
cd ~/code/rustmart/k8s

# Create namespace
kubectl create namespace rustmart

# Apply manifests
kubectl apply -f base/ -n rustmart

# Check status
kubectl get pods -n rustmart
kubectl get services -n rustmart
```

View logs:
```bash
kubectl logs -f deployment/product-service -n rustmart
```

Port forward for testing:
```bash
kubectl port-forward service/product-service 8001:8001 -n rustmart
curl http://localhost:8001/health
```

## Step 6: Rolling Updates

Update your code, build new image, and deploy:

```bash
# Build and tag new version
docker build -t rustmart/product-service:v2 -f product-service/Dockerfile .

# Load into minikube (if using minikube)
minikube image load rustmart/product-service:v2

# Update deployment
kubectl set image deployment/product-service \
  product-service=rustmart/product-service:v2 \
  -n rustmart

# Watch rollout
kubectl rollout status deployment/product-service -n rustmart
```

**Zero downtime!** Old pods stay running until new ones are ready.

## Step 7: Kustomize for Environments

Use Kustomize to manage dev/prod differences.

Create `k8s/base/kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - postgres.yaml
  - product-service.yaml
  - product-service-hpa.yaml
  - api-gateway.yaml
  - configmap.yaml
  - ingress.yaml

namePrefix: rustmart-
namespace: rustmart

commonLabels:
  app.kubernetes.io/name: rustmart
  app.kubernetes.io/managed-by: kustomize
```

Create `k8s/overlays/dev/kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

bases:
  - ../../base

namespace: rustmart-dev

replicas:
  - name: product-service
    count: 1

configMapGenerator:
  - name: rustmart-config
    behavior: merge
    literals:
      - RUST_LOG=debug
```

Create `k8s/overlays/prod/kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

bases:
  - ../../base

namespace: rustmart-prod

replicas:
  - name: product-service
    count: 5

configMapGenerator:
  - name: rustmart-config
    behavior: merge
    literals:
      - RUST_LOG=warn
```

Deploy:
```bash
# Dev environment
kubectl apply -k k8s/overlays/dev

# Production environment
kubectl apply -k k8s/overlays/prod
```

## Step 8: Observability in Kubernetes

Deploy Jaeger:

```bash
kubectl create namespace observability
kubectl apply -f https://github.com/jaegertracing/jaeger-operator/releases/download/v1.51.0/jaeger-operator.yaml -n observability

# Create Jaeger instance
kubectl apply -f - <<EOF
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: jaeger
  namespace: observability
spec:
  strategy: allInOne
  ingress:
    enabled: true
EOF
```

Update services to point to Jaeger:
```yaml
env:
- name: OTEL_EXPORTER_OTLP_ENDPOINT
  value: "http://jaeger-collector.observability:4317"
```

## Step 9: Service Mesh with Linkerd (Optional)

Linkerd adds mTLS, retries, timeouts automatically.

Install Linkerd:
```bash
# Install CLI
curl --proto '=https' --tlsv1.2 -sSfL https://run.linkerd.io/install | sh

# Install on cluster
linkerd install --crds | kubectl apply -f -
linkerd install | kubectl apply -f -
linkerd check

# Inject Linkerd into namespace
kubectl annotate namespace rustmart linkerd.io/inject=enabled
kubectl rollout restart deployment -n rustmart
```

View dashboard:
```bash
linkerd dashboard
```

Linkerd automatically:
- Encrypts all service-to-service traffic
- Adds retries and timeouts
- Provides golden metrics (success rate, latency, traffic)

## Key Kubernetes Concepts

| Resource | Purpose |
|----------|---------|
| Pod | Running container(s) |
| Deployment | Manages pod replicas |
| Service | Load balances to pods |
| Ingress | External HTTP routing |
| ConfigMap | Non-sensitive config |
| Secret | Sensitive data |
| HPA | Auto-scaling |
| PVC | Persistent storage |

## Challenges

1. **Add Prometheus monitoring**: Scrape metrics from services
2. **Add cert-manager**: Automatic TLS certificates
3. **Add network policies**: Restrict pod-to-pod communication
4. **Add PodDisruptionBudget**: Ensure availability during disruptions

## Next Steps

In **Lesson 15**, you'll add **advanced observability**: Prometheus metrics, Grafana dashboards, and alerting!

## Official Documentation

- [Kubernetes Docs](https://kubernetes.io/docs/)
- [kubectl Cheat Sheet](https://kubernetes.io/docs/reference/kubectl/cheatsheet/)
- [Kustomize](https://kustomize.io/)
- [Linkerd](https://linkerd.io/2.14/getting-started/)
- [Horizontal Pod Autoscaler](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
