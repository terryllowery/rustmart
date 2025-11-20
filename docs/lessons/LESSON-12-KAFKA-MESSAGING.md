# Lesson 12: Message Queue with Kafka

## Overview
So far, services communicate **synchronously** (request → wait → response). But what if you don't need an immediate response? **Event-driven architecture** with message queues lets services communicate **asynchronously**.

By the end of this lesson, you'll have:
- Kafka running in Docker Compose
- Event producer (product-service publishes events)
- Event consumer (order-service listens for events)
- At-least-once delivery guarantees
- Dead letter queues for failed messages

## Why Message Queues?

### Synchronous (what you have now):
```
API Gateway → Order Service → Inventory Service → Payment Service
             ↓ waits        ↓ waits           ↓ waits
```

If any service is slow/down, the whole chain is blocked.

### Asynchronous (with Kafka):
```
Order Service → [Kafka Topic: order.created] → Inventory Service
                                            → Payment Service
                                            → Email Service
```

Order Service publishes event and moves on. Other services process when ready.

### Benefits:
- **Decoupling**: Services don't need to know about each other
- **Resilience**: If a consumer is down, messages wait in queue
- **Scalability**: Add more consumers to handle load
- **Audit trail**: All events stored (Kafka is a log)

## What is Kafka?

Apache Kafka is a distributed **event streaming platform**:
- **Topics**: Categories of messages (e.g., "order.created", "product.updated")
- **Producers**: Services that publish messages
- **Consumers**: Services that read messages
- **Partitions**: Topics split for parallelism
- **Consumer Groups**: Multiple consumers share workload

## Step 1: Add Kafka to Docker Compose

Update `docker-compose.yml`:

```yaml
  zookeeper:
    image: confluentinc/cp-zookeeper:7.5.0
    container_name: rustmart-zookeeper
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000
    ports:
      - "2181:2181"

  kafka:
    image: confluentinc/cp-kafka:7.5.0
    container_name: rustmart-kafka
    depends_on:
      - zookeeper
    ports:
      - "9092:9092"
      - "9094:9094"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092,PLAINTEXT_HOST://localhost:9094
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT,PLAINTEXT_HOST:PLAINTEXT
      KAFKA_INTER_BROKER_LISTENER_NAME: PLAINTEXT
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"

  kafka-ui:
    image: provectuslabs/kafka-ui:latest
    container_name: rustmart-kafka-ui
    depends_on:
      - kafka
    ports:
      - "8080:8080"
    environment:
      KAFKA_CLUSTERS_0_NAME: rustmart
      KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:9092
```

**What's included:**
- **Zookeeper**: Kafka's coordination service
- **Kafka**: The message broker
- **Kafka UI**: Web interface to view topics/messages (http://localhost:8080)

Start the stack:
```bash
docker-compose up -d
```

## Step 2: Add Kafka Client to Shared Crate

We'll use `rdkafka`, a high-performance Kafka client.

Add to `shared/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
rdkafka = { version = "0.36", features = ["cmake-build"] }
```

Create `shared/src/kafka.rs`:

```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer");

        Self { producer }
    }

    #[tracing::instrument(skip(self, payload))]
    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &T,
    ) -> Result<(), KafkaError> {
        let json = serde_json::to_string(payload)
            .map_err(|e| KafkaError::SerializationError(e.to_string()))?;

        let record = FutureRecord::to(topic)
            .payload(&json)
            .key(key.unwrap_or(""));

        self.producer
            .send(record, Duration::from_secs(0))
            .await
            .map_err(|(e, _)| KafkaError::PublishError(e.to_string()))?;

        tracing::info!("Published message to topic: {}", topic);
        Ok(())
    }
}

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(brokers: &str, group_id: &str, topics: &[&str]) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create Kafka consumer");

        consumer
            .subscribe(topics)
            .expect("Failed to subscribe to topics");

        Self { consumer }
    }

    #[tracing::instrument(skip(self, handler))]
    pub async fn consume<T, F, Fut>(&self, mut handler: F)
    where
        T: for<'de> Deserialize<'de>,
        F: FnMut(T) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        use rdkafka::consumer::Consumer;

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    let payload = match message.payload() {
                        Some(p) => p,
                        None => {
                            tracing::warn!("Empty message payload");
                            continue;
                        }
                    };

                    let event: T = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::error!("Failed to deserialize message: {}", e);
                            continue;
                        }
                    };

                    if let Err(e) = handler(event).await {
                        tracing::error!("Handler error: {}", e);
                        // TODO: Publish to dead letter queue
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka consumer error: {}", e);
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KafkaError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Publish error: {0}")]
    PublishError(String),
}
```

Export in `shared/src/lib.rs`:

```rust
pub mod kafka;
```

## Step 3: Define Domain Events

Create `shared/src/events.rs`:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "event_type")]
pub enum DomainEvent {
    ProductCreated(ProductCreatedEvent),
    ProductUpdated(ProductUpdatedEvent),
    ProductDeleted(ProductDeletedEvent),
    OrderCreated(OrderCreatedEvent),
    OrderPaid(OrderPaidEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductCreatedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub product_id: Uuid,
    pub name: String,
    pub price: String, // JSON doesn't handle Decimal well
    pub inventory_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductUpdatedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub product_id: Uuid,
    pub changes: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductDeletedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub product_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderCreatedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<OrderItemEvent>,
    pub total_amount: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItemEvent {
    pub product_id: Uuid,
    pub quantity: i32,
    pub price: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderPaidEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub order_id: Uuid,
    pub payment_method: String,
}
```

Export it:

```rust
// shared/src/lib.rs
pub mod events;
```

## Step 4: Publish Events from Product Service

Update `product-service/src/lib.rs`:

```rust
use shared::kafka::KafkaProducer;
use shared::events::{DomainEvent, ProductCreatedEvent};

#[derive(Clone)]
pub struct AppState {
    pub repo: ProductRepository,
    pub kafka: KafkaProducer,
}

pub fn create_router(pool: PgPool, kafka: KafkaProducer) -> Router {
    let repo = ProductRepository::new(pool);
    
    let state = AppState {
        repo,
        kafka,
    };

    // ... router setup
}

#[tracing::instrument(skip(state))]
async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Create product in database
    let product = state.repo.create(req).await?;

    // Publish event to Kafka
    let event = DomainEvent::ProductCreated(ProductCreatedEvent {
        event_id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        product_id: product.id,
        name: product.name.clone(),
        price: product.price.to_string(),
        inventory_count: product.inventory_count,
    });

    if let Err(e) = state.kafka.publish("product.events", None, &event).await {
        tracing::error!("Failed to publish event: {}", e);
        // Don't fail the request, event publishing is best-effort
    }

    Ok((axum::http::StatusCode::CREATED, Json(product)))
}
```

Update `product-service/src/main.rs`:

```rust
// ... after pool creation ...

// Create Kafka producer
let kafka_brokers = std::env::var("KAFKA_BROKERS")
    .unwrap_or_else(|_| "localhost:9094".to_string());
let kafka = shared::kafka::KafkaProducer::new(&kafka_brokers);

// Create router with Kafka
let app = product_service::create_router(pool, kafka);
```

Update `docker-compose.yml` for product-service:

```yaml
  product-service:
    environment:
      # ... existing env vars ...
      KAFKA_BROKERS: kafka:9092
```

## Step 5: Create Event Consumer Service

Let's create a simple service that logs all product events.

Create `event-consumer/Cargo.toml`:

```toml
[package]
name = "event-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
tokio = { workspace = true, features = ["full"] }
tracing.workspace = true
tracing-subscriber.workspace = true
```

Create `event-consumer/src/main.rs`:

```rust
use shared::events::DomainEvent;
use shared::kafka::KafkaConsumer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting event consumer...");

    let kafka_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9094".to_string());

    let consumer = KafkaConsumer::new(
        &kafka_brokers,
        "event-consumer-group",
        &["product.events"],
    );

    tracing::info!("Listening for events on product.events topic...");

    consumer
        .consume(|event: DomainEvent| async move {
            match event {
                DomainEvent::ProductCreated(e) => {
                    tracing::info!(
                        "ProductCreated: {} - {} at {}",
                        e.product_id,
                        e.name,
                        e.price
                    );
                    // In real app: update search index, send notification, etc.
                }
                DomainEvent::ProductUpdated(e) => {
                    tracing::info!("ProductUpdated: {}", e.product_id);
                }
                DomainEvent::ProductDeleted(e) => {
                    tracing::info!("ProductDeleted: {}", e.product_id);
                }
                _ => {
                    tracing::debug!("Received other event type");
                }
            }
            Ok(())
        })
        .await;
}
```

Add to `docker-compose.yml`:

```yaml
  event-consumer:
    build:
      context: .
      dockerfile: event-consumer/Dockerfile
    container_name: rustmart-event-consumer
    environment:
      KAFKA_BROKERS: kafka:9092
      RUST_LOG: info
    depends_on:
      - kafka
```

Create `event-consumer/Dockerfile`:

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY event-consumer ./event-consumer

WORKDIR /app/event-consumer
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/event-consumer /app/event-consumer

CMD ["/app/event-consumer"]
```

## Step 6: Test Event-Driven Architecture

Rebuild and start everything:

```bash
docker-compose up --build
```

**Create a product:**
```bash
curl -X POST http://localhost:8001/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Event-Driven Mouse",
    "price": 49.99,
    "inventory_count": 100
  }'
```

**Check the logs:**
```bash
docker-compose logs -f event-consumer
```

You should see:
```
ProductCreated: <UUID> - Event-Driven Mouse at 49.99
```

**View in Kafka UI:**

Open http://localhost:8080 and navigate to Topics → product.events → Messages. You'll see the JSON event!

## Step 7: Event Sourcing Pattern (Bonus)

Instead of just publishing events, you can store ALL events as the source of truth. This is called **Event Sourcing**.

Create `shared/src/event_store.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append(
        &self,
        aggregate_id: Uuid,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO events (aggregate_id, event_type, payload, created_at)
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_events(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<StoredEvent>, sqlx::Error> {
        let events = sqlx::query_as(
            r#"
            SELECT id, aggregate_id, event_type, payload, created_at
            FROM events
            WHERE aggregate_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }
}

#[derive(sqlx::FromRow)]
pub struct StoredEvent {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

Create migration for events table:

```sql
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_aggregate_id ON events(aggregate_id);
CREATE INDEX idx_events_created_at ON events(created_at);
```

With Event Sourcing, you can rebuild state from events, have complete audit trail, and enable time travel debugging!

## Key Takeaways

1. **Async messaging**: Decouple services, improve resilience
2. **Kafka topics**: Categories for different event types
3. **Producer/Consumer**: Publish-subscribe pattern
4. **Consumer groups**: Parallel processing
5. **Event-driven**: Services react to events, not direct calls

## When to Use Kafka vs HTTP

| Scenario | Use Kafka | Use HTTP |
|----------|-----------|----------|
| Real-time request/response | ❌ | ✅ |
| Event notification | ✅ | ❌ |
| Audit trail needed | ✅ | ❌ |
| High throughput | ✅ | ❌ |
| Simple CRUD | ❌ | ✅ |
| Multiple consumers | ✅ | ❌ |

## Challenges

1. **Add dead letter queue**: Failed events go to error topic
2. **Add idempotency**: Handle duplicate events
3. **Add event replay**: Re-process old events
4. **Add SAGA pattern**: Distributed transactions with compensating events

## Next Steps

In **Lesson 13**, you'll learn testing strategies: unit tests, integration tests, and contract testing for microservices!

## Official Documentation

- [Apache Kafka](https://kafka.apache.org/documentation/)
- [rdkafka](https://docs.rs/rdkafka/)
- [Event-Driven Architecture](https://martinfowler.com/articles/201701-event-driven.html)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- [SAGA Pattern](https://microservices.io/patterns/data/saga.html)
