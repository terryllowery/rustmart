# Rust Learning Journey - RustMart

This document tracks the lessons and concepts covered while building RustMart.

## 📍 Current Status

**Last Updated:** 2025-11-19
**Current Phase:** Learning fundamentals
**Next Step:** Choose between:
1. Start implementing shared library (error types, common models)
2. Build first microservice (product-service)
3. Learn more Rust concepts

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
