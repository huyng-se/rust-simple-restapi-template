# Rust API Service Template

A production-oriented Rust API service template built with **Axum**, **SeaORM**, **PostgreSQL**, **Redis**, **Authentication**, and **Flyway-based raw SQL migrations**.

This template is designed for teams that want a backend service structure that is:

- Simple to understand
- Easy to maintain
- Explicit in dependency wiring
- Production-friendly
- Modular like framework-based services, but still Rust-native
- Flexible enough for real-world API development

---

## 1. Overview

This project provides a clean foundation for building API services in Rust.

The architecture follows a lightweight modular approach inspired by framework-style modules:

- `ApiModule` convention
- Explicit `AppState`
- Constructor-based dependency injection
- Service and repository traits
- SeaORM for runtime database access
- Flyway for database migrations outside the Rust application

The Rust application does **not** run database migrations by itself.  
Database schema changes are managed independently through raw SQL migration files and executed by Flyway.

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| HTTP Framework | Axum |
| Async Runtime | Tokio |
| Middleware | Tower / tower-http |
| ORM | SeaORM |
| Database | PostgreSQL |
| Cache / Session Store | Redis |
| Authentication | JWT Access Token + Refresh Token |
| Password Hashing | Argon2 |
| Migration Tool | Flyway |
| Migration Format | Raw SQL |
| Logging / Tracing | tracing |
| OpenAPI | utoipa |
| Containerization | Docker / Docker Compose |

---

## 3. Key Design Decisions

### 3.1 SeaORM for Runtime Database Access

SeaORM is used for application runtime data access, including:

- Entity modeling
- CRUD operations
- Query building
- Repository implementation
- Relation handling

This keeps application code clean and easier to maintain compared to writing raw SQL for every query.

### 3.2 Raw SQL Migration with Flyway

Database migrations are handled by Flyway using plain SQL files.

This provides:

- Clear schema history
- Easy review in pull requests
- Better PostgreSQL-specific control
- No dependency on Rust migration crates
- Safer production deployment flow

### 3.3 Explicit Dependency Injection

This template does not use a runtime DI container.

Instead, dependencies are wired explicitly through constructors and stored in `AppState`.

This keeps the architecture:

- Easy to debug
- Compile-time friendly
- Transparent
- Suitable for long-term maintenance

### 3.4 Module Convention

Each feature module owns its own:

- Routes
- Handlers
- DTOs
- Service logic
- Repository contract

Example:

```txt
modules/
├── auth/
├── user/
└── health/
