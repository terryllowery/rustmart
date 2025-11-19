# Lesson 5: Building the Shared Library

## Official Documentation References

- **thiserror**: https://docs.rs/thiserror/
- **serde**: https://docs.rs/serde/
- **Rust Book - Modules**: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
- **Rust Book - Traits**: https://doc.rust-lang.org/book/ch10-02-traits.html

---

## What You'll Build

The `shared` library will contain code used by all microservices:
- ✅ Common error types (using `thiserror`)
- ✅ Domain models (Product, Order, User, etc.)
- ✅ Utility functions
- ✅ Configuration types

This follows the **DRY principle** - Don't Repeat Yourself!

---

## Step 1: Set Up the Library Structure

First, let's organize the `shared` library with a proper module structure.

### Check Current State

```bash
cd ~/code/rustmart/shared
ls -la src/
```

You should see `lib.rs` (created when we set up the workspace).

### Your Task: Create the Module Structure

Create these files in `shared/src/`:

```
shared/src/
├── lib.rs           # Library root (already exists)
├── error.rs         # Error types
├── models.rs        # Domain models
└── config.rs        # Configuration types
```

**Do this now:**
```bash
cd ~/code/rustmart/shared/src
touch error.rs models.rs config.rs
```

---

## Step 2: Define Error Types (error.rs)

Now you'll apply what you learned in Lesson 4!

### Your Task: Create Custom Error Types

Open `shared/src/error.rs` and implement the following:

**Requirements:**
1. Create an enum called `ApiError` with these variants:
   - `NotFound` - with a String message
   - `Unauthorized` - with a String message
   - `BadRequest` - with a String message
   - `InternalServer` - with a String message
   - `Database` - wrapping a database error (we'll add this later)

2. Use `thiserror` to derive error handling
3. Add helpful error messages with `#[error(...)]`

**Hints:**
- Remember Lesson 4: use `#[derive(Error, Debug)]`
- Error messages can use placeholders: `#[error("Not found: {0}")]`
- For now, don't worry about the Database variant - we'll add it later

**Don't look ahead! Try to write this yourself based on Lesson 4.**

<details>
<summary>🚨 Click here if you're stuck (try first!)</summary>

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalServer(String),
}
```
</details>

---

## Step 3: Add Dependencies

Your code won't compile yet because we need to add `thiserror` to the shared library!

### Your Task: Update shared/Cargo.toml

Open `shared/Cargo.toml` and add dependencies:

```toml
[dependencies]
thiserror.workspace = true
serde = { workspace = true }
```

**Why `.workspace = true`?**  
Remember Lesson 1 - this uses the version defined in the root `Cargo.toml`!

---

## Step 4: Test Your Error Types

Let's make sure your code compiles!

### Your Task: Update lib.rs

Open `shared/src/lib.rs` and add:

```rust
pub mod error;

// Re-export for convenience
pub use error::ApiError;
```

**Why re-export?**  
So other crates can do `use shared::ApiError` instead of `use shared::error::ApiError`.

### Build and Check

```bash
cd ~/code/rustmart/shared
cargo build
```

**Did it compile?** ✅ Great! Move to Step 5.  
**Got errors?** 🔍 Read the error message carefully - Rust's compiler is very helpful!

---

## Step 5: Define Domain Models (models.rs)

Now let's create the data types that represent our business domain.

### Your Task: Create a Product Model

Open `shared/src/models.rs` and create a `Product` struct:

**Requirements:**
1. Create a struct called `Product` with these fields:
   - `id` - String
   - `name` - String
   - `description` - String
   - `price` - f64
   - `stock` - i32

2. Make it serializable with Serde (we'll need this for JSON APIs later)
3. All fields should be public (other crates need to access them)

**Hints:**
- Use `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`
- Make fields public with `pub`

**Try writing this yourself!**

<details>
<summary>🚨 Solution (try first!)</summary>

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
}
```
</details>

### Add More Models (Optional for now)

If you're feeling confident, add these models too:

**User:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}
```

**Order:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub price: f64,
}
```

---

## Step 6: Export Models from lib.rs

### Your Task: Update lib.rs

Add the models module and re-export the types:

```rust
pub mod error;
pub mod models;

// Re-exports
pub use error::ApiError;
pub use models::{Product, User, Order, OrderItem};
```

**Build again:**
```bash
cd ~/code/rustmart/shared
cargo build
```

---

## Step 7: Add Configuration Types (config.rs)

Every microservice needs configuration (database URLs, ports, etc.).

### Your Task: Create a Config Struct

Open `shared/src/config.rs`:

**Requirements:**
1. Create a `DatabaseConfig` struct with:
   - `url` - String (database connection URL)
   - `max_connections` - u32

2. Create a `ServerConfig` struct with:
   - `host` - String
   - `port` - u16

**Try it yourself!**

<details>
<summary>🚨 Solution</summary>

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
```
</details>

### Export from lib.rs

```rust
pub mod error;
pub mod models;
pub mod config;

// Re-exports
pub use error::ApiError;
pub use models::{Product, User, Order, OrderItem};
pub use config::{DatabaseConfig, ServerConfig};
```

---

## Step 8: Build the Entire Workspace

Now let's make sure everything compiles together!

```bash
cd ~/code/rustmart
cargo build --workspace
```

**Success?** 🎉 Congratulations! You've built your first Rust library!

---

## Step 9: Write a Simple Test

Let's verify your code works with a test.

### Your Task: Add a Test to models.rs

At the bottom of `shared/src/models.rs`, add:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let product = Product {
            id: "1".to_string(),
            name: "Test Product".to_string(),
            description: "A test product".to_string(),
            price: 99.99,
            stock: 10,
        };
        
        assert_eq!(product.id, "1");
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.price, 99.99);
    }
}
```

### Run the Test

```bash
cd ~/code/rustmart/shared
cargo test
```

**Did it pass?** ✅

---

## Step 10: Test Serialization (JSON)

Since we'll be building REST APIs, let's make sure our models can convert to/from JSON.

### Your Task: Add JSON Test

Add this test to `models.rs`:

```rust
#[test]
fn test_product_json_serialization() {
    let product = Product {
        id: "1".to_string(),
        name: "Test Product".to_string(),
        description: "A test".to_string(),
        price: 99.99,
        stock: 10,
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&product).unwrap();
    assert!(json.contains("Test Product"));
    
    // Deserialize from JSON
    let deserialized: Product = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, product.id);
}
```

**But wait!** This won't work yet. You need to add `serde_json` as a **dev dependency**.

Add to `shared/Cargo.toml`:

```toml
[dev-dependencies]
serde_json = "1.0"
```

Now run:
```bash
cargo test
```

---

## What You've Accomplished

✅ Created a shared library with proper module organization  
✅ Defined custom error types using `thiserror`  
✅ Created domain models (Product, User, Order)  
✅ Made models serializable with Serde  
✅ Added configuration types  
✅ Wrote and ran tests  
✅ Verified JSON serialization works  

**This is REAL Rust development!**

---

## Key Concepts Learned

### 1. Module Organization
```rust
pub mod error;      // Declare module
pub use error::ApiError;  // Re-export for convenience
```

### 2. Library Crates
- `lib.rs` is the entry point
- Other crates depend on this: `shared = { path = "../shared" }`

### 3. Derive Macros
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```
- Auto-generate trait implementations
- Save tons of boilerplate code

### 4. Testing
```rust
#[cfg(test)]  // Only compile during tests
mod tests {
    use super::*;  // Import from parent module
    
    #[test]  // Mark as test function
    fn test_something() { }
}
```

---

## Next Steps

Now that you have a shared library, you can:
1. ✅ Build your first microservice (product-service)
2. ✅ Add HTTP endpoints using Axum
3. ✅ Add database operations with SQLx
4. ✅ Add OpenTelemetry instrumentation (IBM demo!)

---

## Troubleshooting

### "Cannot find `thiserror` in scope"
**Solution:** Make sure `thiserror.workspace = true` is in `shared/Cargo.toml`

### "Trait `Serialize` is not implemented"
**Solution:** Add `#[derive(Serialize, Deserialize)]` to your struct

### "Module not found"
**Solution:** Make sure you declared the module in `lib.rs` with `pub mod module_name;`

### Build fails with dependency errors
**Solution:** Run `cargo update` to refresh dependencies

---

## Challenge: Enhance Your Models

Want more practice? Try these:

1. **Add validation** - Create a `new()` method for Product that validates price > 0
2. **Add more fields** - Add `created_at` and `updated_at` timestamps
3. **Add enums** - Create an `OrderStatus` enum (Pending, Paid, Shipped, Delivered)
4. **Add methods** - Implement methods like `calculate_total()` for Order

---

## Summary

You've built the foundation of RustMart! The shared library is now ready to be used by all your microservices. In the next lesson, we'll build the product-service and see how to use this shared code.

**Ready for Lesson 6: Building Product Service with Axum?**
