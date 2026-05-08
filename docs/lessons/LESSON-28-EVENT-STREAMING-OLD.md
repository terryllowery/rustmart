# Lesson 28: Event Streaming & Stream Processing (Outline)

## Overview
Implement advanced event streaming patterns with Kafka, including stream processing, event-driven architectures, CQRS with event sourcing, and real-time analytics for RustMart.

## Core Topics

### 1. Event Streaming Fundamentals
- Event-driven architecture patterns
- Pub/sub vs event streaming
- Event log as source of truth
- Stream vs batch processing
- Kafka architecture overview

### 2. Advanced Kafka Patterns

#### Topic Design
- Topic naming conventions
- Partitioning strategies
- Replication factor
- Retention policies
- Compacted topics

#### Event Schema Evolution
- Avro schemas with Schema Registry
- Backward/forward compatibility
- Schema versioning
- Breaking change management

### 3. Stream Processing

#### Stateless Transformations
```rust
// Map, filter, flat_map operations
async fn process_order_events(consumer: StreamConsumer) {
    let stream = consumer.stream();
    
    stream
        .filter(|msg| is_valid_order(msg))
        .map(|msg| enrich_with_customer_data(msg))
        .for_each(|msg| publish_enriched_event(msg))
        .await;
}
```

#### Stateful Operations
- Aggregations
- Windowing (tumbling, sliding, session)
- Joins (stream-stream, stream-table)
- State stores

### 4. Kafka Streams in Rust

#### rdkafka Advanced Usage
```rust
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::FutureProducer;

// Exactly-once semantics
let consumer = ClientConfig::new()
    .set("enable.idempotence", "true")
    .set("isolation.level", "read_committed")
    .create()?;
```

#### Custom Partitioners
- Partition by key
- Custom partition logic
- Load balancing

#### Offset Management
- Manual offset commits
- Exactly-once processing
- Checkpoint strategies

### 5. Event Sourcing Patterns

#### Event Store Implementation
```rust
#[derive(Serialize, Deserialize)]
enum OrderEvent {
    Created { order_id: Uuid, items: Vec<Item> },
    PaymentProcessed { order_id: Uuid, amount: Decimal },
    Shipped { order_id: Uuid, tracking: String },
    Delivered { order_id: Uuid, timestamp: DateTime },
}

async fn append_event(event: OrderEvent) -> Result<()> {
    producer.send(
        FutureRecord::to("order-events")
            .key(&event.order_id().to_string())
            .payload(&serde_json::to_vec(&event)?)
    ).await?;
    Ok(())
}
```

#### Event Replay
- Rebuild state from events
- Point-in-time recovery
- Debugging with event history

#### Snapshots
- Periodic state snapshots
- Fast bootstrap
- Snapshot compaction

### 6. CQRS with Event Streaming

#### Command Side
- Handle commands
- Validate business rules
- Emit events
- Write to event store

#### Query Side (Read Models)
- Event consumers build read models
- Materialized views in database
- Optimized for queries
- Eventually consistent

#### Projection Updates
```rust
async fn update_order_projection(event: OrderEvent) {
    match event {
        OrderEvent::Created { order_id, items } => {
            sqlx::query("INSERT INTO orders_view ...")
                .execute(&pool)
                .await?;
        }
        OrderEvent::Shipped { order_id, tracking } => {
            sqlx::query("UPDATE orders_view SET status = 'shipped' ...")
                .execute(&pool)
                .await?;
        }
    }
}
```

### 7. Real-time Analytics

#### Stream Aggregations
- Count events per window
- Sum order values
- Moving averages
- Top-K products

#### Time Windows
```rust
// Tumbling window: 5-minute aggregates
// Sliding window: Rolling 1-hour sum
// Session window: User activity sessions
```

#### Materialized Views
- Real-time dashboards
- Business intelligence
- Operational metrics

### 8. Change Data Capture (CDC)

#### Debezium Integration
- Capture database changes
- Postgres WAL streaming
- MySQL binlog
- Event-driven from database

#### CDC Patterns
- Sync databases across services
- Invalidate caches on change
- Trigger downstream workflows
- Audit logging

### 9. Dead Letter Queues

#### Error Handling Strategy
```rust
async fn process_with_dlq(msg: BorrowedMessage) {
    match process_message(&msg).await {
        Ok(_) => commit_offset(&msg),
        Err(e) if is_retriable(&e) => {
            // Retry with backoff
        }
        Err(e) => {
            send_to_dlq(&msg, e).await;
            commit_offset(&msg);
        }
    }
}
```

#### DLQ Processing
- Manual review
- Automated retry
- Alert on DLQ buildup

### 10. Performance & Scalability

#### Producer Optimization
- Batch size tuning
- Compression (lz4, snappy, gzip)
- Async sends
- Connection pooling

#### Consumer Optimization
- Partition assignment strategies
- Parallel processing
- Backpressure handling
- Fetch size tuning

#### Monitoring Kafka
- Consumer lag monitoring
- Partition rebalancing
- Throughput metrics
- Error rates

### 11. Testing Event Streams

#### Unit Tests
- Event serialization/deserialization
- Business logic in event handlers
- State transitions

#### Integration Tests
```rust
#[tokio::test]
async fn test_order_event_processing() {
    let test_kafka = TestKafka::start().await;
    
    // Produce test event
    produce_event(&test_kafka, OrderEvent::Created { ... }).await;
    
    // Consume and verify
    let consumed = consume_events(&test_kafka, 1).await;
    assert_eq!(consumed.len(), 1);
}
```

#### Contract Tests
- Schema compatibility
- Event format validation
- Consumer expectations

### 12. Multi-Cluster & Cross-Region

#### Kafka MirrorMaker
- Replicate topics across clusters
- Active-active setup
- Disaster recovery

#### Global Event Streams
- Cross-region replication
- Latency considerations
- Conflict resolution

## Tools & Libraries

- **rdkafka**: Rust Kafka client (librdkafka bindings)
- **kafka-delta-ingest**: Stream to Delta Lake
- **Schema Registry**: Confluent Schema Registry
- **Debezium**: Change data capture
- **Kafdrop**: Kafka UI for monitoring
- **Kafka Connect**: Integration framework

## Hands-on Exercises

1. Implement event sourcing for order management
2. Build CQRS with separate read/write models
3. Create stream processing pipeline with windowed aggregations
4. Set up CDC from PostgreSQL to Kafka
5. Implement dead letter queue handling
6. Build real-time analytics dashboard
7. Test exactly-once semantics with failures

## Best Practices

- Design events for immutability
- Use semantic event names (past tense)
- Version events from the start
- Implement idempotent consumers
- Monitor consumer lag closely
- Use compacted topics for state
- Partition by entity ID for ordering
- Test failure scenarios thoroughly
- Document event schemas
- Plan for schema evolution

## Resources

- [Kafka: The Definitive Guide](https://www.confluent.io/resources/kafka-the-definitive-guide/)
- [Event Sourcing by Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Kafka Documentation](https://kafka.apache.org/documentation/)
- [rdkafka Documentation](https://docs.rs/rdkafka/)
- [Designing Event-Driven Systems](https://www.confluent.io/designing-event-driven-systems/)
