# Lesson 17: Building a Load Testing Tool in Rust

## Overview
Build a custom Rust load testing tool to stress-test RustMart and demonstrate observability in action. This tool will generate realistic traffic patterns, simulate errors, and help you showcase distributed tracing, metrics, and alerting in your IBM Instana demo.

By the end of this lesson, you'll have:
- A Rust CLI load testing tool
- Configurable load profiles (light, medium, heavy)
- Error injection capabilities
- Real-time statistics and reporting
- Traffic patterns that create interesting traces

## Why Build a Custom Load Testing Tool?

**Commercial tools** (k6, JMeter, Gatling) are great, but a custom Rust tool gives you:
- **Demo control**: Precisely trigger scenarios that showcase observability
- **Error injection**: Simulate failures to demonstrate alerting
- **Rust practice**: Real-world async Rust with tokio
- **Customization**: Add RustMart-specific test scenarios
- **Performance**: Rust handles massive concurrency efficiently

## Step 1: Create the Load Tester Project

```bash
cd ~/code/rustmart
mkdir load-tester
cd load-tester
cargo init --name rustmart-load-tester
```

Update `Cargo.toml`:

```toml
[package]
name = "rustmart-load-tester"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "load-test"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.4", features = ["derive"] }
uuid = { version = "1.6", features = ["v4"] }
colored = "2.1"
indicatif = "0.17"
rand = "0.8"
tokio-metrics = "0.3"
hdrhistogram = "7.5"

[dev-dependencies]
```

## Step 2: Define Load Profiles

Create `load-tester/src/config.rs`:

```rust
use clap::{Parser, ValueEnum};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "RustMart Load Tester")]
#[command(about = "Load testing tool for RustMart microservices", long_about = None)]
pub struct Config {
    /// Load profile to use
    #[arg(short, long, value_enum, default_value = "medium")]
    pub profile: LoadProfile,

    /// Target service URL
    #[arg(short, long, default_value = "http://localhost:8001")]
    pub url: String,

    /// Duration of the test in seconds
    #[arg(short, long, default_value = "60")]
    pub duration: u64,

    /// Error rate (percentage of requests that should fail)
    #[arg(short, long, default_value = "0")]
    pub error_rate: u8,

    /// Enable chaos mode (random failures and delays)
    #[arg(long, default_value = "false")]
    pub chaos: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "10")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LoadProfile {
    /// Light load: 10 RPS, 5 concurrent users
    Light,
    /// Medium load: 50 RPS, 20 concurrent users
    Medium,
    /// Heavy load: 200 RPS, 100 concurrent users
    Heavy,
    /// Stress test: 500+ RPS, 200 concurrent users
    Stress,
}

impl LoadProfile {
    pub fn requests_per_second(&self) -> usize {
        match self {
            LoadProfile::Light => 10,
            LoadProfile::Medium => 50,
            LoadProfile::Heavy => 200,
            LoadProfile::Stress => 500,
        }
    }

    pub fn concurrent_users(&self) -> usize {
        match self {
            LoadProfile::Light => 5,
            LoadProfile::Medium => 20,
            LoadProfile::Heavy => 100,
            LoadProfile::Stress => 200,
        }
    }

    pub fn think_time(&self) -> Duration {
        match self {
            LoadProfile::Light => Duration::from_millis(1000),
            LoadProfile::Medium => Duration::from_millis(500),
            LoadProfile::Heavy => Duration::from_millis(100),
            LoadProfile::Stress => Duration::from_millis(10),
        }
    }
}
```

## Step 3: Create Test Scenarios

Create `load-tester/src/scenarios.rs`:

```rust
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

pub struct TestScenario {
    client: Client,
    base_url: String,
}

#[derive(Debug)]
pub struct RequestResult {
    pub scenario: String,
    pub duration: std::time::Duration,
    pub status: u16,
    pub success: bool,
}

impl TestScenario {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self { client, base_url }
    }

    /// Browse products (read-heavy)
    pub async fn browse_products(&self) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}/products", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = response.status().is_success();
                RequestResult {
                    scenario: "browse_products".to_string(),
                    duration: start.elapsed(),
                    status,
                    success,
                }
            }
            Err(_) => RequestResult {
                scenario: "browse_products".to_string(),
                duration: start.elapsed(),
                status: 0,
                success: false,
            },
        }
    }

    /// Get specific product by ID
    pub async fn get_product_by_id(&self, product_id: Uuid) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}/products/{}", self.base_url, product_id);

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = response.status().is_success();
                RequestResult {
                    scenario: "get_product".to_string(),
                    duration: start.elapsed(),
                    status,
                    success,
                }
            }
            Err(_) => RequestResult {
                scenario: "get_product".to_string(),
                duration: start.elapsed(),
                status: 0,
                success: false,
            },
        }
    }

    /// Create a new product (write operation)
    pub async fn create_product(&self) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}/products", self.base_url);

        let mut rng = rand::thread_rng();
        let product = json!({
            "name": format!("LoadTest-Product-{}", Uuid::new_v4()),
            "price": rng.gen_range(10.0..1000.0),
            "inventory_count": rng.gen_range(1..500),
        });

        match self.client.post(&url).json(&product).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let success = response.status().is_success();
                RequestResult {
                    scenario: "create_product".to_string(),
                    duration: start.elapsed(),
                    status,
                    success,
                }
            }
            Err(_) => RequestResult {
                scenario: "create_product".to_string(),
                duration: start.elapsed(),
                status: 0,
                success: false,
            },
        }
    }

    /// Get non-existent product (trigger 404)
    pub async fn get_missing_product(&self) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}/products/{}", self.base_url, Uuid::nil());

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                // 404 is expected, so this is a "success"
                RequestResult {
                    scenario: "not_found_error".to_string(),
                    duration: start.elapsed(),
                    status,
                    success: true,
                }
            }
            Err(_) => RequestResult {
                scenario: "not_found_error".to_string(),
                duration: start.elapsed(),
                status: 0,
                success: false,
            },
        }
    }

    /// Random scenario selection
    pub async fn random_scenario(&self, error_rate: u8) -> RequestResult {
        let mut rng = rand::thread_rng();
        
        // Inject errors based on error_rate
        if rng.gen_range(0..100) < error_rate {
            return self.get_missing_product().await;
        }

        // Normal traffic distribution
        match rng.gen_range(0..100) {
            0..=70 => self.browse_products().await,      // 70% browse
            71..=90 => self.get_product_by_id(Uuid::new_v4()).await, // 20% get specific
            _ => self.create_product().await,             // 10% create
        }
    }
}
```

## Step 4: Statistics Tracking

Create `load-tester/src/stats.rs`:

```rust
use crate::scenarios::RequestResult;
use colored::*;
use hdrhistogram::Histogram;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct Statistics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    latencies: Histogram<u64>,
    status_codes: HashMap<u16, u64>,
    scenario_counts: HashMap<String, u64>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            latencies: Histogram::<u64>::new(3).unwrap(),
            status_codes: HashMap::new(),
            scenario_counts: HashMap::new(),
        }
    }

    pub fn record_request(&mut self, result: RequestResult) {
        self.total_requests += 1;

        if result.success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        let latency_ms = result.duration.as_millis() as u64;
        let _ = self.latencies.record(latency_ms);

        *self.status_codes.entry(result.status).or_insert(0) += 1;
        *self.scenario_counts.entry(result.scenario).or_insert(0) += 1;
    }

    pub fn print_summary(&self, duration: Duration) {
        println!("\n{}", "=".repeat(60).bright_cyan());
        println!("{}", "Load Test Summary".bright_cyan().bold());
        println!("{}", "=".repeat(60).bright_cyan());

        let duration_secs = duration.as_secs_f64();
        let rps = self.total_requests as f64 / duration_secs;

        println!("\n{}", "Request Statistics:".bright_yellow().bold());
        println!("  Total Requests:     {}", self.total_requests.to_string().bright_white());
        println!("  Successful:         {} ({}%)", 
            self.successful_requests.to_string().green(),
            format!("{:.2}", (self.successful_requests as f64 / self.total_requests as f64) * 100.0).green()
        );
        println!("  Failed:             {} ({}%)", 
            self.failed_requests.to_string().red(),
            format!("{:.2}", (self.failed_requests as f64 / self.total_requests as f64) * 100.0).red()
        );
        println!("  Requests/sec:       {}", format!("{:.2}", rps).bright_white());
        println!("  Test Duration:      {}s", format!("{:.2}", duration_secs).bright_white());

        println!("\n{}", "Latency Statistics (ms):".bright_yellow().bold());
        println!("  Min:                {}", self.latencies.min().to_string().bright_white());
        println!("  Mean:               {}", format!("{:.2}", self.latencies.mean()).bright_white());
        println!("  P50:                {}", self.latencies.value_at_quantile(0.50).to_string().bright_white());
        println!("  P90:                {}", self.latencies.value_at_quantile(0.90).to_string().yellow());
        println!("  P95:                {}", self.latencies.value_at_quantile(0.95).to_string().yellow());
        println!("  P99:                {}", self.latencies.value_at_quantile(0.99).to_string().red());
        println!("  Max:                {}", self.latencies.max().to_string().red());

        println!("\n{}", "Status Code Distribution:".bright_yellow().bold());
        let mut status_vec: Vec<_> = self.status_codes.iter().collect();
        status_vec.sort_by_key(|k| k.0);
        for (status, count) in status_vec {
            let status_str = format!("  {}", status);
            let colored_status = if *status >= 200 && *status < 300 {
                status_str.green()
            } else if *status >= 400 && *status < 500 {
                status_str.yellow()
            } else if *status >= 500 {
                status_str.red()
            } else {
                status_str.white()
            };
            println!("{}:              {}", colored_status, count);
        }

        println!("\n{}", "Scenario Distribution:".bright_yellow().bold());
        for (scenario, count) in &self.scenario_counts {
            let percentage = (*count as f64 / self.total_requests as f64) * 100.0;
            println!("  {:<20} {} ({}%)", 
                scenario.bright_white(), 
                count, 
                format!("{:.1}", percentage).cyan()
            );
        }

        println!("\n{}", "=".repeat(60).bright_cyan());
    }
}

pub type SharedStats = Arc<Mutex<Statistics>>;

pub fn create_shared_stats() -> SharedStats {
    Arc::new(Mutex::new(Statistics::new()))
}
```

## Step 5: Main Load Tester Logic

Create `load-tester/src/main.rs`:

```rust
mod config;
mod scenarios;
mod stats;

use clap::Parser;
use colored::*;
use config::{Config, LoadProfile};
use indicatif::{ProgressBar, ProgressStyle};
use scenarios::TestScenario;
use stats::{create_shared_stats, SharedStats};
use std::time::{Duration, Instant};
use tokio::time::sleep;

async fn run_user_session(
    scenario: TestScenario,
    stats: SharedStats,
    profile: LoadProfile,
    error_rate: u8,
    duration: Duration,
) {
    let start = Instant::now();

    while start.elapsed() < duration {
        // Execute random scenario
        let result = scenario.random_scenario(error_rate).await;

        // Record statistics
        {
            let mut stats_lock = stats.lock().unwrap();
            stats_lock.record_request(result);
        }

        // Think time between requests
        sleep(profile.think_time()).await;
    }
}

async fn run_load_test(config: Config) {
    let stats = create_shared_stats();
    let duration = Duration::from_secs(config.duration);
    let concurrent_users = config.profile.concurrent_users();

    println!("{}", "RustMart Load Tester".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_cyan());
    println!("  Target URL:         {}", config.url.bright_white());
    println!("  Load Profile:       {:?}", config.profile);
    println!("  Concurrent Users:   {}", concurrent_users);
    println!("  Target RPS:         ~{}", config.profile.requests_per_second());
    println!("  Duration:           {}s", config.duration);
    println!("  Error Rate:         {}%", config.error_rate);
    println!("  Chaos Mode:         {}", if config.chaos { "ENABLED".red() } else { "disabled".white() });
    println!("{}", "=".repeat(60).bright_cyan());
    println!();

    // Health check
    let test_scenario = TestScenario::new(config.url.clone());
    print!("Checking service health... ");
    match reqwest::get(format!("{}/health", config.url)).await {
        Ok(response) if response.status().is_success() => {
            println!("{}", "✓ Service is healthy".green());
        }
        _ => {
            println!("{}", "✗ Service is not responding".red());
            println!("{}", "Please start the service before running load tests.".yellow());
            return;
        }
    }

    println!("\n{}", "Starting load test...".bright_green().bold());

    // Progress bar
    let pb = ProgressBar::new(config.duration);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}s ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Spawn concurrent user sessions
    let mut handles = vec![];
    for _ in 0..concurrent_users {
        let scenario = TestScenario::new(config.url.clone());
        let stats_clone = stats.clone();
        let profile = config.profile;
        let error_rate = config.error_rate;

        let handle = tokio::spawn(async move {
            run_user_session(scenario, stats_clone, profile, error_rate, duration).await;
        });

        handles.push(handle);
    }

    // Progress bar updater
    let pb_clone = pb.clone();
    let progress_handle = tokio::spawn(async move {
        let start = Instant::now();
        while start.elapsed() < duration {
            pb_clone.set_position(start.elapsed().as_secs());
            sleep(Duration::from_millis(100)).await;
        }
        pb_clone.finish();
    });

    // Wait for all user sessions to complete
    for handle in handles {
        let _ = handle.await;
    }

    let _ = progress_handle.await;

    println!("\n{}", "Load test completed!".bright_green().bold());

    // Print statistics
    let stats_lock = stats.lock().unwrap();
    stats_lock.print_summary(duration);
}

#[tokio::main]
async fn main() {
    let config = Config::parse();
    run_load_test(config).await;
}
```

## Step 6: Build and Run the Load Tester

Build the tool:
```bash
cd ~/code/rustmart/load-tester
cargo build --release
```

Run different load profiles:

```bash
# Light load - good for development
./target/release/load-test --profile light --duration 30

# Medium load - typical usage
./target/release/load-test --profile medium --duration 60

# Heavy load - stress test
./target/release/load-test --profile heavy --duration 120

# Stress test with errors
./target/release/load-test --profile stress --error-rate 10 --duration 60
```

## Step 7: Create Demo Scenarios

Create `load-tester/src/demo_scenarios.rs`:

```rust
use crate::config::Config;
use colored::*;
use std::time::Duration;
use tokio::time::sleep;

/// Demo scenario that gradually increases load
pub async fn ramp_up_demo(base_config: Config) {
    println!("\n{}", "Demo: Gradual Ramp-Up".bright_cyan().bold());
    println!("{}", "This simulates traffic gradually increasing\n".white());

    let scenarios = vec![
        ("Baseline", "light", 30),
        ("Morning Rush", "medium", 45),
        ("Peak Hours", "heavy", 60),
        ("System Stress", "stress", 30),
    ];

    for (name, profile, duration) in scenarios {
        println!("\n{} Starting: {} ({}s)", "→".bright_green(), name.bright_yellow(), duration);
        
        let mut config = base_config.clone();
        config.profile = match profile {
            "light" => crate::config::LoadProfile::Light,
            "medium" => crate::config::LoadProfile::Medium,
            "heavy" => crate::config::LoadProfile::Heavy,
            "stress" => crate::config::LoadProfile::Stress,
            _ => crate::config::LoadProfile::Medium,
        };
        config.duration = duration;

        super::run_load_test(config).await;
        
        println!("\n{} Cooling down...", "⏸".bright_blue());
        sleep(Duration::from_secs(10)).await;
    }

    println!("\n{}", "Ramp-up demo complete!".bright_green().bold());
}

/// Demo scenario that injects errors to trigger alerts
pub async fn error_injection_demo(mut config: Config) {
    println!("\n{}", "Demo: Error Injection".bright_cyan().bold());
    println!("{}", "This will trigger errors to demonstrate alerting\n".white());

    // Start with normal traffic
    println!("{} Phase 1: Normal traffic (30s)", "→".bright_green());
    config.error_rate = 0;
    config.duration = 30;
    super::run_load_test(config.clone()).await;

    sleep(Duration::from_secs(5)).await;

    // Inject 10% errors
    println!("\n{} Phase 2: 10% error rate (30s)", "→".bright_yellow());
    config.error_rate = 10;
    config.duration = 30;
    super::run_load_test(config.clone()).await;

    sleep(Duration::from_secs(5)).await;

    // Inject 25% errors (should trigger alerts)
    println!("\n{} Phase 3: 25% error rate - ALERT! (30s)", "→".bright_red());
    config.error_rate = 25;
    config.duration = 30;
    super::run_load_test(config.clone()).await;

    sleep(Duration::from_secs(5)).await;

    // Recovery
    println!("\n{} Phase 4: Recovery - normal traffic (30s)", "→".bright_green());
    config.error_rate = 0;
    config.duration = 30;
    super::run_load_test(config.clone()).await;

    println!("\n{}", "Error injection demo complete!".bright_green().bold());
}
```

Add to main.rs:

```rust
#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    config: Config,

    /// Run demo scenario
    #[arg(long, value_enum)]
    demo: Option<DemoScenario>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DemoScenario {
    RampUp,
    ErrorInjection,
}

// In main():
match cli.demo {
    Some(DemoScenario::RampUp) => {
        demo_scenarios::ramp_up_demo(cli.config).await;
    }
    Some(DemoScenario::ErrorInjection) => {
        demo_scenarios::error_injection_demo(cli.config).await;
    }
    None => {
        run_load_test(cli.config).await;
    }
}
```

## Step 8: Create Helper Scripts

Create `scripts/demo-load.sh`:

```bash
#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

LOAD_TESTER="./load-tester/target/release/load-test"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_demo() {
    echo -e "\n${BLUE}==>${NC} ${YELLOW}$1${NC}\n"
}

# Build load tester if needed
if [ ! -f "$LOAD_TESTER" ]; then
    log_info "Building load tester..."
    cd load-tester
    cargo build --release
    cd ..
fi

# Demo menu
echo ""
echo "RustMart Load Testing Demos"
echo "============================"
echo ""
echo "1) Quick Test (30s light load)"
echo "2) Medium Load (60s)"
echo "3) Stress Test (2 min heavy load)"
echo "4) Error Injection Demo (for alerting)"
echo "5) Ramp-Up Demo (gradual increase)"
echo ""
read -p "Select demo (1-5): " choice

case $choice in
    1)
        log_demo "Running quick test..."
        $LOAD_TESTER --profile light --duration 30
        ;;
    2)
        log_demo "Running medium load test..."
        $LOAD_TESTER --profile medium --duration 60
        ;;
    3)
        log_demo "Running stress test..."
        $LOAD_TESTER --profile heavy --duration 120
        ;;
    4)
        log_demo "Running error injection demo..."
        $LOAD_TESTER --demo error-injection
        ;;
    5)
        log_demo "Running ramp-up demo..."
        $LOAD_TESTER --demo ramp-up
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
log_info "Demo complete! Check Jaeger/Instana for traces"
log_info "Grafana: http://localhost:3000"
log_info "Jaeger: http://localhost:16686"
```

Make it executable:
```bash
chmod +x scripts/demo-load.sh
```

## Step 9: Advanced Features - Chaos Mode

Add chaos engineering capabilities:

```rust
// In scenarios.rs
pub async fn chaos_request(&self) -> RequestResult {
    let mut rng = rand::thread_rng();
    
    match rng.gen_range(0..100) {
        // 20% chance of timeout
        0..=20 => {
            tokio::time::sleep(Duration::from_secs(11)).await;
            RequestResult {
                scenario: "timeout".to_string(),
                duration: Duration::from_secs(11),
                status: 0,
                success: false,
            }
        }
        // 20% chance of server error simulation
        21..=40 => self.get_missing_product().await,
        // 60% normal traffic
        _ => self.random_scenario(0).await,
    }
}
```

## Step 10: Export Results for Analysis

Add JSON export:

```rust
// In stats.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct TestReport {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rps: f64,
    pub latency_p50: u64,
    pub latency_p95: u64,
    pub latency_p99: u64,
    pub status_codes: HashMap<u16, u64>,
}

impl Statistics {
    pub fn generate_report(&self, duration: Duration) -> TestReport {
        let rps = self.total_requests as f64 / duration.as_secs_f64();
        
        TestReport {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            rps,
            latency_p50: self.latencies.value_at_quantile(0.50),
            latency_p95: self.latencies.value_at_quantile(0.95),
            latency_p99: self.latencies.value_at_quantile(0.99),
            status_codes: self.status_codes.clone(),
        }
    }

    pub fn export_json(&self, duration: Duration, path: &str) -> std::io::Result<()> {
        let report = self.generate_report(duration);
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

## Usage Examples for IBM Demo

### 1. Baseline Performance Demo
```bash
# Show normal operation
./target/release/load-test --profile medium --duration 120
```
**Talking points**: 
- "Here's RustMart handling typical load"
- Show in Grafana: request rate, latency, error rate
- Show in Jaeger: distributed traces

### 2. Scaling Demo
```bash
# Start with light, then heavy
./target/release/load-test --demo ramp-up
```
**Talking points**:
- "Watch Kubernetes auto-scaling kick in"
- Show HPA scaling pods as CPU increases
- Show traces spreading across pods

### 3. Error Detection Demo
```bash
# Inject errors to trigger alerts
./target/release/load-test --demo error-injection
```
**Talking points**:
- "Watch error rate increase"
- Show alert triggering in Prometheus/Instana
- Show error traces in Jaeger
- "This is how we catch issues before customers notice"

### 4. Stress Test Demo
```bash
# Push system to limits
./target/release/load-test --profile stress --duration 180
```
**Talking points**:
- "Let's see how the system handles extreme load"
- Show circuit breaker activating
- Show database connection pooling
- Show service degradation gracefully

## Key Features Summary

| Feature | Purpose |
|---------|---------|
| Load Profiles | Easy presets for different scenarios |
| Error Injection | Trigger alerts and demonstrate monitoring |
| Real-time Stats | See results as test runs |
| Histogram Metrics | Accurate latency percentiles |
| Scenario Mix | Realistic traffic patterns |
| Demo Modes | Pre-built sequences for presentations |
| Chaos Mode | Simulate failures |
| JSON Export | Analyze results later |

## Observability Integration

The load tester creates perfect demo conditions:

1. **Distributed Traces**: Each request gets a trace ID
2. **Metrics**: Request volume triggers Prometheus alerts
3. **Logs**: Errors appear in structured logs
4. **Dashboards**: Grafana shows live metrics
5. **Alerts**: Error injection triggers alert rules

## Challenges

1. **Add custom scenarios**: E-commerce user journey (browse → add to cart → checkout)
2. **Add think time variation**: Randomize delays for more realistic patterns
3. **Add webhook notifications**: Send results to Slack/Teams
4. **Add comparison mode**: Compare two test runs
5. **Add distributed load testing**: Multiple load generators

## Next Steps

Now you have:
- ✅ Complete microservices platform
- ✅ Full observability stack
- ✅ Database seeding tools
- ✅ Load testing tool

**For your IBM demo**, you can:
1. Seed database: `./scripts/seed-realistic-data.sh`
2. Start services: `docker-compose up`
3. Run load test: `./scripts/demo-load.sh`
4. Show traces in Instana/Jaeger
5. Show metrics in Grafana
6. Demonstrate alerting with error injection

## Official Documentation

- [tokio](https://tokio.rs/)
- [reqwest](https://docs.rs/reqwest/)
- [clap](https://docs.rs/clap/)
- [indicatif](https://docs.rs/indicatif/)
- [hdrhistogram](https://docs.rs/hdrhistogram/)
- [Load Testing Best Practices](https://www.nginx.com/blog/load-testing-best-practices/)

---

**You now have everything you need for a killer IBM/Instana demo!** 🎯🚀
