# Lesson 2: Project Structure - lib.rs vs main.rs

## Official Documentation References

- **Cargo Book - Package Layout**: https://doc.rust-lang.org/cargo/guide/project-layout.html
- **Rust Book - Packages and Crates**: https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
- **Cargo Book - Workspaces**: https://doc.rust-lang.org/cargo/reference/workspaces.html
- **Cargo Reference - Targets**: https://doc.rust-lang.org/cargo/reference/cargo-targets.html
- **API Guidelines - Crate Organization**: https://rust-lang.github.io/api-guidelines/organization.html

## The Question

When building Rust applications, should you put code in:
- `src/main.rs` (binary crate)
- `src/lib.rs` (library crate)
- Both?

**Answer:** For any non-trivial application, **use both**. This is the idiomatic Rust pattern.

## What Does "Non-Trivial" Mean?

**Non-trivial** = **Not simple** or **More than basic/minimal**

### Trivial Applications (Just use main.rs)

These are simple enough to live in a single `main.rs` file:

**Example 1: Simple CLI tool**
```rust
// A trivial "hello world" CLI - just main.rs is fine
fn main() {
    let name = std::env::args().nth(1).unwrap_or("World".to_string());
    println!("Hello, {}!", name);
}
```

**Example 2: Basic file converter**
```rust
// A trivial file converter - just main.rs is fine
fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let output = input.to_uppercase();
    std::fs::write("output.txt", output).unwrap();
}
```

**Characteristics of trivial applications:**
- ✅ 10-50 lines total
- ✅ Single file is sufficient
- ✅ No need to test individual functions
- ✅ Won't be reused by other code
- ✅ No complex business logic
- ✅ Single, simple purpose

### Non-Trivial Applications (Use lib.rs + main.rs)

These need proper structure and organization:

**Example 1: Web service** (like our microservices)
- Has HTTP handlers, database connections, config, error handling, middleware
- Multiple concerns that need organization

**Example 2: Game engine**
- Has rendering, physics, input, audio, scripting systems
- Many subsystems that need to be testable separately

**Example 3: CLI tool with business logic** (e.g., `cargo`, `git`)
- Complex command handling, config management, plugin systems

**Characteristics of non-trivial applications:**
- ✅ 100+ lines of code
- ✅ Multiple responsibilities/concerns
- ✅ Need to test logic independently
- ✅ Might be used by other crates
- ✅ Has business logic worth documenting
- ✅ Will grow and evolve over time
- ✅ Benefits from modular organization

### Rule of Thumb

**Use just main.rs if:**
- Single-purpose script/tool
- < 50 lines of code
- No tests needed
- Won't be reused by other code
- Simple, straightforward logic

**Use lib.rs + main.rs if:**
- **Web service/HTTP server** ← Our case!
- Has business logic to test
- Multiple modules/concerns
- Will grow to 100+ lines
- Part of a larger system
- **Any microservice architecture** ← Definitely our case!

### Why All Our RustMart Services Are Non-Trivial

All our services (api-gateway, product-service, order-service, etc.) are **definitely non-trivial** because they:

1. **Handle HTTP requests** - Need route handlers, middleware, error responses
2. **Connect to databases** - Require connection pools, queries, transactions
3. **Have business logic** - Product catalog, order processing, payment flows
4. **Need error handling** - Database errors, validation, HTTP status codes
5. **Require testing** - Unit tests for logic, integration tests for APIs
6. **Will grow significantly** - Start at hundreds of lines, grow to thousands
7. **Part of distributed system** - Interact with other services, message queues
8. **Need observability** - Logging, tracing, metrics

Therefore, we'll use the **lib.rs + main.rs** pattern for all of them.

**Reference**: The Rust community widely adopts this pattern for any production service. See examples in:
- [Tokio's mini-redis example](https://github.com/tokio-rs/mini-redis)
- [Actix-web examples](https://github.com/actix/examples)
- [Real-world Rust web services](https://github.com/gothinkster/realworld)

## Understanding Crates and Targets

**From the [Rust Book](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html):**

- A **crate** is the smallest amount of code the Rust compiler considers at a time
- A **package** can contain multiple crates
- A crate can be a **binary crate** (has main.rs, produces an executable)
- A crate can be a **library crate** (has lib.rs, provides functionality to other crates)

**From the [Cargo Book](https://doc.rust-lang.org/cargo/guide/project-layout.html):**
```
.
├── Cargo.toml
├── src/
│   ├── lib.rs       ← Library crate root
│   ├── main.rs      ← Binary crate root (default binary)
│   └── bin/         ← Additional binaries
│       └── tool.rs
└── tests/           ← Integration tests
    └── test.rs
```

## The Pattern: Thin Binary, Fat Library

### Structure
```
product-service/
├── Cargo.toml
└── src/
    ├── main.rs          ← Thin (5-20 lines)
    ├── lib.rs           ← Entry point for library
    ├── config.rs        ← Configuration
    ├── handlers/        ← HTTP handlers
    │   ├── mod.rs
    │   └── product.rs
    ├── models/          ← Data models
    │   ├── mod.rs
    │   └── product.rs
    ├── db.rs            ← Database logic
    └── error.rs         ← Error types
```

### Why This Pattern?

#### ✅ Benefits of Fat Library

1. **Testability**
   - Libraries can be unit tested easily
   - Binaries are harder to test (they just run)
   - You can test internal functions without running the whole app
   - **Reference**: [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)

2. **Reusability**
   - Other crates can depend on your library
   - Integration tests can import your library
   - Tools/scripts can use your core logic
   - **Reference**: [Cargo Book - Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)

3. **Documentation**
   - `cargo doc` works on libraries, not binaries
   - Library code gets proper documentation generation
   - Internal APIs are discoverable
   - **Reference**: [rustdoc Book](https://doc.rust-lang.org/rustdoc/index.html)

4. **Modularity**
   - Forces you to think about public vs private APIs
   - Cleaner separation of concerns
   - Easier to refactor
   - **Reference**: [Rust Book - Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)

#### ❌ Problems with Fat Binary

1. Can't unit test functions easily
2. No way to reuse code in other crates
3. All code is effectively "private"
4. Can't generate documentation
5. Harder to structure large codebases

## The Idiomatic Pattern

### main.rs - The Thin Binary (5-20 lines)

**Reference**: [Cargo Targets - Binaries](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)

```rust
// product-service/src/main.rs

// Just import and run!
use product_service::{Config, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup (minimal)
    tracing_subscriber::fmt::init();
    let config = Config::from_env()?;
    
    // Run the app (all logic in lib.rs)
    run(config).await
}
```

**What main.rs should do:**
- ✅ Parse environment/config
- ✅ Setup logging/tracing
- ✅ Call the library's `run()` function
- ❌ NO business logic
- ❌ NO HTTP handlers
- ❌ NO database code

### lib.rs - The Fat Library

**Reference**: [Cargo Targets - Libraries](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)

```rust
// product-service/src/lib.rs

// Re-export main types for easy importing
pub use config::Config;
pub use error::Error;

// Declare modules
pub mod config;
pub mod handlers;
pub mod models;
pub mod db;
pub mod error;

// Main application logic
pub async fn run(config: Config) -> anyhow::Result<()> {
    // Setup database pool
    let db = db::connect(&config.database_url).await?;
    
    // Setup router
    let app = handlers::create_router(db);
    
    // Start server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

**What lib.rs should do:**
- ✅ Declare all modules
- ✅ Re-export public types (see [API Guidelines - Re-exports](https://rust-lang.github.io/api-guidelines/organization.html#c-reexport))
- ✅ Contain main `run()` or `start()` function
- ✅ ALL business logic
- ✅ Define public API

## Cargo.toml Configuration

**Reference**: [Cargo Book - The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)

### For Binary + Library (Recommended)

```toml
[package]
name = "product-service"
version.workspace = true
edition.workspace = true

# By default, Cargo creates:
# - A library if src/lib.rs exists
# - A binary if src/main.rs exists
# - Both if both exist!

[dependencies]
# Your dependencies
```

**From the [Cargo Book](https://doc.rust-lang.org/cargo/guide/project-layout.html):**
> Cargo automatically determines the targets from the layout of the files on the filesystem.

**No special configuration needed!** Cargo automatically detects both.

### Library-Only Package

```toml
[package]
name = "shared"
version.workspace = true
edition.workspace = true

# Only src/lib.rs exists - this is a library-only crate
# Cannot be run with `cargo run`

[dependencies]
```

### Binary-Only Package (Not Recommended for Services)

```toml
[package]
name = "simple-tool"
version.workspace = true
edition.workspace = true

# Only src/main.rs exists - this is a binary-only crate
# Cannot be imported by other crates

[dependencies]
```

## Real-World Example: Product Service

### Option 1: Binary-Only (❌ Not Recommended)
```rust
// src/main.rs - 500+ lines of mixed concerns
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // Config parsing
    let db_url = std::env::var("DATABASE_URL").unwrap();
    
    // Database setup
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
    
    // Handlers defined inline or in same file
    let app = Router::new()
        .route("/products", get(list_products));
    
    // Server startup
    axum::Server::bind(&"0.0.0.0:8081".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_products() -> String {
    // Handler logic here
    "Products".to_string()
}

// 400 more lines of handlers, models, etc...
```

**Problems:**
- Can't test `list_products` without running the whole server
- Can't reuse any logic in integration tests
- No clear separation of concerns
- Hard to navigate and maintain

### Option 2: Library + Binary (✅ Recommended)
```rust
// src/main.rs (15 lines)
use product_service::{Config, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env()?;
    run(config).await
}
```

```rust
// src/lib.rs
pub mod config;
pub mod handlers;
pub mod models;
pub mod db;
pub mod error;

pub use config::Config;
pub use error::Error;

pub async fn run(config: Config) -> anyhow::Result<()> {
    let db = db::connect(&config.database_url).await?;
    let app = handlers::create_router(db);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

```rust
// src/handlers/product.rs
pub async fn list_products(
    State(db): State<PgPool>
) -> Result<Json<Vec<Product>>, Error> {
    let products = sqlx::query_as!(Product, "SELECT * FROM products")
        .fetch_all(&db)
        .await?;
    Ok(Json(products))
}
```

**Benefits:**
- ✅ Can test handlers: `use product_service::handlers::product::list_products;`
- ✅ Integration tests can import: `use product_service::Config;`
- ✅ Clear module structure
- ✅ Easy to navigate
- ✅ Can generate documentation

## Module Organization Best Practices

**Reference**: [Rust Book - Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

### Flat Structure (Small Projects)
```
src/
├── main.rs
├── lib.rs
├── config.rs
├── handlers.rs
├── models.rs
└── error.rs
```

### Nested Structure (Larger Projects - Our Approach)
```
src/
├── main.rs
├── lib.rs
├── config.rs
├── error.rs
├── handlers/
│   ├── mod.rs        ← Re-exports all handlers
│   ├── product.rs
│   ├── health.rs
│   └── metrics.rs
├── models/
│   ├── mod.rs
│   ├── product.rs
│   └── category.rs
└── db/
    ├── mod.rs
    ├── product.rs
    └── migrations.rs
```

### lib.rs with Nested Modules

**Reference**: [Rust Book - Paths for Referring to an Item](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)

```rust
// src/lib.rs
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod db;

// Re-export commonly used items
pub use config::Config;
pub use error::Error;
pub use models::{Product, Category};

pub async fn run(config: Config) -> anyhow::Result<()> {
    // Main app logic
}
```

### handlers/mod.rs

**Reference**: [Rust Book - Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)

```rust
// src/handlers/mod.rs
pub mod product;
pub mod health;
pub mod metrics;

use axum::Router;
use sqlx::PgPool;

// Create the router - this is exported for lib.rs to use
pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/products", get(product::list_products))
        .route("/health", get(health::check))
        .route("/metrics", get(metrics::prometheus))
        .with_state(db)
}
```

## Testing Patterns

**Reference**: [Rust Book - How to Write Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)

### Unit Tests (Only Possible with lib.rs)
```rust
// src/handlers/product.rs
pub fn calculate_price(base: f64, discount: f64) -> f64 {
    base * (1.0 - discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_price() {
        assert_eq!(calculate_price(100.0, 0.1), 90.0);
    }
}
```

### Integration Tests

**Reference**: [Rust Book - Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)

```rust
// tests/integration_test.rs
use product_service::{Config, handlers};

#[tokio::test]
async fn test_product_handler() {
    // Can import and test library functions
    // Without starting the whole server
}
```

## Visibility and Privacy

**Reference**: [Rust Book - Controlling Visibility with pub](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#exposing-paths-with-the-pub-keyword)

### Visibility Levels
```rust
// Private - only visible within current module (default)
fn internal_helper() {}

// Public - visible to anyone who imports this crate
pub fn public_api() {}

// Public within crate - visible only within your crate
pub(crate) fn internal_api() {}

// Public within parent module
pub(super) fn parent_visible() {}

// Public within specific path
pub(in crate::handlers) fn handlers_only() {}
```

## Do's and Don'ts

### ✅ Do

1. **Use lib.rs + main.rs pattern** for all services
2. **Keep main.rs thin** (< 20 lines typically)
3. **Put all logic in lib.rs and modules**
4. **Export a public `run()` or `start()` function** from lib.rs
5. **Use modules to organize code** (handlers, models, db, etc.)
6. **Write unit tests alongside code** in library modules ([Testing](https://doc.rust-lang.org/book/ch11-00-testing.html))
7. **Re-export commonly used types** in lib.rs ([API Guidelines](https://rust-lang.github.io/api-guidelines/organization.html#c-reexport))
8. **Use `pub` carefully** - only expose what others need

### ❌ Don't

1. **Don't put business logic in main.rs**
2. **Don't make everything public** - use `pub(crate)` for internal APIs
3. **Don't skip lib.rs** for non-trivial applications
4. **Don't have a 1000-line main.rs**
5. **Don't mix configuration and business logic**
6. **Don't forget to organize into modules** as project grows

## Our RustMart Structure

For each service (product-service, order-service, etc.):

```
service-name/
├── Cargo.toml
├── src/
│   ├── main.rs          ← 10-15 lines: setup + call run()
│   ├── lib.rs           ← Module declarations + run() function
│   ├── config.rs        ← Config struct + env parsing
│   ├── error.rs         ← Error types
│   ├── handlers/        ← HTTP request handlers
│   │   ├── mod.rs
│   │   └── ...
│   ├── models/          ← Domain models (structs)
│   │   ├── mod.rs
│   │   └── ...
│   └── db/              ← Database queries
│       ├── mod.rs
│       └── ...
└── tests/               ← Integration tests
    └── integration_test.rs
```

## Commands to Remember

**Reference**: [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/index.html)

```bash
# Build the library + binary
cargo build -p product-service

# Run the binary (requires main.rs)
cargo run -p product-service

# Test the library (unit + doc tests)
cargo test -p product-service

# Test just the library code (no integration tests)
cargo test -p product-service --lib

# Generate documentation for the library
cargo doc -p product-service --open

# Check the library code
cargo check -p product-service --lib
```

## Key Takeaways

1. **Thin binary, fat library** is the idiomatic Rust pattern
2. `main.rs` is just the entry point - it should be **5-20 lines**
3. All business logic goes in `lib.rs` and supporting modules
4. This pattern enables **testing, reusability, and documentation**
5. Cargo automatically detects and builds both if both files exist
6. Structure your code into **logical modules** (handlers, models, db, etc.)
7. Use `pub` to control what's exposed in your library's API
8. Follow the [official project layout guidelines](https://doc.rust-lang.org/cargo/guide/project-layout.html)

## Additional Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Rust Patterns**: https://rust-unofficial.github.io/patterns/

## Next Steps

Now we'll implement this pattern for each of our microservices, starting with the shared library.
