# Lesson 28: Event Streaming & Stream Processing

## Overview
Implement advanced event streaming with Kafka including stream processing, event-driven architectures, CQRS with event sourcing, and real-time analytics for RustMart.

## Why This Matters
Event streaming enables:
- **Event-Driven Architecture** - Loosely coupled services
- **Event Sourcing** - Complete audit log, temporal queries
- **Real-Time Processing** - React to events as they happen
- **Scalability** - Process millions of events per second

## Advanced Kafka Producer

```rust
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

struct EventPublisher {
    producer: FutureProducer,
}

impl EventPublisher {
    fn new(brokers: &str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("compression.type", "lz4")
            .set("batch.size", "16384")
            .set("linger.ms", "10")
            .set("enable.idempotence", "true")  // Exactly-once
            .create()
            .expect("Producer creation failed");
        
        Self { producer }
    }
    
    async fn publish_order_event(&self, event: OrderEvent) -> Result<(), Error> {
        let key = event.order_id.to_string();
        let payload = serde_json::to_vec(&event)?;
        
        let record = FutureRecord::to("order-events")
            .key(&key)
            .payload(&payload);
        
        self.producer.send(record, Duration::from_secs(0)).await
            .map_err(|(e, _)| e)?;
        
        Ok(())
    }
}
```

## Event Sourcing Pattern

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum OrderEvent {
    Created { order_id: Uuid, items: Vec<OrderItem>, customer_id: Uuid },
    PaymentProcessed { order_id: Uuid, amount: Decimal, transaction_id: String },
    Shipped { order_id: Uuid, tracking_number: String },
    Delivered { order_id: Uuid, delivered_at: DateTime<Utc> },
    Cancelled { order_id: Uuid, reason: String },
}

// Append events to Kafka (event store)
async fn append_event(publisher: &EventPublisher, event: OrderEvent) -> Result<(), Error> {
    publisher.publish_order_event(event).await
}

// Rebuild state from events
async fn rebuild_order_state(order_id: Uuid, consumer: &StreamConsumer) -> Result<Order, Error> {
    let mut order = Order::default();
    
    // Read all events for this order
    let events = fetch_events_for_order(consumer, order_id).await?;
    
    for event in events {
        order.apply_event(event);
    }
    
    Ok(order)
}

impl Order {
    fn apply_event(&mut self, event: OrderEvent) {
        match event {
            OrderEvent::Created { order_id, items, customer_id } => {
                self.id = order_id;
                self.items = items;
                self.customer_id = customer_id;
                self.status = OrderStatus::Created;
            }
            OrderEvent::PaymentProcessed { transaction_id, .. } => {
                self.status = OrderStatus::Paid;
                self.transaction_id = Some(transaction_id);
            }
            OrderEvent::Shipped { tracking_number, .. } => {
                self.status = OrderStatus::Shipped;
                self.tracking_number = Some(tracking_number);
            }
            // ... handle other events
        }
    }
}
```

## Stream Processing

```rust
use rdkafka::consumer::{Consumer, StreamConsumer};

async fn process_order_events(consumer: StreamConsumer) {
    loop {
        match consumer.recv().await {
            Ok(msg) => {
                let payload = msg.payload().unwrap();
                let event: OrderEvent = serde_json::from_slice(payload)?;
                
                // Process event
                match event {
                    OrderEvent::Created { .. } => update_inventory(&event).await?,
                    OrderEvent::PaymentProcessed { .. } => send_confirmation_email(&event).await?,
                    OrderEvent::Shipped { .. } => notify_customer(&event).await?,
                    _ => {}
                }
                
                // Commit offset
                consumer.commit_message(&msg, CommitMode::Async)?;
            }
            Err(e) => error!("Kafka error: {}", e),
        }
    }
}
```

## Windowed Aggregations

```rust
use std::collections::HashMap;
use chrono::{Duration, Utc};

// Calculate orders per minute (tumbling window)
struct OrderCounter {
    window_size: Duration,
    counts: HashMap<DateTime<Utc>, u64>,
}

impl OrderCounter {
    fn process_event(&mut self, event: OrderEvent) {
        let window_start = self.window_start(event.timestamp());
        *self.counts.entry(window_start).or_insert(0) += 1;
    }
    
    fn window_start(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let mins = timestamp.minute();
        timestamp - Duration::minutes(mins as i64 % 5)  // 5-min windows
    }
}
```

## CQRS with Event Streaming

**Command Side**:
```rust
// Handle commands, emit events
async fn handle_create_order(cmd: CreateOrderCommand) -> Result<(), Error> {
    // Validate
    validate_inventory(&cmd.items).await?;
    
    // Create event
    let event = OrderEvent::Created {
        order_id: Uuid::new_v4(),
        items: cmd.items,
        customer_id: cmd.customer_id,
    };
    
    // Publish to Kafka
    event_publisher.publish(event).await?;
    
    Ok(())
}
```

**Query Side**:
```rust
// Consume events, update read model
async fn update_read_model(event: OrderEvent) -> Result<(), Error> {
    match event {
        OrderEvent::Created { order_id, items, customer_id } => {
            sqlx::query!(
                "INSERT INTO orders_view (id, customer_id, status, total) VALUES ($1, $2, 'created', $3)",
                order_id,
                customer_id,
                calculate_total(&items)
            )
            .execute(&pool)
            .await?;
        }
        // Handle other events...
    }
    Ok(())
}
```

## Dead Letter Queue (DLQ) Pattern

```rust
async fn process_with_dlq(msg: BorrowedMessage<'_>) -> Result<(), Error> {
    match process_message(&msg).await {
        Ok(_) => {
            // Success - commit offset
            consumer.commit_message(&msg, CommitMode::Async)?;
        }
        Err(e) if is_retriable(&e) => {
            // Retry with exponential backoff
            retry_with_backoff(|| process_message(&msg), 3).await?;
        }
        Err(e) => {
            // Unrecoverable - send to DLQ
            send_to_dlq(&msg, &e).await?;
            consumer.commit_message(&msg, CommitMode::Async)?;
        }
    }
    Ok(())
}

async fn send_to_dlq(msg: &BorrowedMessage<'_>, error: &Error) -> Result<(), Error> {
    let dlq_record = FutureRecord::to("order-events-dlq")
        .key(msg.key().unwrap_or(b""))
        .payload(msg.payload().unwrap_or(b""))
        .headers(OwnedHeaders::new()
            .insert(Header::new("error", error.to_string().as_bytes()))
            .insert(Header::new("original_topic", msg.topic().as_bytes())));
    
    dlq_producer.send(dlq_record, Duration::from_secs(0)).await?;
    Ok(())
}
```

## Schema Registry Integration

```rust
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;

struct EventPublisher {
    producer: FutureProducer,
    encoder: AvroEncoder,
}

impl EventPublisher {
    async fn publish_with_schema(&self, event: &OrderEvent) -> Result<(), Error> {
        // Encode with Avro schema
        let encoded = self.encoder
            .encode(
                vec![("order_event", event)],
                SubjectNameStrategy::TopicNameStrategy("order-events".to_string(), false),
            )
            .await?;
        
        let record = FutureRecord::to("order-events")
            .key(&event.order_id.to_string())
            .payload(&encoded);
        
        self.producer.send(record, Duration::from_secs(0)).await?;
        Ok(())
    }
}
```

## Exactly-Once Semantics

```rust
use rdkafka::producer::Producer;

async fn exactly_once_processing() -> Result<(), Error> {
    let consumer = ClientConfig::new()
        .set("group.id", "order-processor")
        .set("enable.auto.commit", "false")
        .set("isolation.level", "read_committed")
        .create()?;
    
    let producer = ClientConfig::new()
        .set("transactional.id", "order-processor-tx")
        .set("enable.idempotence", "true")
        .create()?;
    
    producer.init_transactions(Duration::from_secs(30))?;
    
    loop {
        let msg = consumer.recv().await?;
        
        // Begin transaction
        producer.begin_transaction()?;
        
        // Process message
        let result = process_order_event(&msg).await?;
        
        // Produce result
        producer.send(result).await?;
        
        // Commit offsets within transaction
        producer.send_offsets_to_transaction(
            &consumer.position()?,
            &consumer.group_metadata(),
            Duration::from_secs(30),
        )?;
        
        // Commit transaction
        producer.commit_transaction(Duration::from_secs(30))?;
    }
}
```

## Performance Optimization

### Producer Tuning
```rust
let producer = ClientConfig::new()
    .set("bootstrap.servers", brokers)
    .set("compression.type", "lz4")        // Fast compression
    .set("batch.size", "32768")            // 32KB batches
    .set("linger.ms", "10")                // Wait 10ms to batch
    .set("acks", "all")                    // Wait for all replicas
    .set("max.in.flight.requests.per.connection", "5")
    .create()?;
```

### Consumer Tuning
```rust
let consumer = ClientConfig::new()
    .set("fetch.min.bytes", "1024")        // Min 1KB per fetch
    .set("fetch.max.wait.ms", "500")       // Max 500ms wait
    .set("max.partition.fetch.bytes", "1048576")  // 1MB per partition
    .set("session.timeout.ms", "10000")    // 10s session timeout
    .create()?;
```

## Monitoring Kafka

```rust
use prometheus::{register_counter_vec, register_gauge_vec};

lazy_static! {
    static ref KAFKA_MESSAGES_PRODUCED: CounterVec = register_counter_vec!(
        "kafka_messages_produced_total",
        "Total messages produced",
        &["topic"]
    ).unwrap();
    
    static ref KAFKA_CONSUMER_LAG: GaugeVec = register_gauge_vec!(
        "kafka_consumer_lag",
        "Consumer lag by topic and partition",
        &["topic", "partition"]
    ).unwrap();
}

pub fn record_message_produced(topic: &str) {
    KAFKA_MESSAGES_PRODUCED.with_label_values(&[topic]).inc();
}

pub fn update_consumer_lag(topic: &str, partition: i32, lag: i64) {
    KAFKA_CONSUMER_LAG
        .with_label_values(&[topic, &partition.to_string()])
        .set(lag as f64);
}
```

## Testing Event Streams

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_event_sourcing_rebuild() {
        // Start test Kafka
        let kafka = clients::Cli::default()
            .run(images::kafka::Kafka::default());
        
        let publisher = EventPublisher::new(&kafka.get_host_port());
        
        // Publish events
        let order_id = Uuid::new_v4();
        publisher.publish(OrderEvent::Created {
            order_id,
            items: vec![],
            customer_id: Uuid::new_v4(),
        }).await?;
        
        publisher.publish(OrderEvent::PaymentProcessed {
            order_id,
            amount: Decimal::new(10000, 2),
            transaction_id: "tx_123".to_string(),
        }).await?;
        
        // Rebuild state
        let order = rebuild_order_state(order_id, &consumer).await?;
        
        assert_eq!(order.status, OrderStatus::Paid);
        assert_eq!(order.transaction_id, Some("tx_123".to_string()));
    }
}
```

## Best Practices
- **Design events for immutability** - Events are facts, never change them
- **Use past tense for event names** - OrderCreated, not CreateOrder
- **Version events from day one** - Add version field to all events
- **Implement idempotent consumers** - Handle duplicate messages gracefully
- **Monitor consumer lag** - Alert when lag exceeds thresholds
- **Use compacted topics for state** - Retain only latest value per key
- **Partition by entity ID** - Ensures ordering within entity
- **Test with test containers** - Use Docker for integration tests
- **Plan for schema evolution** - Use Schema Registry with Avro/Protobuf
- **Set up DLQs** - Don't lose unprocessable messages

## Official Documentation
- [Kafka Documentation](https://kafka.apache.org/documentation/)
- [rdkafka Rust Client](https://docs.rs/rdkafka/)
- [Event Sourcing by Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Confluent Schema Registry](https://docs.confluent.io/platform/current/schema-registry/)
- [Designing Event-Driven Systems (Free Book)](https://www.confluent.io/designing-event-driven-systems/)
