# Rust Learning Journey - RustMart

This document tracks the lessons and concepts covered while building RustMart.

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

## 🔜 Next Topics to Cover

- [ ] Rust module system and code organization
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
