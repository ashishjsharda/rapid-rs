# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-11-22

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
- `AUTH.md` documentation for authentication features

### Changed
- Auth feature is enabled by default (use `default-features = false` to disable)
- Updated prelude to include `AuthUser` and `AuthConfig` when auth feature is enabled

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


