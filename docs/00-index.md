# Rust Learning Journey - RustMart

This document tracks the lessons and concepts covered while building RustMart.

## 📍 Current Status

**Last Updated:** 2025-11-20
**Current Phase:** Building microservices
**Next Step:** Add OpenTelemetry instrumentation for IBM demo

**Note:** When reopening Warp, say "Continue from docs/00-index.md" to resume.

## 🎯 Learning Goals
- Build a production-ready microservices application in Rust
- Learn async/await patterns with Tokio
- Understand error handling and Result types
- Work with databases, caching, and message queues
- Implement observability and tracing
- Deploy to Docker and Kubernetes

---

## 📚 Lessons Completed

### ✅ Lesson 1: Cargo Workspaces (2025-11-19)
**File:** [01-workspaces.md](./01-workspaces.md)

**Concepts covered:**
- Cargo workspace architecture and benefits
- `[workspace.package]` for shared metadata
- `[workspace.dependencies]` for version consistency
- Single `target/` directory and compilation cache
- Member package configuration with `.workspace = true`
- Build process and dependency resolution
- Workspace-wide commands (`--workspace`, `-p`)

**What we built:**
- Root workspace with 5 microservices + shared library
- Workspace dependencies (tokio, axum, sqlx, etc.)
- Empty member packages ready for implementation

**Key takeaways:**
- Workspaces save disk space and speed up builds dramatically
- Single source of truth for dependency versions prevents conflicts
- Workspace members must have valid `Cargo.toml` files
- Git submodules/nested repos can cause issues (resolved by removing `shared/.git`)

---

### ✅ Lesson 2: Project Structure - lib.rs vs main.rs (2025-11-19)
**File:** [02-project-structure.md](./02-project-structure.md)

**Concepts covered:**
- Binary vs library crates
- "Thin binary, fat library" pattern
- Project layout and module organization
- `pub`, `pub(crate)`, and visibility modifiers
- Testing patterns (unit vs integration)
- Official Cargo project layout guidelines

**Key takeaways:**
- main.rs should be 5-20 lines - just setup and calling lib.rs
- All business logic goes in lib.rs and modules
- This pattern enables testing, reusability, and documentation
- Cargo auto-detects both lib.rs and main.rs

---

### ✅ Lesson 3: Cargo - The Rust Build Tool (2025-11-19)
**File:** [03-cargo-guide.md](./03-cargo-guide.md)

**Concepts covered:**
- Cargo as build system, package manager, and toolchain
- Project structure and Cargo.toml manifest
- Dependency management and features
- Build profiles and optimization
- Cross-compilation to multiple platforms
- Testing, documentation, and publishing
- Configuration and best practices

**Key takeaways:**
- Cargo automates building, testing, and dependency management
- Use workspaces for multi-package projects
- Cross-compile with rustup targets or `cross` tool
- Commit Cargo.lock for binaries, not libraries
- Use profiles to optimize for different scenarios

---

### ✅ Lesson 4: Error Handling in Rust (2025-11-19)
**File:** [04-error-handling.md](./04-error-handling.md)

**Concepts covered:**
- Option<T> for values that might not exist
- Result<T, E> for operations that can fail
- The ? operator for error propagation
- Custom error types with enums
- thiserror for library errors
- anyhow for application errors
- panic! for unrecoverable errors
- Best practices and patterns

**Key takeaways:**
- Rust has no exceptions - errors are values
- Use Result for recoverable errors, panic! for bugs
- ? operator is idiomatic for propagating errors
- thiserror makes custom errors easy
- anyhow simplifies error handling in applications

---

### ✅ Lesson 5: Building the Shared Library (2025-11-19)
**File:** [05-shared-library.md](./05-shared-library.md)

**Concepts covered:**
- Module organization and structure
- Creating library crates
- Custom error types with thiserror
- Domain model design (Product, User, Order)
- Serde for JSON serialization
- Public vs private visibility
- Re-exports for ergonomic APIs
- Writing and running tests

**What we built:**
- shared library with error types, models, and config
- ApiError enum with thiserror
- Product, User, Order, OrderItem structs
- DatabaseConfig and ServerConfig
- Tests for models and JSON serialization

**Key takeaways:**
- Shared libraries enable code reuse across microservices
- Use pub mod to make modules public
- Re-export types at crate root for easier imports
- Everything is private by default in Rust
- use super::* imports from parent module (common in tests)
- Serde makes JSON serialization straightforward

---

### ✅ Lesson 6: Building Product Service with Axum (2025-11-20)
**File:** [06-product-service.md](./06-product-service.md)

**Concepts covered:**
- Async programming with Tokio
- Building HTTP APIs with Axum
- Router and request handlers
- Path parameter extraction
- JSON serialization with Axum
- Error handling in web services
- IntoResponse trait implementation
- Tracing and logging

**What we built:**
- First working microservice (product-service)
- Health check endpoint
- REST API endpoints (GET /products, GET /products/:id)
- Error responses with proper HTTP status codes
- Integration with shared library

**Key takeaways:**
- async/await enables non-blocking I/O
- Axum uses extractors (Path, Json, etc.) to parse requests
- IntoResponse trait converts types to HTTP responses
- Shared error types work across all services
- Tokio runtime manages async execution

---

## 🔜 Next Topics to Cover

- [ ] Common types and error handling patterns
- [ ] Async/await and Tokio runtime basics
- [ ] Building HTTP APIs with Axum
- [ ] Database operations with SQLx
- [ ] Trait implementations for shared behavior
- [ ] Testing strategies (unit, integration, mocking)
- [ ] Error propagation with `?` operator
- [ ] JSON serialization with Serde
- [ ] Environment configuration
- [ ] Structured logging with tracing
- [ ] OpenTelemetry instrumentation
- [ ] Docker containerization
- [ ] Kubernetes deployment

---

## 💡 Questions & Notes

*Add any questions or observations as we go...*
