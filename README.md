# RustMart - Microservices E-Commerce Demo

A production-ready microservices application built in Rust for demonstrating observability with Instana.

## 🏗️ Architecture

```
┌─────────────┐
│ API Gateway │ :8080
└──────┬──────┘
       │
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│ Product  │   │ Order    │   │Inventory │   │ Payment  │
│ Service  │   │ Service  │   │ Service  │   │ Service  │
│  :8081   │   │  :8082   │   │  :8083   │   │  :8084   │
└────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘
     │              │              │              │
     ▼              ▼              ▼              │
┌──────────┐   ┌──────────┐   ┌──────────┐      │
│PostgreSQL│   │PostgreSQL│   │PostgreSQL│      │
│          │   │          │   │  +Redis  │      │
└──────────┘   └────┬─────┘   └──────────┘      │
                    │                            │
                    └────────────┬───────────────┘
                                 ▼
                          ┌──────────────┐
                          │  RabbitMQ    │
                          │  (Events)    │
                          └──────────────┘
```

## 🎯 Services

- **API Gateway** (8080): Entry point, routing, rate limiting
- **Product Service** (8081): Product catalog with 1000+ mock products
- **Order Service** (8082): Order processing with event publishing
- **Inventory Service** (8083): Stock management with Redis caching
- **Payment Service** (8084): Payment simulation with realistic delays

## 🚀 Quick Start

### Docker Compose (Development)
```bash
docker-compose up -d
```

### K3s (Local Kubernetes)
```bash
./scripts/deploy-k3s.sh
```

### Kubernetes (Production)
```bash
kubectl apply -f k8s/
```

## 📊 Load Testing

```bash
# Light load
./scripts/load-test.sh light

# Heavy load
./scripts/load-test.sh heavy
```

## 🔍 Observability

- **Traces**: OpenTelemetry → Instana
- **Metrics**: Prometheus format on `/metrics`
- **Logs**: Structured JSON logs
- **Health**: `/health` and `/ready` endpoints

## 🛠️ Development

### Build all services
```bash
cargo build --workspace
```

### Run a service locally
```bash
cargo run -p product-service
```

### Run tests
```bash
cargo test --workspace
```

## 📦 Tech Stack

- **Framework**: Axum (async web framework)
- **Database**: PostgreSQL + SQLx
- **Cache**: Redis
- **Message Queue**: RabbitMQ
- **Observability**: OpenTelemetry + Tracing
- **Deployment**: Docker, K3s, K8s

## 🎓 Learning Rust

This project demonstrates:
- ✅ Async/await with Tokio
- ✅ Error handling with Result types
- ✅ Database operations
- ✅ Message queues
- ✅ HTTP clients & servers
- ✅ Trait implementations
- ✅ Testing strategies
- ✅ Production patterns

## Author

Terry Lowery
