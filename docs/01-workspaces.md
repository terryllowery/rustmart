# Lesson 1: Cargo Workspaces

## The Problem Workspaces Solve

Imagine you have 6 different Rust projects (5 microservices + 1 shared library). Without workspaces:
- Each would have its own `target/` directory → **6x disk space usage!**
- Each would download tokio 1.35 separately → **6x duplicate dependencies**
- Building all services would be slow → **no shared compilation cache**
- Version management nightmare → **keeping all services on same dependency versions**

## Our Workspace Structure

```
rustmart/
├── Cargo.toml              ← Workspace root (coordinator)
├── target/                 ← Single shared build directory
├── api-gateway/
│   └── Cargo.toml          ← Member package
├── product-service/
│   └── Cargo.toml          ← Member package
├── order-service/
│   └── Cargo.toml          ← Member package
├── inventory-service/
│   └── Cargo.toml          ← Member package
├── payment-service/
│   └── Cargo.toml          ← Member package
└── shared/
    └── Cargo.toml          ← Member package (library)
```

## How Workspaces Work

### 1. The Root Workspace File

Location: `/Users/Terry/code/rustmart/Cargo.toml`

```toml
[workspace]
resolver = "2"              # Use the newer dependency resolver
members = [
    "api-gateway",
    "product-service",
    "order-service",
    "inventory-service",
    "payment-service",
    "shared",
]
```

**What this does:**
- Lists all packages that belong to the workspace
- When you run `cargo build` at the root, it builds **all members**
- Creates a **single** `target/` directory shared by everyone
- Cargo knows these packages are related and can optimize builds

### 2. Shared Configuration

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Terry"]
```

**Benefits:**
- Define common package metadata **once**
- Members inherit these with `version.workspace = true`
- Change version in one place, updates everywhere
- Ensures consistency across all services

### 3. Shared Dependencies

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
```

**How it works:**
- Define dependency versions **once** at the root
- Members opt-in by using `dependency.workspace = true` in their own `Cargo.toml`
- Ensures all services use the **exact same version** (prevents version conflicts!)
- Cargo downloads and compiles each dependency only once

### 4. Member Packages

Example: `shared/Cargo.toml`

```toml
[package]
name = "shared"
version.workspace = true    # Inherits "0.1.0" from root
edition.workspace = true    # Inherits "2021" from root
authors.workspace = true    # Inherits ["Terry"] from root

[dependencies]
# This package's specific dependencies
# Can use .workspace = true to use workspace versions
```

## Key Benefits

### Single Build Directory
Instead of:
```
api-gateway/target/        (200 MB)
product-service/target/    (200 MB)
order-service/target/      (200 MB)
...
```

You get:
```
rustmart/target/           (200 MB total!)
├── debug/
│   ├── api-gateway        (binary)
│   ├── shared.rlib        (library)
│   └── product-service    (binary)
```

### Shared Compilation Cache
When `product-service` needs `tokio`, Cargo:
1. Checks if it's already compiled in `target/`
2. **Reuses it** if available (super fast!)
3. Only compiles once for all services

First build:
```
product-service: 2m 30s (compiles tokio + 200 dependencies)
order-service:   30s    (reuses tokio, only compiles new code)
inventory-service: 25s  (reuses even more)
```

### Workspace-Wide Commands

```bash
# Build everything
cargo build --workspace

# Test everything
cargo test --workspace

# Check all code without building
cargo check --workspace

# Run a specific service
cargo run -p product-service

# Build just one service (still shares dependencies)
cargo build -p api-gateway
```

## How Members Reference Each Other

When a service needs the shared library:

```toml
# In product-service/Cargo.toml
[dependencies]
shared = { path = "../shared" }
```

The workspace automatically:
- Links them together
- Ensures `shared` is built first
- Allows `product-service` to use `shared`'s types and functions

## Build Process Explained

When you run `cargo build --workspace`:

1. **Discovery:** Cargo reads root `Cargo.toml` and finds all members
2. **Analysis:** Analyzes dependencies across ALL members
3. **Unification:** Creates a unified dependency graph (deduplicates everything)
4. **Compilation:** 
   - Compiles shared dependencies once
   - Each service only compiles its unique code
   - Libraries (`shared`) are built before binaries that depend on them

## Common Patterns

### Adding a New Dependency to Multiple Services

**Option 1: Use workspace dependencies (recommended)**
```toml
# Root Cargo.toml
[workspace.dependencies]
uuid = { version = "1.6", features = ["v4", "serde"] }

# product-service/Cargo.toml
[dependencies]
uuid.workspace = true
```

**Option 2: Add directly to member**
```toml
# product-service/Cargo.toml
[dependencies]
uuid = { version = "1.6", features = ["v4"] }
```

### Shared Code Pattern

Put common code in `shared/`:
```rust
// shared/src/error.rs
pub enum ApiError {
    NotFound,
    Internal(String),
}

// product-service/src/main.rs
use shared::error::ApiError;
```

## Common Issues and Solutions

### Issue: "failed to load manifest for workspace member"
**Cause:** Workspace member listed but `Cargo.toml` doesn't exist
**Solution:** Create the `Cargo.toml` file or remove from workspace members list

### Issue: Version conflicts between services
**Cause:** Services using different versions of same dependency
**Solution:** Use `[workspace.dependencies]` to enforce single version

### Issue: "error: current package believes it's in a workspace"
**Cause:** Package has its own `[workspace]` section conflicting with parent
**Solution:** Remove `[workspace]` from member, only use `[package]`

## Key Takeaways

✅ Workspaces allow multiple packages to share dependencies and build settings
✅ Single `target/` directory saves massive amounts of disk space
✅ Compilation cache shared across all members speeds up builds significantly
✅ Workspace members must have valid `Cargo.toml` files
✅ Use `[workspace.dependencies]` to enforce version consistency
✅ Use `cargo run -p <package>` to run specific services
✅ Nested git repos can cause issues (we hit this with `shared/.git`)

## Next Steps

Now that you understand workspaces, we can:
- Build the shared library with common types
- Implement individual microservices
- Set up inter-service dependencies
- Add testing infrastructure
