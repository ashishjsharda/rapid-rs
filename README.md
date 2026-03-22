# 🚀 rapid-rs

**Zero-config, batteries-included web framework for Rust**  
*FastAPI meets Spring Boot - Build production-ready APIs in minutes, not days*

[![Crates.io](https://img.shields.io/crates/v/rapid-rs.svg)](https://crates.io/crates/rapid-rs)
[![Documentation](https://docs.rs/rapid-rs/badge.svg)](https://docs.rs/rapid-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/ashishjsharda/rapid-rs/blob/main/LICENSE-MIT)

---

## ✨ What's New in v0.5.0

**🎉 Phase 4 Complete - Full-Stack Features!**

- **🔷 GraphQL** - First-class GraphQL support with async-graphql + interactive playground
- **📧 Email/SMS** - SMTP email and Twilio SMS notifications out of the box
- **📁 File Uploads** - Multipart uploads with local storage backend
- **🖥️ Admin Dashboard** - Embedded web UI with real-time stats and health monitoring
- **🗄️ More Databases** - SQLite and MySQL backends in addition to PostgreSQL

[See full changelog](https://github.com/ashishjsharda/rapid-rs/blob/main/CHANGELOG.md)

---

## 🎯 Why rapid-rs?

Stop wasting time on boilerplate. Get a production-ready API with authentication, database, validation, and more - **all configured automatically**.

```rust
use rapid_rs::rapid;

#[rapid]
async fn main() {
    // That's it! You now have:
    // ✅ REST API with OpenAPI docs
    // ✅ Database migrations
    // ✅ JWT authentication
    // ✅ Request validation
    // ✅ Error handling
    // ✅ Logging & tracing
    // And much more...
}
```

**Visit:** `http://localhost:8080/swagger-ui` for interactive API docs!

---

## 🚀 Quick Start

### Installation

```toml
[dependencies]
rapid-rs = "0.4"
tokio = { version = "1", features = ["full"] }
```

### Your First API

```rust
use rapid_rs::{rapid, web, rapid_web::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
}

#[web::get("/users/{id}")]
async fn get_user(id: web::Path<i32>) -> Json<User> {
    // Auto-validated, auto-serialized, auto-documented!
    Json(User {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

#[rapid]
async fn main() {
    // Your API is live at http://localhost:8080 🎉
}
```

---

## 🎨 Features

### Core Features (Always Available)

- ✅ **REST API** - Built on Axum for blazing-fast performance
- ✅ **OpenAPI/Swagger** - Auto-generated interactive documentation
- ✅ **Database** - SQLx integration with migrations
- ✅ **Validation** - Request/response validation with `validator`
- ✅ **Error Handling** - Consistent, user-friendly error responses
- ✅ **Logging** - Structured logging with `tracing`
- ✅ **Configuration** - Environment-based config management

### Authentication (`auth` feature)

```rust
use rapid_rs::auth::{AuthUser, hash_password, create_token};

#[web::post("/login")]
async fn login(credentials: Json<LoginRequest>) -> Json<LoginResponse> {
    // Built-in JWT + password hashing
    let token = create_token(&user)?;
    Json(LoginResponse { token })
}

#[web::get("/profile")]
async fn profile(user: AuthUser) -> Json<User> {
    // Automatic auth validation!
    Json(user.into())
}
```

### Background Jobs (`jobs` feature) 🆕

```rust
use rapid_rs::jobs::{JobQueue, JobPriority};

let queue = JobQueue::new(storage, config);

// Submit job
queue.enqueue(
    SendEmailJob { to: "user@example.com" },
    "send_email"
).await?;

// Schedule for later
queue.schedule(
    job,
    "job_type",
    chrono::Utc::now() + Duration::hours(1)
).await?;
```

### WebSocket (`websocket` feature) 🆕

```rust
use rapid_rs::websocket::{WebSocketServer, WebSocketHandler};

let ws_server = WebSocketServer::new();
ws_server.set_handler(MyHandler).await;

app.merge(ws_server.routes());
// WebSocket ready at ws://localhost:8080/ws
```

### Caching (`cache` feature) 🆕

```rust
use rapid_rs::cache::{Cache, CacheConfig};

let cache = Cache::new(CacheConfig::default());

// Cache with TTL
cache.set("user:123", &user, Duration::from_secs(300)).await?;

// Get or compute
let user = cache.get_or_compute(
    "user:123",
    Duration::from_secs(300),
    || fetch_user_from_db(123)
).await?;
```

### Rate Limiting (`rate-limit` feature) 🆕

```rust
use rapid_rs::rate_limit::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    requests_per_period: 100,
    period: Duration::from_secs(60),
    burst_size: 10,
});

// Apply to routes
app.layer(axum::middleware::from_fn_with_state(
    limiter,
    rate_limit_middleware
));
```

### Metrics (`observability` feature) 🆕

```rust
use rapid_rs::metrics::MetricsExporter;

let metrics = MetricsExporter::new();

// Prometheus metrics at /metrics
app.merge(metrics.routes());
```

### Feature Flags (`feature-flags` feature) 🆕

```rust
use rapid_rs::feature_flags::{FeatureFlags, FeatureConfig};

let mut flags = FeatureFlags::new();

flags.add_flag("dark_mode", FeatureConfig {
    enabled: true,
    rollout_percentage: 50,
    allowed_users: vec!["beta_testers".to_string()],
});

if flags.is_enabled("dark_mode", Some(&user_id)) {
    // Show dark mode UI
}
```

### Multi-Tenancy (`multi-tenancy` feature) 🆕

```rust
use rapid_rs::multi_tenancy::{TenantContext, TenantExtractor};

#[web::get("/data")]
async fn get_data(tenant: TenantExtractor) -> Json<Data> {
    let tenant_id = tenant.0.tenant_id();
    // Data automatically scoped to tenant!
    fetch_tenant_data(tenant_id).await
}
```

### GraphQL (`graphql` feature) 🆕

```rust
use rapid_rs::graphql::{graphql_routes, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello from rapid-rs GraphQL!"
    }
}

#[derive(SimpleObject)]
struct User { id: i32, name: String }

let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
// Playground at http://localhost:3000/graphql/playground
app.merge(graphql_routes(schema));
```

### Email/SMS Notifications (`notifications` feature) 🆕

```rust
use rapid_rs::notifications::{NotificationService, EmailConfig, EmailMessage};

let service = NotificationService::new()
    .with_email(
        EmailConfig::new()
            .with_smtp_host("smtp.gmail.com")
            .with_port(587)
            .with_tls()
            .with_credentials("user@gmail.com", "app-password")
            .with_from("My App <noreply@myapp.com>")
    );

service.send_email(
    EmailMessage::new("user@example.com", "Welcome!", "Thanks for signing up!")
        .with_html("<h1>Thanks for signing up!</h1>")
).await?;
```

### File Uploads (`file-uploads` feature) 🆕

```rust
use rapid_rs::uploads::{FileUploadService, UploadConfig, upload_routes};
use std::sync::Arc;

let service = Arc::new(FileUploadService::new(
    UploadConfig::new()
        .with_max_size(10 * 1024 * 1024)       // 10 MB
        .with_allowed_types(vec!["image/jpeg", "image/png"])
        .with_upload_dir("./uploads")
));

// POST /upload - accepts multipart/form-data
app.merge(upload_routes(service));
```

### Admin Dashboard (`admin` feature) 🆕

```rust
use rapid_rs::admin::{AdminConfig, admin_routes};

// Embedded dashboard at /admin
app.merge(admin_routes(
    AdminConfig::new()
        .with_app_name("My API")
        .with_secret_key("admin-secret")
));
```

---

## 📦 Feature Flags

Choose the features you need:

```toml
[dependencies]
rapid-rs = { version = "0.5", features = ["full"] }

# Or pick specific features:
rapid-rs = { version = "0.5", features = [
    "auth",               # JWT authentication
    "jobs",               # Background jobs
    "websocket",          # WebSocket support
    "cache",              # In-memory caching
    "cache-redis",        # Redis caching
    "rate-limit",         # Rate limiting
    "observability",      # Prometheus metrics
    "feature-flags",      # Feature flags
    "multi-tenancy",      # Multi-tenant support
    "graphql",            # GraphQL API support
    "notifications",      # Email notifications
    "notifications-sms",  # SMS via Twilio
    "file-uploads",       # Multipart file uploads
    "admin",              # Admin dashboard
    "db-sqlite",          # SQLite backend
    "db-mysql",           # MySQL backend
]}
```

---

## 🏗️ Architecture

```
rapid-rs
├── Core Framework (Axum + Tower)
├── Database (SQLx + Migrations + SQLite + MySQL)
├── Auth (JWT + Argon2)
├── Validation (validator crate)
├── Jobs (Async Queue + Scheduler)
├── WebSocket (Real-time Communication)
├── Cache (Memory + Redis)
├── Rate Limiting (Token Bucket)
├── Metrics (Prometheus)
├── Feature Flags (A/B Testing)
├── Multi-Tenancy (SaaS Ready)
├── GraphQL (async-graphql + Playground)
├── Notifications (SMTP Email + Twilio SMS)
├── File Uploads (Multipart + Local Storage)
└── Admin Dashboard (Embedded UI + Stats API)
```

---

## 📚 Documentation

- **[Getting Started Guide](https://docs.rs/rapid-rs/latest/rapid_rs/#getting-started)**
- **[API Reference](https://docs.rs/rapid-rs)**
- **[Examples](https://github.com/ashishjsharda/rapid-rs/tree/main/examples)**
- **[Changelog](https://github.com/ashishjsharda/rapid-rs/blob/main/CHANGELOG.md)**

---

## 🤝 Contributing

Contributions welcome! Please read our [Contributing Guide](https://github.com/ashishjsharda/rapid-rs/blob/main/CONTRIBUTING.md) first.

---

## 📜 License

MIT License - see [LICENSE-MIT](https://github.com/ashishjsharda/rapid-rs/blob/main/LICENSE-MIT) or [LICENSE-APACHE](https://github.com/ashishjsharda/rapid-rs/blob/main/LICENSE-APACHE) file for details

---

## 🙏 Credits

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Database toolkit
- [Tower](https://github.com/tower-rs/tower) - Middleware
- [Utoipa](https://github.com/juhaku/utoipa) - OpenAPI generation
- And many more amazing Rust crates!

---

## 🎯 Roadmap

### Phase 1 ✅ (v0.1.0)
- Core REST API framework
- Database integration
- OpenAPI documentation

### Phase 2 ✅ (v0.2.0 - v0.3.0)
- JWT Authentication
- Password hashing
- Role-based access control
- Testing utilities

### Phase 3 ✅ (v0.4.0)
- Background jobs
- WebSocket support
- Caching layer
- Rate limiting
- Prometheus metrics
- Feature flags
- Multi-tenancy

### Phase 4 ✅ (v0.5.0)
- GraphQL support
- Email/SMS notifications
- File uploads
- Admin dashboard
- More database backends (SQLite + MySQL)

### Phase 5 📋 (Future)
- Serverless deployment
- CLI code generation
- Plugin system
- Real-time subscriptions

---

## ⭐ Star History

If rapid-rs helps you build faster, give us a star! ⭐

---

**Made with ❤️ for the Rust community**