# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-03-22

### 🎉 Phase 4 Complete - Full-Stack Features

This release adds 5 major features completing the Phase 4 roadmap.

### ✨ Added

#### GraphQL Support (`graphql` feature)
- **async-graphql integration** - Full GraphQL support via async-graphql v7
- **graphql_routes()** - Axum router with POST `/graphql` endpoint
- **GraphiQL Playground** - Interactive browser IDE at `/graphql/playground`
- **SchemaBuilder** - Convenience wrapper for async-graphql schema building
- **GraphQLConfig** - Configure endpoint paths, max depth, and complexity limits
- **Re-exports** - `Object`, `SimpleObject`, `InputObject`, `Enum`, `Union`, `Schema`, `Context`

#### Email/SMS Notifications (`notifications` feature)
- **NotificationService** - Unified interface for all notification channels
- **SmtpEmailProvider** - SMTP email via lettre with TLS support
- **EmailConfig** - Builder for SMTP host, port, credentials, from address
- **EmailMessage** - Email with plain text + optional HTML body, CC support
- **Multipart emails** - Automatic `text/plain` + `text/html` multipart construction

#### SMS Notifications (`notifications-sms` feature)
- **TwilioSmsProvider** - SMS via Twilio REST API
- **SmsConfig** - Twilio Account SID, Auth Token, and from number
- **SmsMessage** - SMS content with optional from number override

#### File Uploads (`file-uploads` feature)
- **FileUploadService** - Multipart file upload processing
- **UploadConfig** - Max file size, allowed MIME types, upload directory, max files per request
- **LocalStorage** - Save uploaded files to local filesystem with UUID-based naming
- **StorageBackend** - Enum-based dispatch for multiple storage backends
- **upload_routes()** - Axum router with POST `/upload` multipart handler
- **UploadedFile** - Metadata struct (id, original name, stored name, content type, size, URL)

#### Admin Dashboard (`admin` feature)
- **admin_routes()** - Mounts admin UI and API at configurable base path (default: `/admin`)
- **AdminConfig** - Configure secret key, base path, app name, version
- **Embedded HTML Dashboard** - Dark-themed SPA with real-time stats (no external deps)
- **GET /admin** - Full admin dashboard web UI
- **GET /admin/stats** - JSON stats (uptime, request count, error rate, system info, enabled features)
- **GET /admin/health** - Detailed health check endpoint
- **Atomic counters** - `increment_request_count()`, `increment_error_count()` helpers

#### More Database Backends
- **SQLite** (`db-sqlite` feature) - `sqlite::connect()`, `sqlite::connect_in_memory()`, `sqlite::connect_file()`
- **MySQL** (`db-mysql` feature) - `mysql::connect()`, `mysql::connect_with_pool_size()`

### 🔧 Changed
- Bumped version from `0.4.0` to `0.5.0`
- Added `async-graphql`, `async-graphql-axum`, `lettre`, `reqwest` to workspace dependencies
- Added `axum/multipart` feature activation via `file-uploads` feature flag
- Added `sqlx/sqlite` and `sqlx/mysql` activation via `db-sqlite` and `db-mysql` feature flags
- Updated `full` feature to include all Phase 4 features
- Updated architecture diagram and feature flags section in README

### 📝 Documentation
- New `examples/graphql-api.rs` demonstrating GraphQL schema and routes
- Updated README with Phase 4 feature showcases and code examples
- Updated roadmap: Phase 4 marked complete ✅

## [0.4.1] - 2026-01-19

### Fixed
- Fixed README documentation links to point to GitHub URLs
- Fixed LICENSE link (now points to LICENSE-MIT)
- Fixed CONTRIBUTING.md link
- Fixed CHANGELOG.md link
- Fixed examples directory link

## [0.4.0] - 2026-01-18

### 🎉 Phase 3 Complete - Enterprise Features

This is a major release adding 7 production-ready enterprise features!

### ✨ Added

#### Background Jobs (`jobs` feature)
- **JobQueue** - Async job queue with configurable storage backends
- **Job Priorities** - High, Normal, Low priority levels
- **Job Scheduling** - Schedule jobs for future execution with cron-like scheduling
- **Job Storage** - In-memory storage with optional database backend support
- **Job Worker** - Configurable worker pool with automatic job processing
- **Job Stats** - Queue metrics and monitoring (pending, running, completed, failed)
- **Job Lifecycle** - Complete job state management (pending → running → completed/failed)

#### WebSocket Support (`websocket` feature)
- **WebSocketServer** - Full-duplex real-time communication
- **Room Management** - Group chat and broadcasting capabilities
- **Connection Tracking** - Track and manage active WebSocket connections
- **Message Types** - Text, JSON, Binary, System, and Error message support
- **Handler Trait** - Customizable WebSocket event handlers (connect, message, disconnect)
- **Connection Info** - Metadata tracking for each WebSocket connection

#### Caching Layer (`cache` feature)
- **Multi-Backend Support** - Memory (Moka) and Redis backends
- **Cache Interface** - Unified API for all cache backends
- **TTL Support** - Time-to-live for cached entries
- **Cache Stats** - Hit rate, miss rate, and entry count metrics
- **get_or_compute** - Automatic cache population pattern
- **Configurable** - TTL, max entries, and eviction policies
- **Memory Cache** - Fast in-memory caching with Moka
- **Redis Cache** (`cache-redis` feature) - Distributed caching with Redis

#### Rate Limiting (`rate-limit` feature)
- **RateLimiter** - Token bucket algorithm via Governor
- **Configurable Limits** - Requests per period and burst size
- **Middleware Integration** - Easy route protection
- **Flexible Configuration** - Per-minute, per-hour, per-day helpers

#### Metrics (`observability` feature)
- **Prometheus Integration** - Industry-standard metrics format
- **MetricsExporter** - HTTP `/metrics` endpoint
- **Request Metrics** - Automatic HTTP request tracking (count, duration, errors)
- **Custom Metrics** - Counter, Gauge, and Histogram support
- **Metrics Middleware** - Auto-record request metrics
- **Configurable Buckets** - Custom histogram buckets for latency tracking

#### Feature Flags (`feature-flags` feature)
- **FeatureFlags** - Runtime feature toggles
- **Rollout Percentage** - Gradual feature rollouts (A/B testing)
- **User Targeting** - Enable features for specific users
- **Flag Provider Trait** - Pluggable flag storage backends
- **Hash-based Rollout** - Consistent user assignment to feature variants

#### Multi-Tenancy (`multi-tenancy` feature)
- **TenantContext** - Request-scoped tenant information
- **TenantResolver** - Subdomain and header-based tenant resolution
- **Tenant Configuration** - Per-tenant settings and metadata
- **Tenant Plans** - Starter, Professional, Enterprise plan support
- **Tenant Limits** - Per-tenant quotas (users, requests, storage)
- **Tenant Middleware** - Automatic tenant extraction and context injection
- **TenantExtractor** - Easy tenant access in handlers

### 🔧 Changed
- Updated Axum dependency to include WebSocket support (`ws` feature)
- Added `async-trait` as optional dependency for job handlers
- Added `futures` dependency for WebSocket stream handling
- Metrics crate upgraded to v0.22 for latest features
- Added `dashmap`, `moka`, `redis`, `governor`, `prometheus` optional dependencies

### 📝 Documentation
- Added comprehensive examples for all Phase 3 features
- Updated README with feature showcases and code examples
- Added inline documentation for all public APIs
- Created migration guides for new features

### 🐛 Fixed
- Fixed cache trait object compatibility issues (switched to enum-based dispatch)
- Fixed metrics macro lifetime issues (using `'static` lifetimes)
- Fixed WebSocket handler type signatures (Message type instead of String)
- Fixed Redis type annotations for future compatibility
- Fixed multi-tenancy resolver trait bounds

### 🧪 Testing
- 97% test coverage (36+ passing tests)
- Unit tests for all major features
- Integration test examples
- Doc tests for public APIs

## [0.3.2] - 2025-12-14

### Fixed
- Fixed remaining documentation links in README (Examples section)

## [0.3.1] - 2025-12-14

### Fixed
- Fixed README documentation links on crates.io

## [0.3.0] - 2025-12-14

### Added
- 🗄️ **Database Migrations Management**
  - `MigrationConfig` for configuring migration behavior
  - `connect_and_migrate()` for automatic database setup and migration
  - `run_migrations()` for manual migration control
  - `ensure_database_exists()` to create databases automatically
  - Integration with sqlx's migration system
  - Support for custom migration paths
  - Automatic migration running on startup (configurable)
- 🧪 **Testing Utilities**
  - `TestClient` for easy API endpoint testing
  - `TestResponse` with assertion helpers
  - Support for authenticated requests in tests
  - Database testing utilities (with `db-tests` feature)
  - Integration test examples and best practices
- 📚 **Additional Project Templates**
  - GraphQL template with async-graphql integration
  - gRPC template with tonic integration
  - Updated CLI to support `rapid new myapi --template graphql|grpc`
- 📖 **Comprehensive Documentation**
  - `MIGRATIONS.md` - Complete migration guide with best practices
  - `TESTING.md` - Testing guide with examples and patterns
  - Updated README with Phase 2 completion status

### Changed
- Updated CLI to support GraphQL and gRPC templates
- Enhanced project scaffolding with migration support
- Improved documentation structure

### Fixed
- Database connection handling in test environments

## [0.2.0] - 2025-11-27

### Added
- 🔐 **Authentication & Authorization (Phase 2)**
  - JWT-based authentication with access and refresh tokens
  - Password hashing with Argon2id (industry-standard security)
  - `AuthUser` extractor for protected routes
  - `OptionalAuthUser` for optional authentication
  - Role-based access control with `require_role()`, `require_any_role()`, `require_all_roles()`
  - `UserStore` trait for custom database backends
  - `InMemoryUserStore` for development/testing
  - Built-in auth routes: `/auth/login`, `/auth/register`, `/auth/refresh`, `/auth/logout`, `/auth/me`
  - Password strength validation
  - Configurable token expiry times
  - Environment variable configuration (`AUTH_JWT_SECRET`, etc.)
- New `auth-api` example demonstrating authentication
- `AUTH.md` comprehensive documentation for authentication features

### Changed
- Auth feature is enabled by default (use `default-features = false` to disable)
- Updated prelude to include `AuthUser` and `AuthConfig` when auth feature is enabled

### Fixed
- `AuthUser` extractor now automatically falls back to environment config when not explicitly provided
- Improved out-of-the-box experience - no middleware configuration required for basic auth

## [0.1.4] - 2025-11-19

### Fixed
- Fixed examples link in README to point to GitHub repository

## [0.1.3] - 2025-11-19

### Fixed
- Added README.md to package manifest so it displays on crates.io

## [0.1.2] - 2025-11-18

### Changed
- **Default port changed from 3000 to 8080** to avoid Windows permission issues
- Updated all documentation to reflect port 8080
- Updated CLI templates to use port 8080

### Fixed
- Resolved Windows permission denied errors on port 3000
- Improved cross-platform compatibility

## [0.1.1] - 2025-11-18

### Changed
- **BREAKING**: Made Swagger UI optional via feature flag (enabled by default)
- Downgraded `utoipa-swagger-ui` from v7.0 to v6.0 for better stability
- Updated documentation with Swagger UI configuration instructions

### Fixed
- Resolved installation issues caused by `utoipa-swagger-ui` v7.0 download failures
- Improved error messages when Swagger UI feature is disabled

### Added
- `swagger-ui` feature flag (enabled by default)
- Instructions in README for disabling/enabling Swagger UI
- Helpful log message when Swagger UI is disabled

## [0.1.0] - 2025-11-18

### Added
- Initial release! 🎉
- Zero-config application setup with `App::new().auto_configure()`
- Request validation with `ValidatedJson<T>` extractor
- Unified error handling with `ApiError` and `ApiResult`
- Auto-generated OpenAPI documentation with Swagger UI
- Type-safe configuration from TOML files and environment variables
- Structured logging with tracing and request correlation
- CORS support with sensible defaults
- Health check endpoint at `/health`
- CLI tool for project scaffolding (`rapid new`)
- Hot reload support (`rapid dev`)
- REST API example with full CRUD operations

### Framework Features
- Built on Axum 0.7 for excellent performance
- Async by default with Tokio
- Compile-time type safety
- Convention over configuration
- Production-ready observability

---

## Migration Guides

### Upgrading to 0.4.0 from 0.3.x

**No breaking changes!** All Phase 2 and Phase 3 features are fully compatible.

**New Features:**
- Enable new features via Cargo.toml feature flags
- All features are opt-in and don't affect existing code

**Example:**
```toml
# Old (still works)
rapid-rs = { version = "0.3", features = ["auth"] }

# New (add features as needed)
rapid-rs = { version = "0.4", features = ["auth", "jobs", "cache", "websocket"] }

# Or enable everything
rapid-rs = { version = "0.4", features = ["full"] }
```

**Feature Flags:**
- `jobs` - Background job processing
- `websocket` - WebSocket support
- `cache` - In-memory caching
- `cache-redis` - Redis caching
- `rate-limit` - Rate limiting
- `observability` - Prometheus metrics
- `feature-flags` - Feature flags
- `multi-tenancy` - Multi-tenant support
- `full` - Enable all features

---

## Links

- **Crates.io:** https://crates.io/crates/rapid-rs
- **Documentation:** https://docs.rs/rapid-rs
- **Repository:** https://github.com/rapid-rs/rapid-rs
- **Issues:** https://github.com/rapid-rs/rapid-rs/issues

---

**Thank you to all contributors! 🎉**