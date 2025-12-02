# 🚀 rapid-rs

> **Zero-config, batteries-included web framework for Rust**  
> FastAPI meets Spring Boot, powered by Axum

[![Crates.io](https://img.shields.io/crates/v/rapid-rs.svg)](https://crates.io/crates/rapid-rs)
[![Documentation](https://docs.rs/rapid-rs/badge.svg)](https://docs.rs/rapid-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

<!-- ⭐ NEW SECTION: Highlight v0.2.0 -->
## 🆕 What's New in v0.2.0

**Complete JWT Authentication System!** 🔐
```rust
use rapid_rs::prelude::*;
use rapid_rs::auth::{AuthConfig, auth_routes, AuthUser};

// Protected route - authentication required
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

**Features:**
- ✅ JWT tokens (access + refresh)
- ✅ Argon2id password hashing
- ✅ Role-based access control
- ✅ Dead simple `AuthUser` extractor
- ✅ Full documentation: [AUTH.md](AUTH.md)

---
<!-- End of new section -->

## Why rapid-rs?

Building web APIs in Rust shouldn't require wiring together 10+ crates and writing hundreds of lines of boilerplate. **rapid-rs** gives you the productivity of FastAPI and Spring Boot, with Rust's performance and type safety.

### ⚡ Features

- 🎯 **Zero Config** - Database, migrations, CORS, logging work out of the box
- 🔐 **Built-in Authentication** - JWT auth, password hashing, RBAC (NEW in v0.2.0!) <!-- ⭐ UPDATED -->
- 🔒 **Type-Safe** - Compile-time guarantees for routes, validation, and serialization
- 📚 **Auto-Generated Docs** - Swagger UI and OpenAPI specs from your code
- ✅ **Built-in Validation** - Request validation with helpful error messages
- 🔥 **Hot Reload** - Fast development cycle with `rapid dev`
- 🎨 **Opinionated Structure** - Convention over configuration
- 🚀 **Production Ready** - Structured logging, error handling, health checks

## Quick Start

### Installation
```bash
cargo install rapid-rs-cli
```

**Note:** By default, rapid-rs includes Swagger UI for API documentation. If you encounter installation issues, you can install without it:
```bash
cargo add rapid-rs --no-default-features
```

To enable Swagger UI later:
```bash
cargo add rapid-rs --features swagger-ui
```

### Create Your First API
```bash
# Create a new project
rapid new myapi

# Run it
cd myapi
cargo run
```

Your API is now running at:
- 🌐 **http://localhost:8080** - API endpoints
- 📚 **http://localhost:8080/docs** - Swagger UI
- 💚 **http://localhost:8080/health** - Health check

### Your First Endpoint
```rust
use rapid_rs::prelude::*;

#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
    email: String,
}

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 2))]
    name: String,
    
    #[validate(email)]
    email: String,
}

async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUser>
) -> ApiResult<User> {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
    };
    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    App::new()
        .auto_configure()
        .route("/users", post(create_user))
        .run()
        .await
        .unwrap();
}
```

That's it! You get:
- ✅ Automatic request validation
- ✅ Type-safe JSON serialization
- ✅ Structured error responses
- ✅ OpenAPI documentation
- ✅ Request tracing and logging

## Comparison

| Feature | FastAPI | Spring Boot | **rapid-rs** |
|---------|---------|-------------|--------------|
| Type Safety | ❌ Runtime | ⚠️ Runtime | ✅ Compile-time |
| Auto OpenAPI | ✅ | ✅ | ✅ |
| Hot Reload | ✅ | ✅ | ✅ |
| Zero Config | ✅ | ✅ | ✅ |
| Performance | ⚠️ Good | ⚠️ Good | ✅ Blazing Fast |
| Memory Safety | ❌ | ❌ | ✅ Guaranteed |
| Async by Default | ⚠️ Partial | ❌ | ✅ |
| Learning Curve | Easy | Medium | Easy |

## What's Included?

### 🎁 Out of the Box

- **Configuration Management** - TOML files + environment variables
- **Authentication & Authorization** - JWT-based auth with role-based access control (NEW!) <!-- ⭐ UPDATED -->
- **Database Integration** - PostgreSQL with connection pooling (SQLx)
- **Request Validation** - Derive-based validation with helpful errors
- **Error Handling** - Centralized error handling with proper HTTP status codes
- **CORS** - Sensible defaults, fully configurable
- **Logging & Tracing** - Structured logging with request correlation
- **Health Checks** - `/health` endpoint for orchestration
- **OpenAPI/Swagger** - Auto-generated docs at `/docs` (with `swagger-ui` feature, enabled by default)

### 📚 Swagger UI Configuration

**Enabled by default** - Swagger UI is included with the default features:
```toml
[dependencies]
rapid-rs = "0.2"  # Includes Swagger UI and Auth <!-- ⭐ UPDATED version -->
```

**Disable if needed** (smaller binary, faster compile):
```toml
[dependencies]
rapid-rs = { version = "0.2", default-features = false } <!-- ⭐ UPDATED version -->
```

**Re-enable later**:
```toml
[dependencies]
rapid-rs = { version = "0.2", features = ["swagger-ui", "auth"] } <!-- ⭐ UPDATED version and features -->
```

### 📦 CLI Tool
```bash
# Create new project with template
rapid new myapi --template rest-api

# Run with hot reload
rapid dev

# Coming soon:
# rapid generate resource User
# rapid db migrate
```

## Configuration

Configuration is loaded from multiple sources (in order of priority):

1. `config/default.toml` - Base configuration
2. `config/local.toml` - Local overrides (gitignored)
3. Environment variables - Prefixed with `APP__`
```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 8080 

[database]
url = "postgres://localhost/mydb"
max_connections = 10
```

Override with environment variables:
```bash
APP__SERVER__PORT=8080 cargo run
```

<!-- ⭐ NEW SECTION: Auth configuration -->
### Authentication Configuration

Set your JWT secret via environment variable (required in production):
```bash
export AUTH_JWT_SECRET="your-super-secret-key-at-least-32-characters-long"
```

See [AUTH.md](https://github.com/ashishjsharda/rapid-rs/blob/main/AUTH.md) for complete authentication documentation.
<!-- End of new section -->

## Examples

Check out the [examples](https://github.com/ashishjsharda/rapid-rs/tree/main/examples) directory for:

- ✅ **REST API** - Full CRUD with validation
- ✅ **Auth API** - JWT authentication with protected routes (NEW!) <!-- ⭐ UPDATED -->
- 🔜 **GraphQL API** - Coming soon
- 🔜 **gRPC Service** - Coming soon
- 🔜 **WebSocket Chat** - Coming soon

## Roadmap

### Phase 1 ✅ Complete
- [x] Core framework with auto-configuration
- [x] Request validation
- [x] OpenAPI generation
- [x] CLI tool for project scaffolding
- [x] Hot reload support

### Phase 2 ✅ Complete (v0.2.0) <!-- ⭐ UPDATED -->
- [x] Authentication & Authorization (JWT, sessions) <!-- ⭐ UPDATED -->
- [ ] Database migrations management
- [ ] Testing utilities
- [ ] More templates (GraphQL, gRPC)

### Phase 3 (Future)
- [ ] Background jobs
- [ ] Multi-tenancy support
- [ ] Feature flags
- [ ] Admin panel generation

## Contributing

Contributions are welcome! This is an early-stage project with lots of opportunities to make an impact.

### Development Setup
```bash
git clone https://github.com/ashishjsharda/rapid-rs
cd rapid-rs
cargo build
cargo test

# Run the REST API example
cd examples/rest-api
cargo run

# Run the Auth API example <!-- ⭐ NEW -->
cd examples/auth-api
cargo run
```

## Philosophy

**rapid-rs** is built on these principles:

1. **Convention over Configuration** - Sensible defaults, minimal boilerplate
2. **Type Safety First** - Leverage Rust's type system to catch errors at compile time
3. **Developer Experience** - Make the common case easy, the hard case possible
4. **Production Ready** - Include observability, error handling, and best practices by default
5. **Composable** - Built on Axum, use Axum patterns when you need them

## Why Not Just Use Axum?

**Axum** is fantastic - it's the foundation of rapid-rs! But Axum is intentionally minimal and unopinionated. You need to wire up:

- Configuration loading
- Database connections
- Validation
- Error handling patterns
- OpenAPI generation
- Logging setup
- CORS
- Project structure
- Authentication (NEW: now included!) <!-- ⭐ UPDATED -->

**rapid-rs** gives you all of this out of the box, while still giving you access to the full power of Axum when you need it.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/ashishjsharda/rapid-rs/blob/main/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/ashishjsharda/rapid-rs/blob/main/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Credits

Built by [Ashish Sharda](https://github.com/ashishjsharda)

Standing on the shoulders of giants:
- [Axum](https://github.com/tokio-rs/axum) - The excellent web framework this is built on
- [FastAPI](https://fastapi.tiangolo.com/) - Inspiration for developer experience
- [Spring Boot](https://spring.io/projects/spring-boot) - Inspiration for conventions

---

**Star ⭐ this repo if you find it useful!**

[Report Bug](https://github.com/ashishjsharda/rapid-rs/issues) · [Request Feature](https://github.com/ashishjsharda/rapid-rs/issues) · [Documentation](https://docs.rs/rapid-rs)
