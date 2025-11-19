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

### Lesson 1: Workspace Setup (2025-11-19)
**Concepts:**
- Cargo workspaces for monorepo structure
- Workspace-level dependencies and configuration
- Using `workspace = true` to inherit package metadata

**What we did:**
- Created workspace with 5 services + shared library
- Set up workspace dependencies in root `Cargo.toml`
- Initialized empty service directories with minimal `Cargo.toml` files

**Key takeaways:**
- Workspaces allow multiple packages to share dependencies and build settings
- Workspace members must have a valid `Cargo.toml` or Cargo will error
- Git submodules/nested repos can cause issues with workspace members

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
