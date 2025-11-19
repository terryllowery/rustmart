# Lesson 3: Cargo - The Rust Build Tool

Comprehensive guide to Cargo, Rust's package manager and build tool, including cross-compilation.

## Official Documentation References

- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Cargo Commands**: https://doc.rust-lang.org/cargo/commands/index.html
- **Cargo Manifest Format**: https://doc.rust-lang.org/cargo/reference/manifest.html
- **Cargo Configuration**: https://doc.rust-lang.org/cargo/reference/config.html
- **Cargo Build Scripts**: https://doc.rust-lang.org/cargo/reference/build-scripts.html
- **Cross-Compilation**: https://rust-lang.github.io/rustup/cross-compilation.html
- **Platform Support**: https://doc.rust-lang.org/rustc/platform-support.html

---

## Table of Contents

1. [What is Cargo?](#what-is-cargo)
2. [Project Structure](#project-structure)
3. [Cargo.toml Manifest](#cargotoml-manifest)
4. [Common Commands](#common-commands)
5. [Dependencies](#dependencies)
6. [Features](#features)
7. [Workspaces](#workspaces)
8. [Build Profiles](#build-profiles)
9. [Testing](#testing)
10. [Documentation](#documentation)
11. [Publishing](#publishing)
12. [Cross-Compilation](#cross-compilation)
13. [Configuration](#configuration)
14. [Best Practices](#best-practices)

---

## What is Cargo?

**Reference**: [Cargo Book - Introduction](https://doc.rust-lang.org/cargo/index.html)

Cargo is Rust's official:
- **Build system** - Compiles your code
- **Package manager** - Downloads and manages dependencies
- **Test runner** - Runs your tests
- **Documentation generator** - Creates API docs
- **Publishing tool** - Publishes to crates.io

### Why Cargo?

✅ **Automatic dependency management** - No manual downloading
✅ **Reproducible builds** - Same code = same result
✅ **Zero-config for simple projects** - Convention over configuration
✅ **Powerful for complex projects** - Workspaces, features, profiles
✅ **Integrated tooling** - Test, doc, publish all in one

---

## Project Structure

**Reference**: [Cargo Book - Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)

### Standard Layout

```
my-project/
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Locked dependencies (commit for binaries)
├── src/
│   ├── main.rs            # Binary crate root
│   ├── lib.rs             # Library crate root
│   └── bin/               # Additional binaries
│       └── tool.rs
├── tests/                 # Integration tests
│   └── integration_test.rs
├── benches/               # Benchmarks
│   └── benchmark.rs
├── examples/              # Example programs
│   └── example.rs
├── target/                # Build output (don't commit!)
│   ├── debug/
│   └── release/
└── .cargo/                # Project-specific cargo config
    └── config.toml
```

### What Goes Where

| Location | Purpose | Example |
|----------|---------|---------|
| `src/main.rs` | Default binary | Application entry point |
| `src/lib.rs` | Library root | Reusable code |
| `src/bin/*.rs` | Additional binaries | CLI tools |
| `tests/*.rs` | Integration tests | Full API tests |
| `benches/*.rs` | Benchmarks | Performance tests |
| `examples/*.rs` | Usage examples | Documentation examples |
| `build.rs` | Build script | Codegen, C bindings |

---

## Cargo.toml Manifest

**Reference**: [Cargo Book - The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)

### Basic Structure

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
description = "A short description"
homepage = "https://example.com/my-project"
repository = "https://github.com/user/my-project"
readme = "README.md"
keywords = ["cli", "tool"]
categories = ["command-line-utilities"]

[dependencies]
serde = "1.0"
tokio = { version = "1.35", features = ["full"] }

[dev-dependencies]
criterion = "0.5"

[build-dependencies]
cc = "1.0"

[[bin]]
name = "my-tool"
path = "src/bin/tool.rs"
```

### Package Metadata

```toml
[package]
# Required
name = "my-project"          # Package name (kebab-case)
version = "0.1.0"            # Semver version
edition = "2021"             # Rust edition (2015, 2018, 2021)

# Recommended
authors = ["Name <email>"]
description = "Short description"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/repo"

# Optional
homepage = "https://example.com"
documentation = "https://docs.rs/my-project"
readme = "README.md"
keywords = ["cli", "web"]    # Max 5
categories = ["web-programming"]
publish = true               # Can publish to crates.io

# Rust version requirement
rust-version = "1.70"        # Minimum Rust version

# Exclude/include files
exclude = ["tests/fixtures/*"]
include = ["src/**/*", "Cargo.toml"]
```

### Dependency Specification

**Reference**: [Cargo Book - Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)

```toml
[dependencies]
# Version from crates.io
serde = "1.0"                    # ^1.0 (>= 1.0.0, < 2.0.0)
serde = "1.0.100"                # ^1.0.100 (>= 1.0.100, < 2.0.0)
serde = "=1.0.100"               # Exactly 1.0.100

# With features
tokio = { version = "1.35", features = ["full"] }

# Optional (for features)
serde = { version = "1.0", optional = true }

# Git dependency
my-lib = { git = "https://github.com/user/repo" }
my-lib = { git = "https://github.com/user/repo", branch = "main" }
my-lib = { git = "https://github.com/user/repo", tag = "v1.0" }
my-lib = { git = "https://github.com/user/repo", rev = "abc123" }

# Local path
my-lib = { path = "../my-lib" }

# From registry
my-lib = { version = "1.0", registry = "my-registry" }

# Platform-specific
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

### Version Requirements

| Syntax | Range | Example |
|--------|-------|---------|
| `1.2.3` | `^1.2.3` | `>= 1.2.3, < 2.0.0` |
| `^1.2.3` | Caret | `>= 1.2.3, < 2.0.0` |
| `~1.2.3` | Tilde | `>= 1.2.3, < 1.3.0` |
| `= 1.2.3` | Exact | Exactly `1.2.3` |
| `>= 1.2` | Greater/equal | `>= 1.2.0` |
| `< 2` | Less than | `< 2.0.0` |
| `*` | Wildcard | Any version |

**Idiom**: Use caret `^` (default) for most dependencies. Use exact `=` only for critical dependencies.

---

## Common Commands

**Reference**: [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/index.html)

### Building and Running

```bash
# Create new project
cargo new my-project              # Binary (default)
cargo new --lib my-library        # Library
cargo new --vcs none              # Without git

# Initialize in existing directory
cargo init                        # Binary
cargo init --lib                  # Library

# Build
cargo build                       # Debug build (fast, unoptimized)
cargo build --release             # Release build (slow, optimized)
cargo build --target x86_64-pc-windows-gnu  # Cross-compile

# Run
cargo run                         # Build and run
cargo run --release               # Run optimized
cargo run -- arg1 arg2            # Pass arguments
cargo run --bin my-tool           # Run specific binary

# Check (fast, no executable)
cargo check                       # Type check only
cargo check --all-targets         # Check tests, benches too
```

### Testing

```bash
# Run tests
cargo test                        # Run all tests
cargo test test_name              # Run specific test
cargo test --lib                  # Only library tests
cargo test --doc                  # Doc tests only
cargo test --test integration     # Specific integration test
cargo test -- --nocapture         # Show println! output
cargo test -- --test-threads=1    # Run tests serially
cargo test -- --ignored           # Run ignored tests

# Benchmarks
cargo bench                       # Run benchmarks
cargo bench my_bench              # Specific benchmark
```

### Documentation

```bash
# Generate docs
cargo doc                         # Generate documentation
cargo doc --open                  # Generate and open
cargo doc --no-deps               # Skip dependencies
cargo doc --document-private-items  # Include private items

# Search docs
cargo doc --open std              # Open std library docs
```

### Maintenance

```bash
# Clean
cargo clean                       # Remove target/

# Update dependencies
cargo update                      # Update to latest compatible
cargo update -p serde             # Update specific package

# Dependency tree
cargo tree                        # Show dependency tree
cargo tree --duplicates           # Show duplicates
cargo tree -i serde               # Inverse (who depends on serde)

# Audit
cargo audit                       # Check for vulnerabilities (requires cargo-audit)
```

### Code Quality

```bash
# Format
cargo fmt                         # Format all code
cargo fmt --check                 # Check if formatted

# Lint
cargo clippy                      # Run linter
cargo clippy --fix                # Auto-fix issues
cargo clippy -- -D warnings       # Warnings as errors

# Expand macros
cargo expand                      # Show macro expansion (requires cargo-expand)
```

### Publishing

```bash
# Package
cargo package                     # Create .crate file
cargo package --list              # List packaged files
cargo package --allow-dirty       # Package with uncommitted changes

# Publish
cargo login <token>               # Authenticate
cargo publish                     # Publish to crates.io
cargo publish --dry-run           # Test publish
cargo yank --vers 1.0.0           # Yank published version
```

---

## Dependencies

**Reference**: [Cargo Book - Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)

### Adding Dependencies

```bash
# Add from crates.io
cargo add serde                   # Latest version
cargo add serde@1.0               # Specific version
cargo add serde --features derive # With features
cargo add serde --optional        # Optional dependency

# Add dev dependency
cargo add --dev criterion         # For tests/benches

# Add build dependency
cargo add --build cc              # For build.rs

# Remove dependency
cargo rm serde
```

### Cargo.lock

**Reference**: [Cargo Book - Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)

**For Libraries:**
- ❌ Don't commit `Cargo.lock`
- Let users decide versions

**For Binaries/Applications:**
- ✅ Commit `Cargo.lock`
- Ensures reproducible builds

```bash
# Update all dependencies
cargo update

# Update specific package
cargo update -p serde

# Update to breaking versions
# Edit Cargo.toml manually, then:
cargo update
```

---

## Features

**Reference**: [Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html)

Features allow conditional compilation and optional dependencies.

### Defining Features

```toml
[features]
default = ["json", "xml"]        # Enabled by default
json = ["serde_json"]            # Feature named "json"
xml = ["quick-xml"]              # Feature named "xml"
full = ["json", "xml", "yaml"]   # Meta-feature

[dependencies]
serde = "1.0"
serde_json = { version = "1.0", optional = true }
quick-xml = { version = "0.31", optional = true }
```

### Using Features

```rust
// In your code
#[cfg(feature = "json")]
pub fn parse_json(s: &str) -> Result<Value> {
    serde_json::from_str(s)
}

#[cfg(not(feature = "json"))]
pub fn parse_json(s: &str) -> Result<Value> {
    Err("JSON support not enabled".into())
}
```

### Building with Features

```bash
# Default features
cargo build

# No default features
cargo build --no-default-features

# Specific features
cargo build --features json

# Multiple features
cargo build --features "json,xml"

# All features
cargo build --all-features
```

### Common Patterns

```toml
[features]
# Meta-feature for everything
full = ["json", "xml", "yaml", "toml"]

# Enable by default, allow opt-out
default = ["std"]
std = []  # Empty feature, used for #[cfg]

# Optional dependencies as features
serde = ["dep:serde"]  # Explicit optional dependency
```

---

## Workspaces

**Reference**: [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

Already covered in detail in [01-workspaces.md](./01-workspaces.md), but here's a quick reference:

### Workspace Root

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "crate-a",
    "crate-b",
    "crate-c",
]
exclude = ["old-crate"]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Name <email>"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
```

### Workspace Commands

```bash
# Build all packages
cargo build --workspace

# Test all packages
cargo test --workspace

# Build specific package
cargo build -p crate-a

# Run from specific package
cargo run -p crate-a
```

---

## Build Profiles

**Reference**: [Cargo Book - Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)

Profiles control compilation settings.

### Default Profiles

```toml
# Debug profile (cargo build)
[profile.dev]
opt-level = 0              # No optimization
debug = true               # Include debug info
overflow-checks = true     # Runtime checks

# Release profile (cargo build --release)
[profile.release]
opt-level = 3              # Maximum optimization
debug = false              # No debug info
lto = false                # Link-time optimization
codegen-units = 16         # Parallel codegen

# Test profile
[profile.test]
opt-level = 0
debug = true

# Bench profile
[profile.bench]
opt-level = 3
debug = false
```

### Custom Profiles

```toml
# Custom profile for production
[profile.production]
inherits = "release"
opt-level = 3
lto = "fat"                # Full LTO
codegen-units = 1          # Single codegen unit
strip = true               # Strip symbols
panic = "abort"            # Abort on panic

# Build with custom profile
# cargo build --profile production
```

### Optimization Levels

| Level | Optimization | Build Time | Binary Size | Performance |
|-------|-------------|------------|-------------|-------------|
| 0 | None | Fast | Large | Slow |
| 1 | Basic | Fast | Large | Medium |
| 2 | Medium | Medium | Medium | Good |
| 3 | Max | Slow | Small | Best |
| "s" | Size | Slow | Smallest | Good |
| "z" | Size (aggressive) | Slow | Smallest | Medium |

### Dependency Optimization

```toml
# Optimize dependencies even in dev builds
[profile.dev.package."*"]
opt-level = 2

# Specific package optimization
[profile.dev.package.my-slow-dependency]
opt-level = 3
```

---

## Testing

**Reference**: [Cargo Book - Tests](https://doc.rust-lang.org/cargo/guide/tests.html)

### Unit Tests

```rust
// In src/lib.rs or src/main.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("This should panic");
    }

    #[test]
    #[ignore]
    fn expensive_test() {
        // Run with: cargo test -- --ignored
    }
}
```

### Integration Tests

```rust
// In tests/integration_test.rs
use my_crate::some_function;

#[test]
fn test_public_api() {
    let result = some_function();
    assert!(result.is_ok());
}
```

### Doc Tests

```rust
/// Adds two numbers
///
/// # Examples
///
/// ```
/// let result = my_crate::add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Test Commands

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests matching pattern
cargo test add_

# Run single-threaded
cargo test -- --test-threads=1

# Show ignored tests
cargo test -- --ignored
```

---

## Documentation

**Reference**: [rustdoc Book](https://doc.rust-lang.org/rustdoc/index.html)

### Doc Comments

```rust
/// Brief one-line description
///
/// Longer detailed description with multiple paragraphs.
///
/// # Examples
///
/// ```
/// use my_crate::example;
/// let result = example();
/// ```
///
/// # Errors
///
/// Returns error if...
///
/// # Panics
///
/// Panics if...
///
/// # Safety
///
/// (For unsafe functions) This function is unsafe because...
pub fn example() -> Result<(), Error> {
    Ok(())
}

//! Module-level documentation
//!
//! This module provides...
```

### Documentation Commands

```bash
# Generate docs
cargo doc                         # Build docs
cargo doc --open                  # Build and open
cargo doc --no-deps               # Skip dependencies
cargo doc --document-private-items # Include private

# Test docs
cargo test --doc                  # Run doc tests
```

---

## Publishing

**Reference**: [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)

### Prepare for Publishing

1. **Choose License**
```toml
[package]
license = "MIT OR Apache-2.0"    # Dual license (recommended)
# OR
license-file = "LICENSE"          # Custom license
```

2. **Add Metadata**
```toml
[package]
description = "A short description of your crate"
homepage = "https://example.com"
repository = "https://github.com/user/project"
readme = "README.md"
keywords = ["web", "api", "json"]  # Max 5
categories = ["web-programming"]
```

3. **Check Package**
```bash
cargo package --list              # See what will be included
cargo package                     # Create .crate file
```

### Publish to crates.io

```bash
# Login (get token from https://crates.io/me)
cargo login <your-token>

# Dry run
cargo publish --dry-run

# Publish
cargo publish

# Yank (remove from new projects, existing work)
cargo yank --vers 1.0.0

# Undo yank
cargo yank --vers 1.0.0 --undo
```

### Version Management

**Semantic Versioning (SemVer):**
- `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

```bash
# Update version in Cargo.toml, then:
cargo publish
```

---

## Cross-Compilation

**Reference**: [Rustup Book - Cross-compilation](https://rust-lang.github.io/rustup/cross-compilation.html)

Cross-compilation allows building for different platforms (Windows, Linux, macOS, ARM, etc.) from a single machine.

### Architecture Overview

```
Host System (macOS)
    ↓ Build for →
Target System (Linux x86_64, Windows, ARM, etc.)
```

### Platform Targets

**Common targets:**

| Target Triple | Platform | Architecture |
|---------------|----------|--------------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 (glibc) |
| `x86_64-unknown-linux-musl` | Linux | x86_64 (musl, static) |
| `x86_64-pc-windows-gnu` | Windows | x86_64 (GNU) |
| `x86_64-pc-windows-msvc` | Windows | x86_64 (MSVC) |
| `x86_64-apple-darwin` | macOS | x86_64 (Intel) |
| `aarch64-apple-darwin` | macOS | ARM64 (Apple Silicon) |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 |
| `armv7-unknown-linux-gnueabihf` | Linux | ARMv7 |
| `wasm32-unknown-unknown` | WebAssembly | WASM |

**View all targets:**
```bash
rustc --print target-list
```

### Installing Cross-Compilation Targets

```bash
# Add target with rustup
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin

# List installed targets
rustup target list --installed

# List all available targets
rustup target list
```

### Cross-Compiling

```bash
# Build for specific target
cargo build --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Binary location
# target/<target-triple>/release/my-binary
```

### Cross-Compilation Setup by Platform

#### From macOS

**To Linux (x86_64):**
```bash
# Install target
rustup target add x86_64-unknown-linux-gnu

# Install linker
brew install filosottile/musl-cross/musl-cross

# Build
cargo build --target x86_64-unknown-linux-gnu
```

**To Windows:**
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install linker (MinGW)
brew install mingw-w64

# Build
cargo build --target x86_64-pc-windows-gnu
```

**To Linux (musl, static binary):**
```bash
# Install target
rustup target add x86_64-unknown-linux-musl

# Build (no extra linker needed!)
cargo build --target x86_64-unknown-linux-musl
```

#### From Linux

**To Windows:**
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install linker
sudo apt install mingw-w64

# Build
cargo build --target x86_64-pc-windows-gnu
```

**To macOS:** (Complex, use Docker or VM)
```bash
# Requires osxcross toolchain
# See: https://github.com/tpoechtrager/osxcross
```

#### From Windows

**To Linux:**
```bash
# Install target
rustup target add x86_64-unknown-linux-gnu

# Requires WSL or Docker for linker
# Or use cross tool (see below)
```

### Using `cross` Tool

**Reference**: https://github.com/cross-rs/cross

`cross` is a Docker-based tool that simplifies cross-compilation.

```bash
# Install
cargo install cross

# Use like cargo but with cross
cross build --target x86_64-unknown-linux-gnu
cross build --target aarch64-unknown-linux-gnu
cross test --target armv7-unknown-linux-gnueabihf

# Supports most targets without manual setup!
```

### Configuration for Cross-Compilation

#### Cargo Config

`.cargo/config.toml`:
```toml
# Specify linker for target
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-gnu-gcc"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

# Default target
[build]
target = "x86_64-unknown-linux-musl"
```

#### Build Script for Cross-Compilation

`build.rs`:
```rust
fn main() {
    let target = std::env::var("TARGET").unwrap();
    
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=ws2_32");
    }
    
    if target.contains("musl") {
        println!("cargo:rustc-link-arg=-static");
    }
}
```

### Platform-Specific Dependencies

```toml
# In Cargo.toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser"] }

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

### Static Compilation

For fully static binaries (no runtime dependencies):

```bash
# Use musl target
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Binary is now fully static!
ldd target/x86_64-unknown-linux-musl/release/my-binary
# Output: not a dynamic executable
```

### Multi-Platform Releases

```bash
#!/bin/bash
# Build for multiple platforms

# Linux (musl for static binary)
cargo build --release --target x86_64-unknown-linux-musl

# Windows
cargo build --release --target x86_64-pc-windows-gnu

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Package binaries
mkdir -p release
cp target/x86_64-unknown-linux-musl/release/my-app release/my-app-linux-x64
cp target/x86_64-pc-windows-gnu/release/my-app.exe release/my-app-windows-x64.exe
cp target/x86_64-apple-darwin/release/my-app release/my-app-macos-x64
cp target/aarch64-apple-darwin/release/my-app release/my-app-macos-arm64
```

### CI/CD Example (GitHub Actions)

```yaml
name: Build
on: [push]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: rustup target add ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
```

### Troubleshooting Cross-Compilation

**Missing linker:**
```bash
# Error: linker 'x86_64-linux-gnu-gcc' not found
# Solution: Install cross-compilation toolchain
brew install filosottile/musl-cross/musl-cross  # macOS
sudo apt install gcc-x86-64-linux-gnu           # Linux
```

**OpenSSL issues:**
```toml
# Use rustls instead of openssl for easier cross-compilation
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls"] }
```

**C dependencies:**
```bash
# Use 'cross' tool for complex C dependencies
cargo install cross
cross build --target x86_64-unknown-linux-gnu
```

---

## Configuration

**Reference**: [Cargo Book - Configuration](https://doc.rust-lang.org/cargo/reference/config.html)

Cargo can be configured at multiple levels:

### Config Hierarchy

1. `.cargo/config.toml` (project-specific)
2. `~/.cargo/config.toml` (user-level)
3. `$CARGO_HOME/config.toml` (global)
4. Environment variables

### Common Configuration

`.cargo/config.toml`:
```toml
# Default build target
[build]
target = "x86_64-unknown-linux-musl"
target-dir = "target"
jobs = 4                     # Parallel build jobs

# Incremental compilation
incremental = true

# Linker configuration
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Registry aliases
[registries.my-registry]
index = "https://my-registry.com/git/index"

# Network settings
[net]
retry = 3
git-fetch-with-cli = true

# HTTP settings
[http]
proxy = "http://proxy.example.com:8080"
timeout = 60

# Terminal settings
[term]
verbose = false
color = "auto"              # auto, always, never
```

### Environment Variables

```bash
# Rust compiler
export RUSTC=/path/to/rustc
export RUSTC_WRAPPER=sccache  # Caching compiler

# Build settings
export CARGO_BUILD_JOBS=4
export CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
export CARGO_TARGET_DIR=/tmp/target

# Features
export CARGO_BUILD_FEATURES="json xml"

# Network
export CARGO_HTTP_PROXY=http://proxy:8080
export CARGO_NET_RETRY=3

# Home directory
export CARGO_HOME=/custom/cargo

# Offline mode
export CARGO_OFFLINE=true
```

---

## Best Practices

### Project Structure

✅ **Do:**
- Keep `src/main.rs` thin (5-20 lines)
- Put logic in `src/lib.rs`
- Use `tests/` for integration tests
- Add examples in `examples/`

❌ **Don't:**
- Put business logic in `main.rs`
- Mix test types in same file
- Commit `target/` directory

### Dependencies

✅ **Do:**
- Use workspace dependencies for consistency
- Pin versions for production apps
- Use `cargo update` regularly
- Audit dependencies (`cargo audit`)

❌ **Don't:**
- Use `*` for versions
- Ignore `Cargo.lock` for binaries
- Add unnecessary dependencies
- Trust unmaintained crates

### Versioning

✅ **Do:**
- Follow SemVer strictly
- Document breaking changes
- Test before publishing
- Use `cargo publish --dry-run`

❌ **Don't:**
- Break compatibility in PATCH
- Yank versions frequently
- Publish without testing

### Performance

✅ **Do:**
- Use release builds for production
- Enable LTO for final builds
- Profile before optimizing
- Use `cargo bench` for benchmarks

❌ **Don't:**
- Optimize prematurely
- Use debug builds in production
- Ignore compile time
- Optimize without measuring

### Security

✅ **Do:**
- Run `cargo audit` regularly
- Keep dependencies updated
- Review dependency tree
- Use `cargo deny` for policies

❌ **Don't:**
- Ignore vulnerability warnings
- Use deprecated crates
- Trust without verification
- Skip security updates

---

## Useful Cargo Plugins

**Install with `cargo install <plugin>`:**

| Plugin | Purpose |
|--------|---------|
| `cargo-edit` | Add/remove dependencies easily |
| `cargo-watch` | Auto-rebuild on file changes |
| `cargo-expand` | Show macro expansions |
| `cargo-audit` | Check for vulnerabilities |
| `cargo-outdated` | Find outdated dependencies |
| `cargo-tree` | Visualize dependency tree |
| `cargo-bloat` | Find what takes space |
| `cargo-deny` | Lint dependencies |
| `cargo-geiger` | Detect unsafe code |
| `cross` | Easy cross-compilation |
| `cargo-chef` | Docker layer caching |
| `cargo-make` | Task runner |

```bash
# Install common plugins
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-audit
cargo install cross
```

---

## Quick Reference

### Essential Commands

```bash
cargo new my-project          # Create project
cargo build                   # Debug build
cargo build --release         # Release build
cargo run                     # Build and run
cargo test                    # Run tests
cargo check                   # Fast check
cargo clippy                  # Lint
cargo fmt                     # Format
cargo doc --open              # Generate docs
cargo add serde               # Add dependency
cargo update                  # Update deps
cargo tree                    # Show dep tree
```

### Build Targets

```bash
rustup target list            # Available targets
rustup target add <target>    # Install target
cargo build --target <target> # Cross-compile
```

### Workspace Commands

```bash
cargo build --workspace       # Build all
cargo test --workspace        # Test all
cargo build -p <package>      # Build one
```

---

## Summary

Cargo is your Swiss Army knife for Rust development:

✅ **Builds** your code (debug and release)
✅ **Manages** dependencies automatically
✅ **Runs** tests and benchmarks
✅ **Generates** documentation
✅ **Publishes** to crates.io
✅ **Cross-compiles** to many platforms
✅ **Configures** with profiles and features

**Key Takeaways:**
- Start with `cargo new`, follow conventions
- Use `Cargo.toml` for all configuration
- Commit `Cargo.lock` for binaries
- Use workspaces for multi-package projects
- Cross-compile with targets and `cross`
- Optimize with profiles
- Test often with `cargo test`
- Document with `cargo doc`

For our RustMart project, we'll use:
- Workspace for multiple services
- Cross-compilation for deployments
- Features for optional functionality
- Profiles for dev/prod optimization

## Next Steps

With Cargo knowledge, you can now:
- Build and organize Rust projects
- Manage dependencies effectively
- Create reproducible builds
- Cross-compile for deployment
- Publish libraries to crates.io
