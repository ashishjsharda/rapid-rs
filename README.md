# 🚀 rapid-rs

> **Zero-config, batteries-included web framework for Rust**  
> FastAPI meets Spring Boot, powered by Axum

[![Crates.io](https://img.shields.io/crates/v/rapid-rs.svg)](https://crates.io/crates/rapid-rs)
[![Documentation](https://docs.rs/rapid-rs/badge.svg)](https://docs.rs/rapid-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 🆕 What's New in v0.3.0

**Phase 2 Complete!** 🎉

- 🗄️ **Database Migrations** - Automatic migration management with sqlx
- 🧪 **Testing Utilities** - Comprehensive testing framework for APIs
- 📚 **GraphQL & gRPC Templates** - Ready-to-use project scaffolding

```rust
// Database with auto-migrations
use rapid_rs::database::{connect_and_migrate, MigrationConfig};

let pool = connect_and_migrate(
    "postgres://localhost/myapp",
    MigrationConfig::default()
).await?;
```

```rust
// Easy API testing
use rapid_rs::testing::TestClient;

let client = TestClient::new(app);
let response = client.get("/users").await;
response.assert_status(StatusCode::OK);
```

[See full changelog →](CHANGELOG.md)

---

## Why rapid-rs?

Building web APIs in Rust shouldn't require wiring together 10+ crates and writing hundreds of lines of boilerplate. **rapid-rs** gives you the productivity of FastAPI and Spring Boot, with Rust's performance and type safety.

### ⚡ Features

- 🎯 **Zero Config** - Database, migrations, CORS, logging work out of the box
- 🗄️ **Auto Migrations** - Database migrations run automatically on startup (NEW!)
- 🔐 **Built-in Auth** - JWT authentication, password hashing, RBAC
- 🧪 **Testing Suite** - Comprehensive testing utilities included (NEW!)
- 🔒 **Type-Safe** - Compile-time guarantees for routes, validation, serialization
- 📚 **Auto Docs** - Swagger UI and OpenAPI specs from your code
- ✅ **Validation** - Request validation with helpful error messages
- 🔥 **Hot Reload** - Fast development with `rapid dev`
- 🎨 **Opinionated** - Convention over configuration
- 🚀 **Production Ready** - Structured logging, error handling, health checks

## Quick Start

### Installation
```bash
cargo install rapid-rs-cli
```

### Create Your First API
```bash
# REST API (default)
rapid new myapi

# GraphQL API (NEW!)
rapid new mygraphql --template graphql

# gRPC service (NEW!)
rapid new mygrpc --template grpc

cd myapi
cargo run
```

Your API is now running at:
- 🌐 **http://localhost:8080** - API endpoints
- 📚 **http://localhost:8080/docs** - Swagger UI
- 💚 **http://localhost:8080/health** - Health check

### Your First Endpoint with Database

```rust
use rapid_rs::prelude::*;
use rapid_rs::database::{connect_and_migrate, MigrationConfig};

#[derive(Serialize, Deserialize, Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,
    #[validate(length(min = 2))]
    name: String,
}

async fn create_user(
    State(pool): State<PgPool>,
    ValidatedJson(payload): ValidatedJson<CreateUser>
) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *",
        payload.email,
        payload.name
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    // Auto-create database and run migrations
    let pool = connect_and_migrate(
        "postgres://localhost/myapp",
        MigrationConfig::default()
    ).await.unwrap();
    
    App::new()
        .auto_configure()
        .route("/users", post(create_user))
        .with_state(pool)
        .run()
        .await
        .unwrap();
}
```

That's it! You get:
- ✅ Automatic database creation
- ✅ Automatic migration running
- ✅ Request validation
- ✅ Type-safe database queries
- ✅ Structured error handling
- ✅ OpenAPI documentation

## Comparison

| Feature | FastAPI | Spring Boot | **rapid-rs** |
|---------|---------|-------------|--------------|
| Type Safety | ❌ Runtime | ⚠️ Runtime | ✅ Compile-time |
| Auto OpenAPI | ✅ | ✅ | ✅ |
| Auto Migrations | ⚠️ Alembic | ✅ | ✅ |
| Testing Utils | ⚠️ Partial | ✅ | ✅ |
| Hot Reload | ✅ | ✅ | ✅ |
| Zero Config | ✅ | ✅ | ✅ |
| Performance | ⚠️ Good | ⚠️ Good | ✅ Blazing Fast |
| Memory Safety | ❌ | ❌ | ✅ Guaranteed |
| Async by Default | ⚠️ Partial | ❌ | ✅ |
| Learning Curve | Easy | Medium | Easy |

## What's Included?

### 🎁 Out of the Box

- **Configuration** - TOML files + environment variables
- **Authentication** - JWT-based auth with role-based access control
- **Database** - PostgreSQL with connection pooling and migrations
- **Validation** - Derive-based validation with helpful errors
- **Error Handling** - Centralized error handling with proper HTTP codes
- **CORS** - Sensible defaults, fully configurable
- **Logging** - Structured logging with request correlation
- **Health Checks** - `/health` endpoint for orchestration
- **OpenAPI/Swagger** - Auto-generated docs at `/docs`
- **Testing** - Comprehensive testing utilities (NEW!)

### 📦 CLI Tool
```bash
# Create project with template
rapid new myapi --template rest-api|graphql|grpc

# Run with hot reload
rapid dev
```

### 📚 Project Templates

- **REST API** - Full CRUD with authentication and database
- **GraphQL API** - async-graphql with subscriptions support
- **gRPC Service** - tonic-based microservice template

## Database Migrations

Migrations run automatically on startup:

```rust
use rapid_rs::database::{connect_and_migrate, MigrationConfig};

let config = MigrationConfig::new()
    .migrations_path("./migrations")
    .auto_migrate(true)
    .create_db_if_missing(true);

let pool = connect_and_migrate(&database_url, config).await?;
```

Create migrations with sqlx-cli:
```bash
cargo install sqlx-cli
sqlx migrate add create_users
```

[Learn more about migrations →](MIGRATIONS.md)

## Testing Your API

Built-in testing utilities make API testing a breeze:

```rust
use rapid_rs::testing::TestClient;

#[tokio::test]
async fn test_create_user() {
    let app = setup_app();
    let client = TestClient::new(app);
    
    let response = client.post("/users", &json!({
        "email": "test@example.com",
        "name": "Test User"
    })).await;
    
    response.assert_status(StatusCode::CREATED);
    let user: User = response.json();
    assert_eq!(user.email, "test@example.com");
}
```

[Learn more about testing →](TESTING.md)

## Authentication

Complete JWT authentication system included:

```rust
use rapid_rs::auth::{AuthConfig, auth_routes, AuthUser};

// Protected route
async fn protected(user: AuthUser) -> impl IntoResponse {
    format!("Hello, {}!", user.email)
}

#[tokio::main]
async fn main() {
    App::new()
        .auto_configure()
        .mount(auth_routes(AuthConfig::from_env()))
        .route("/protected", get(protected))
        .run()
        .await
        .unwrap();
}
```

[Learn more about authentication →](AUTH.md)

## Configuration

Configuration loads from multiple sources:

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://localhost/mydb"
```

Override with environment variables:
```bash
APP__SERVER__PORT=8080 cargo run
AUTH_JWT_SECRET="your-secret-key" cargo run
```

## Examples

Check out the [examples](https://github.com/ashishjsharda/rapid-rs/tree/main/examples) directory:

- ✅ **REST API** - Full CRUD with validation and database
- ✅ **Auth API** - JWT authentication with protected routes
- ✅ **GraphQL API** - Complete GraphQL server (NEW!)
- ✅ **gRPC Service** - Microservice with protocol buffers (NEW!)

## Roadmap

### Phase 1 ✅ Complete
- [x] Core framework with auto-configuration
- [x] Request validation
- [x] OpenAPI generation
- [x] CLI tool
- [x] Hot reload

### Phase 2 ✅ Complete (v0.3.0)
- [x] Authentication & Authorization
- [x] Database migrations management
- [x] Testing utilities
- [x] GraphQL template
- [x] gRPC template

### Phase 3 (Future)
- [ ] Background jobs
- [ ] WebSocket support
- [ ] Multi-tenancy
- [ ] Feature flags
- [ ] Admin panel generation
- [ ] Metrics and observability
- [ ] Rate limiting
- [ ] Caching layer

## Contributing

Contributions welcome! This project has completed Phase 2 with lots of opportunities for Phase 3 features.

### Development Setup
```bash
git clone https://github.com/ashishjsharda/rapid-rs
cd rapid-rs
cargo build
cargo test

# Run examples
cd examples/rest-api
cargo run
```

[Contributing Guide →](CONTRIBUTING.md)

## Philosophy

**rapid-rs** is built on these principles:

1. **Convention over Configuration** - Sensible defaults, minimal boilerplate
2. **Type Safety First** - Leverage Rust's type system
3. **Developer Experience** - Make the common case easy
4. **Production Ready** - Observability and best practices by default
5. **Composable** - Built on Axum, use Axum patterns when needed

## Why Not Just Use Axum?

**Axum** is fantastic - rapid-rs is built on it! But Axum is intentionally minimal. You still wire up:

- Configuration loading
- Database connections
- Migrations
- Validation
- Error handling
- OpenAPI generation
- Logging
- CORS
- Authentication
- Testing utilities

**rapid-rs** gives you all of this out of the box.

## Documentation

- 📖 [API Documentation](https://docs.rs/rapid-rs)
- 🔐 [Authentication Guide](AUTH.md)
- 🗄️ [Migrations Guide](MIGRATIONS.md)
- 🧪 [Testing Guide](TESTING.md)
- 🤝 [Contributing Guide](CONTRIBUTING.md)
- 📝 [Changelog](CHANGELOG.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Credits

Built by [Ashish Sharda](https://github.com/ashishjsharda)

Standing on the shoulders of giants:
- [Axum](https://github.com/tokio-rs/axum) - The excellent web framework
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [FastAPI](https://fastapi.tiangolo.com/) - Inspiration for developer experience
- [Spring Boot](https://spring.io/projects/spring-boot) - Inspiration for conventions

---

**Star ⭐ this repo if you find it useful!**

[Report Bug](https://github.com/ashishjsharda/rapid-rs/issues) · [Request Feature](https://github.com/ashishjsharda/rapid-rs/issues) · [Documentation](https://docs.rs/rapid-rs)
