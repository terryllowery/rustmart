# Lesson 6: Building Product Service with Axum

## Official Documentation References

- **Axum**: https://docs.rs/axum/
- **Tokio**: https://tokio.rs/
- **Async Book**: https://rust-lang.github.io/async-book/
- **Axum Examples**: https://github.com/tokio-rs/axum/tree/main/examples

---

## What You'll Build

A real HTTP microservice that:
- ✅ Handles HTTP GET/POST requests
- ✅ Returns JSON responses
- ✅ Uses your shared library
- ✅ Runs asynchronously with Tokio
- ✅ Has proper error handling

**This is where it all comes together!**

---

## Understanding Async Rust (Quick Primer)

Before we code, understand this:

### Synchronous (Blocking)
```rust
fn get_data() -> String {
    // Blocks the thread while waiting
    std::thread::sleep(Duration::from_secs(1));
    "data".to_string()
}
```

### Asynchronous (Non-Blocking)
```rust
async fn get_data() -> String {
    // Doesn't block! Other tasks can run
    tokio::time::sleep(Duration::from_secs(1)).await;
    "data".to_string()
}
```

**Key concepts:**
- `async fn` = function that can be paused and resumed
- `.await` = pause here until result is ready
- **Tokio** = the runtime that manages all this

---

## Step 1: Check Product Service Structure

First, let's see what we have:

```bash
cd ~/code/rustmart/product-service
ls -la src/
```

You should see `main.rs` and `lib.rs` (from workspace setup).

---

## Step 2: Add Dependencies

We need to add the dependencies for our service.

### Your Task: Update product-service/Cargo.toml

Open `product-service/Cargo.toml` and add:

```toml
[dependencies]
# Use shared library
shared = { path = "../shared" }

# Web framework
axum.workspace = true

# Async runtime
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json.workspace = true

# Error handling
anyhow.workspace = true
```

**Note:** These use `.workspace = true` because they're defined in the root `Cargo.toml`!

---

## Step 3: Create Your First Route (Health Check)

Let's start simple with a health check endpoint.

### Your Task: Set Up lib.rs

Open `product-service/src/lib.rs` and add:

```rust
use axum::{
    routing::get,
    Router,
};

/// Create the application router with all routes
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
```

**What's happening:**
- `Router::new()` = creates a new router
- `.route("/health", get(health_check))` = GET /health calls health_check
- `async fn health_check()` = async handler that returns "OK"

### Test It Compiles

```bash
cd ~/code/rustmart/product-service
cargo build
```

**Did it compile?** ✅

---

## Step 4: Create the Main Function

Now make it runnable!

### Your Task: Update main.rs

Open `product-service/src/main.rs`:

```rust
use product_service::create_router;

#[tokio::main]
async fn main() {
    // Create the router
    let app = create_router();
    
    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8001")
        .await
        .unwrap();
    
    println!("🚀 Product service listening on http://localhost:8001");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
```

**What's happening:**
- `#[tokio::main]` = sets up the async runtime
- `TcpListener::bind()` = listen on port 8001
- `axum::serve()` = start serving requests

### Run Your Service!

```bash
cargo run
```

You should see:
```
🚀 Product service listening on http://localhost:8001
```

### Test It!

Open another terminal and run:
```bash
curl http://localhost:8001/health
```

**Did you see "OK"?** 🎉 **You just built your first microservice!**

Press Ctrl+C to stop the server.

---

## Step 5: Add JSON Response

Let's return JSON instead of plain text.

### Your Task: Create a JSON Health Response

Update `lib.rs`:

```rust
use axum::{
    routing::get,
    Router,
    Json,
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        service: "product-service".to_string(),
    })
}
```

**What changed:**
- Added `Json` wrapper from Axum
- Created `HealthResponse` struct
- Return `Json<HealthResponse>` = auto-serializes to JSON!

### Test It

```bash
cargo run
```

In another terminal:
```bash
curl http://localhost:8001/health
```

**You should see:**
```json
{"status":"OK","service":"product-service"}
```

---

## Step 6: Add Product Endpoints

Now let's use your shared library!

### Your Task: Add Product Routes

Update `lib.rs` to add product endpoints:

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Serialize, Deserialize};
use shared::Product;  // ← Use your shared library!

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/products", get(list_products))
        .route("/products/:id", get(get_product))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        service: "product-service".to_string(),
    })
}

async fn list_products() -> Json<Vec<Product>> {
    // For now, return mock data
    let products = vec![
        Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            description: "High-performance laptop".to_string(),
            price: 999.99,
            stock: 10,
        },
        Product {
            id: "2".to_string(),
            name: "Mouse".to_string(),
            description: "Wireless mouse".to_string(),
            price: 29.99,
            stock: 50,
        },
    ];
    
    Json(products)
}

async fn get_product() -> Json<Product> {
    // Mock single product
    Json(Product {
        id: "1".to_string(),
        name: "Laptop".to_string(),
        description: "High-performance laptop".to_string(),
        price: 999.99,
        stock: 10,
    })
}
```

### Test Your New Endpoints

```bash
cargo run
```

Try these:
```bash
# List all products
curl http://localhost:8001/products

# Get single product
curl http://localhost:8001/products/1
```

**You should see JSON responses with product data!**

---

## Step 7: Extract Path Parameters

Let's make the product ID dynamic.

### Your Task: Use Path Parameters

Update the `get_product` function:

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::Path,  // ← Add this
};

// ... other code ...

async fn get_product(Path(id): Path<String>) -> Json<Product> {
    // Now we have the actual ID from the URL!
    println!("Fetching product with ID: {}", id);
    
    Json(Product {
        id: id.clone(),  // Use the actual ID
        name: "Laptop".to_string(),
        description: "High-performance laptop".to_string(),
        price: 999.99,
        stock: 10,
    })
}
```

**What's new:**
- `Path(id): Path<String>` = extracts the `:id` from the URL
- Now `id` contains whatever was in the URL!

### Test It

```bash
cargo run
```

```bash
curl http://localhost:8001/products/123
curl http://localhost:8001/products/abc
```

Check your terminal - you should see the logs showing different IDs!

---

## Step 8: Add Error Handling

Let's use your shared ApiError!

### Your Task: Return Errors

Update `lib.rs`:

```rust
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};
use shared::{Product, ApiError};

// ... other code ...

async fn get_product(Path(id): Path<String>) -> Result<Json<Product>, ApiError> {
    // Simulate not found
    if id == "999" {
        return Err(ApiError::NotFound(format!("Product {} not found", id)));
    }
    
    Ok(Json(Product {
        id: id.clone(),
        name: "Laptop".to_string(),
        description: "High-performance laptop".to_string(),
        price: 999.99,
        stock: 10,
    }))
}

// Make ApiError work with Axum
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalServer(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        (status, message).into_response()
    }
}
```

### Test Error Handling

```bash
cargo run
```

```bash
# This should work
curl http://localhost:8001/products/1

# This should return 404
curl http://localhost:8001/products/999
```

**You should see a 404 error for product 999!**

---

## Step 9: Add Logging

Let's see what's happening!

### Your Task: Add Tracing

Update `main.rs`:

```rust
use product_service::create_router;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let app = create_router();
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8001")
        .await
        .unwrap();
    
    tracing::info!("🚀 Product service listening on http://localhost:8001");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
```

**Add to Cargo.toml:**
```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

Now you'll see nice log messages!

---

## What You've Accomplished

✅ Built your first async Rust service  
✅ Created HTTP endpoints with Axum  
✅ Used your shared library (Product, ApiError)  
✅ Handled path parameters  
✅ Implemented error handling  
✅ Added JSON serialization  
✅ Added logging  

**You now have a working microservice!**

---

## Key Concepts Learned

### 1. Async/Await
```rust
async fn my_function() -> String {
    some_async_operation().await
}
```
- `async` = function can pause
- `.await` = pause until ready

### 2. Axum Routing
```rust
Router::new()
    .route("/path", get(handler))
    .route("/path/:id", get(handler_with_id))
```

### 3. Extractors
```rust
async fn handler(Path(id): Path<String>) -> Json<Data> {
    // id is extracted from URL
}
```

### 4. Error Responses
```rust
impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        // Convert error to HTTP response
    }
}
```

---

## Next Steps

Now you can:
1. ✅ Add OpenTelemetry instrumentation (Lesson 7)
2. ✅ Add database operations
3. ✅ Build the other microservices
4. ✅ Add tests

---

## Challenges

Want more practice?

1. **Add a POST endpoint** to create products
2. **Add query parameters** for filtering products
3. **Add validation** for product prices (must be > 0)
4. **Add an in-memory store** (HashMap) instead of mock data
5. **Add tests** for your handlers

---

## Troubleshooting

### "Cannot find `axum` in scope"
**Solution:** Make sure `axum.workspace = true` is in `Cargo.toml`

### "Cannot find `tokio` macro"
**Solution:** Make sure tokio is in dependencies with features enabled

### Server won't start
**Solution:** Check if port 8001 is already in use: `lsof -i :8001`

### Compile errors about traits
**Solution:** Make sure you imported everything at the top of lib.rs

---

## Summary

You've built a real HTTP microservice in Rust! You learned:
- Async programming with Tokio
- Web framework (Axum)
- HTTP routing and handlers
- JSON serialization
- Error handling
- Path parameters

**This is production-grade Rust development!**

**Ready for Lesson 7: Adding OpenTelemetry?**
