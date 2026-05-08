# Lesson 19: Performance Profiling and Optimization

## Overview
Learn to identify performance bottlenecks and optimize Rust code for production. This lesson covers CPU profiling, memory profiling, async runtime analysis, and real-world optimization techniques that make great demo content for your IBM presentations.

By the end of this lesson, you'll have:
- CPU profiling with flamegraphs
- Memory profiling and leak detection
- Async runtime profiling with tokio-console
- Database query optimization
- Before/after metrics for demos

## Why Performance Profiling Matters

**For your career:**
- Production systems require optimization
- Shows deep technical expertise (Band 10 level)
- Great demo content (before/after improvements)
- SRE teams value performance work
- Directly applicable to Tiger Team work at IBM

**Impact:**
- 10x performance improvements are common
- Reduced infrastructure costs
- Better user experience
- Scalability for growth

## The Performance Optimization Loop

```
1. Measure (baseline)
   ↓
2. Profile (find bottleneck)
   ↓
3. Hypothesize (what's slow?)
   ↓
4. Optimize (make changes)
   ↓
5. Measure (verify improvement)
   ↓
Back to step 2 (iterate)
```

**Rule #1:** Always measure first. Never guess.

## Step 1: Establish a Baseline

Before optimizing, measure current performance.

Create `benchmarks/baseline.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustmart_load_tester::scenarios::TestScenario;

fn benchmark_endpoints(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let scenario = TestScenario::new("http://localhost:8001".to_string());
    
    let mut group = c.benchmark_group("api_endpoints");
    
    group.bench_function("get_all_products", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(scenario.browse_products().await)
        });
    });
    
    group.bench_function("create_product", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(scenario.create_product().await)
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_endpoints);
criterion_main!(benches);
```

Run baseline:

```bash
cargo bench --bench baseline > baseline_results.txt
```

## Step 2: CPU Profiling with Flamegraphs

Install profiling tools:

```bash
# macOS
brew install flamegraph

# Or use cargo-flamegraph
cargo install flamegraph
```

### Generate Flamegraph

```bash
# Profile the running service
sudo cargo flamegraph --bin product-service

# This creates flamegraph.svg
open flamegraph.svg
```

**Reading a Flamegraph:**
- **X-axis**: Alphabetical order (NOT time!)
- **Y-axis**: Stack depth
- **Width**: Time spent in function
- **Colors**: Random (for visual distinction)

**Look for:**
- Wide bars = hot functions
- Plateaus = optimization opportunities

### Using perf (Linux) or Instruments (macOS)

**macOS with Instruments:**

```bash
# Build with debug symbols
cargo build --release --bin product-service

# Run with Instruments
instruments -t "Time Profiler" ./target/release/product-service
```

**Linux with perf:**

```bash
# Record profile
sudo perf record --call-graph=dwarf ./target/release/product-service

# Generate report
sudo perf report

# Generate flamegraph
sudo perf script | stackcollapse-perf.pl | flamegraph.pl > perf-flamegraph.svg
```

## Step 3: Finding Hot Paths

Let's profile the product service under load:

```bash
# Terminal 1: Start service with profiling
sudo cargo flamegraph --bin product-service

# Terminal 2: Generate load
./load-tester/target/release/load-test --profile heavy --duration 60

# Terminal 3: Stop profiling (Ctrl+C in Terminal 1)
```

**Common hotspots in web services:**
1. **JSON serialization/deserialization** - Use simd-json or consider binary formats
2. **Database queries** - Add indexes, optimize queries
3. **String allocations** - Use `&str` instead of `String` where possible
4. **Mutex contention** - Use RwLock or lock-free structures
5. **Cloning** - Use `Arc` or references

## Step 4: Memory Profiling

### Using valgrind (massif)

```bash
# Install valgrind
brew install valgrind  # macOS
# OR
sudo apt install valgrind  # Linux

# Run with memory profiling
valgrind --tool=massif --massif-out-file=massif.out \
  ./target/release/product-service

# Visualize
ms_print massif.out > memory_profile.txt
```

### Using heaptrack

```bash
# Linux only
sudo apt install heaptrack

# Profile
heaptrack ./target/release/product-service

# Analyze
heaptrack_gui heaptrack.product-service.*.gz
```

### Memory Leak Detection

```rust
// Add to Cargo.toml
[dependencies]
tracing-tracy = "0.10"

// In main.rs
use tracing_tracy::TracyLayer;

tracing_subscriber::registry()
    .with(TracyLayer::new())
    .init();
```

Then use Tracy Profiler to visualize allocations.

## Step 5: Async Runtime Profiling with tokio-console

Install tokio-console:

```bash
cargo install tokio-console
```

Update `product-service/Cargo.toml`:

```toml
[dependencies]
tokio = { workspace = true, features = ["full", "tracing"] }
console-subscriber = "0.2"
```

Update `product-service/src/main.rs`:

```rust
#[tokio::main]
async fn main() {
    // Initialize console subscriber
    console_subscriber::init();
    
    // Rest of your code...
}
```

Run with console:

```bash
# Terminal 1: Start service
RUSTFLAGS="--cfg tokio_unstable" cargo run -p product-service

# Terminal 2: Open console
tokio-console

# Terminal 3: Generate load
./scripts/demo-load.sh
```

**What tokio-console shows:**
- **Tasks**: All async tasks running
- **Resources**: Locks, channels, etc.
- **Blocked time**: Tasks waiting on I/O
- **Busy time**: Tasks doing CPU work

**Look for:**
- Tasks blocked for too long
- Too many spawned tasks
- Mutex/RwLock contention
- Unbounded channel growth

## Step 6: Database Query Optimization

### Using EXPLAIN ANALYZE

```sql
-- In psql
EXPLAIN ANALYZE SELECT * FROM products WHERE price > 100;

-- Look for:
-- - Seq Scan (bad) vs Index Scan (good)
-- - High cost values
-- - Long execution time
```

### Add Strategic Indexes

```sql
-- Create index on commonly filtered columns
CREATE INDEX idx_products_price ON products(price);

-- Composite index for complex queries
CREATE INDEX idx_products_price_inventory 
  ON products(price, inventory_count);

-- Verify index is used
EXPLAIN SELECT * FROM products WHERE price > 100;
-- Should show "Index Scan" now
```

### Query Optimization in Rust

```rust
// ❌ BAD: N+1 query problem
async fn get_products_with_details_slow() -> Vec<ProductWithDetails> {
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(&pool)
        .await?;
    
    let mut result = vec![];
    for product in products {
        // This runs N queries!
        let details = sqlx::query_as::<_, Details>(
            "SELECT * FROM details WHERE product_id = $1"
        )
        .bind(product.id)
        .fetch_one(&pool)
        .await?;
        
        result.push(ProductWithDetails { product, details });
    }
    
    result
}

// ✅ GOOD: Single JOIN query
async fn get_products_with_details_fast() -> Vec<ProductWithDetails> {
    sqlx::query_as::<_, ProductWithDetails>(
        "SELECT p.*, d.* FROM products p 
         JOIN details d ON d.product_id = p.id"
    )
    .fetch_all(&pool)
    .await?
}
```

### Connection Pool Tuning

```rust
// Default pool is too small for high load
let pool = PgPoolOptions::new()
    .max_connections(5)  // ❌ Too small!
    .connect(&database_url)
    .await?;

// Better for production
let pool = PgPoolOptions::new()
    .max_connections(20)          // More connections
    .min_connections(5)            // Keep warm connections
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

## Step 7: Real-World Optimizations

### 1. Reduce Allocations

```rust
// ❌ SLOW: Creates many strings
fn format_products_slow(products: &[Product]) -> Vec<String> {
    products.iter()
        .map(|p| format!("{}: ${}", p.name, p.price))
        .collect()
}

// ✅ FAST: Pre-allocate and reuse buffer
fn format_products_fast(products: &[Product]) -> String {
    let mut buf = String::with_capacity(products.len() * 50);
    
    for p in products {
        use std::fmt::Write;
        writeln!(buf, "{}: ${}", p.name, p.price).unwrap();
    }
    
    buf
}
```

### 2. Use Cow for Conditional Cloning

```rust
use std::borrow::Cow;

fn process_name(name: &str) -> Cow<str> {
    if name.contains("special") {
        // Only clone if modification needed
        Cow::Owned(name.to_uppercase())
    } else {
        // No allocation, just borrow
        Cow::Borrowed(name)
    }
}
```

### 3. Lazy Static for Expensive Initialization

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// ❌ SLOW: Compile regex on every call
fn validate_email_slow(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

// ✅ FAST: Compile once, reuse forever
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

fn validate_email_fast(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}
```

### 4. Batch Operations

```rust
// ❌ SLOW: Insert one at a time
async fn insert_products_slow(products: Vec<Product>) {
    for product in products {
        sqlx::query("INSERT INTO products (name, price) VALUES ($1, $2)")
            .bind(product.name)
            .bind(product.price)
            .execute(&pool)
            .await?;
    }
}

// ✅ FAST: Batch insert
async fn insert_products_fast(products: Vec<Product>) {
    let mut query = String::from("INSERT INTO products (name, price) VALUES ");
    
    // Build values string
    for (i, _) in products.iter().enumerate() {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(&format!("(${}, ${})", i * 2 + 1, i * 2 + 2));
    }
    
    let mut q = sqlx::query(&query);
    for product in products {
        q = q.bind(product.name).bind(product.price);
    }
    
    q.execute(&pool).await?;
}
```

### 5. Use simd-json for Faster Parsing

```toml
[dependencies]
simd-json = "0.13"
```

```rust
// 2-5x faster than serde_json
use simd_json;

let mut data = json_string.as_bytes().to_vec();
let product: Product = simd_json::from_slice(&mut data)?;
```

## Step 8: Benchmark Your Changes

After each optimization:

```bash
# Run benchmark again
cargo bench --bench baseline > optimized_results.txt

# Compare results
cargo benchcmp baseline_results.txt optimized_results.txt
```

Example output:
```
name                    baseline ns/iter  optimized ns/iter  diff ns/iter   diff %
get_all_products        1,234,567         456,789           -777,778       -63.0%
create_product          890,123           345,678           -544,445       -61.2%
```

## Step 9: Production Profiling (Non-intrusive)

### Using pprof in Production

Add to `Cargo.toml`:

```toml
[dependencies]
pprof = { version = "0.13", features = ["flamegraph", "protobuf-codec"] }
```

Add profiling endpoint:

```rust
use pprof::ProfilerGuard;

async fn profiling_handler() -> impl IntoResponse {
    let guard = ProfilerGuard::new(100).unwrap();
    
    // Profile for 30 seconds
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    if let Ok(report) = guard.report().build() {
        let mut body = Vec::new();
        report.flamegraph(&mut body).unwrap();
        
        return (
            [(header::CONTENT_TYPE, "image/svg+xml")],
            body
        );
    }
    
    (StatusCode::INTERNAL_SERVER_ERROR, vec![])
}

// Add to router
.route("/debug/profile", get(profiling_handler))
```

Access: `http://localhost:8001/debug/profile` (save and open the SVG)

## Step 10: Optimization Checklist

### Before Demo:

```bash
# 1. Baseline metrics
cargo bench > before.txt

# 2. Profile CPU
sudo cargo flamegraph --bin product-service &
./scripts/demo-load.sh

# 3. Check memory
heaptrack ./target/release/product-service

# 4. Analyze async runtime
RUSTFLAGS="--cfg tokio_unstable" cargo run -p product-service &
tokio-console

# 5. Database queries
psql $DATABASE_URL
\d products  -- Check indexes
EXPLAIN ANALYZE SELECT ...;

# 6. Optimize (make changes)

# 7. Verify improvement
cargo bench > after.txt
cargo benchcmp before.txt after.txt
```

## Common Performance Wins

| Optimization | Typical Speedup | Difficulty |
|--------------|----------------|------------|
| Add database index | 10-100x | Easy |
| Batch operations | 5-50x | Easy |
| Reduce cloning | 2-10x | Medium |
| Use binary format (not JSON) | 2-5x | Medium |
| Connection pooling | 5-20x | Easy |
| Cache frequently accessed data | 10-1000x | Medium |
| Use lazy_static for regex | 100x+ | Easy |
| SIMD JSON parsing | 2-5x | Easy |
| Reduce allocations | 2-10x | Hard |
| Lock-free data structures | 2-10x | Hard |

## Demo Script for IBM Presentation

```markdown
# "10x Performance Improvement in Production"

## Before (Show flamegraph with hotspot)
"Here's RustMart under load - this wide bar shows 60% of time is spent here"

## Hypothesis
"Looking at the code, we're doing N database queries in a loop"

## Fix (Show the code change)
"Changed from N queries to a single JOIN"

## After (Show new flamegraph)
"That hotspot is gone - now evenly distributed"

## Metrics (Show benchmark comparison)
"Latency dropped from 450ms to 45ms - 10x improvement"
"Throughput increased from 50 RPS to 500 RPS"

## Impact
"This allows us to handle 10x more customers with same hardware"
"Reduced AWS costs by $X/month"
```

## Key Takeaways

1. **Always measure first** - Don't optimize without profiling
2. **Focus on hot paths** - 80/20 rule applies
3. **Database is often the bottleneck** - Index and optimize queries
4. **Allocations are expensive** - Reduce unnecessary clones
5. **Async != fast** - Profile the runtime
6. **Benchmark your changes** - Verify improvements

## Challenges

1. **Optimize JSON serialization**: Switch to simd-json and measure speedup
2. **Find memory leak**: Add memory profiling and track down a leak
3. **Fix N+1 query**: Identify and fix in your codebase
4. **Reduce allocations**: Profile and eliminate unnecessary String clones
5. **Async profiling**: Use tokio-console to find blocked tasks

## Next Steps

In **Lesson 20**, you'll learn security hardening to protect your optimized system from attacks!

## Official Documentation

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [tokio-console](https://github.com/tokio-rs/console)
- [criterion](https://docs.rs/criterion/)
- [perf Examples](https://www.brendangregg.com/perf.html)

---

**Performance optimization is a superpower!** Show these skills in your IBM demos and you'll stand out as someone who understands production systems at a deep level. 🚀
