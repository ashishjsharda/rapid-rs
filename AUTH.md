# 🔐 Authentication & Authorization

rapid-rs includes a complete JWT-based authentication system out of the box.

## Quick Start

```rust
use rapid_rs::prelude::*;
use rapid_rs::auth::{AuthConfig, auth_routes};

#[tokio::main]
async fn main() {
    let auth_config = AuthConfig::from_env();
    
    App::new()
        .auto_configure()
        .mount(auth_routes(auth_config))
        .run()
        .await
        .unwrap();
}
```

This gives you these endpoints automatically:
- `POST /auth/register` - Register a new user
- `POST /auth/login` - Login and get JWT tokens
- `POST /auth/refresh` - Refresh access token
- `POST /auth/logout` - Logout
- `GET /auth/me` - Get current user info (protected)

## Configuration

### Environment Variables

```bash
# Required in production - use a strong random string!
AUTH_JWT_SECRET=your-super-secret-key-min-32-chars

# Optional
AUTH_ACCESS_TOKEN_EXPIRY_SECS=900      # 15 minutes (default)
AUTH_REFRESH_TOKEN_EXPIRY_SECS=604800  # 7 days (default)
AUTH_ISSUER=your-app-name
AUTH_AUDIENCE=your-api
```

### Programmatic Configuration

```rust
use rapid_rs::auth::AuthConfig;
use std::time::Duration;

let config = AuthConfig::new("your-secret-key")
    .access_token_expiry(Duration::from_secs(30 * 60))  // 30 minutes
    .refresh_token_expiry(Duration::from_secs(14 * 24 * 60 * 60))  // 14 days
    .issuer("my-app")
    .audience("my-api");
```

## Protected Routes

Use the `AuthUser` extractor to protect routes:

```rust
use rapid_rs::prelude::*;
use rapid_rs::auth::AuthUser;

// This route requires authentication
async fn protected_route(user: AuthUser) -> impl IntoResponse {
    format!("Hello, {}!", user.email)
}

// This route requires the "admin" role
async fn admin_only(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    user.require_role("admin")?;
    Ok("Welcome, admin!")
}

// Check multiple roles
async fn editor_or_admin(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    user.require_any_role(&["editor", "admin"])?;
    Ok("You have edit access!")
}
```

## Optional Authentication

Use `OptionalAuthUser` for routes that work with or without authentication:

```rust
use rapid_rs::auth::OptionalAuthUser;

async fn maybe_personalized(user: OptionalAuthUser) -> impl IntoResponse {
    match user.0 {
        Some(user) => format!("Hello, {}!", user.email),
        None => "Hello, anonymous!".to_string(),
    }
}
```

## Custom User Store

By default, rapid-rs uses an in-memory store (for development only). 

For production, implement the `UserStore` trait for your database:

```rust
use rapid_rs::auth::{UserStore, StoredUser, CreateUserData};
use sqlx::PgPool;

struct PostgresUserStore {
    pool: PgPool,
}

#[async_trait]
impl UserStore for PostgresUserStore {
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
        let row = sqlx::query!(
            "SELECT id, email, name, password_hash, roles FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| StoredUser {
            id: r.id.to_string(),
            email: r.email,
            name: r.name,
            password_hash: r.password_hash,
            roles: r.roles,
        }))
    }
    
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("Invalid ID".into()))?;
        
        let row = sqlx::query!(
            "SELECT id, email, name, password_hash, roles FROM users WHERE id = $1",
            uuid
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| StoredUser {
            id: r.id.to_string(),
            email: r.email,
            name: r.name,
            password_hash: r.password_hash,
            roles: r.roles,
        }))
    }
    
    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError> {
        let id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, name, password_hash, roles)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            user.email,
            user.name,
            user.password_hash,
            &["user".to_string()][..],
        )
        .execute(&self.pool)
        .await?;
        
        Ok(StoredUser {
            id: id.to_string(),
            email: user.email,
            name: user.name,
            password_hash: user.password_hash,
            roles: vec!["user".to_string()],
        })
    }
    
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("Invalid ID".into()))?;
        
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            password_hash,
            uuid
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn email_exists(&self, email: &str) -> Result<bool, ApiError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            email
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(exists.unwrap_or(false))
    }
}
```

Then use it:

```rust
use rapid_rs::auth::auth_routes_with_store;

let pool = PgPool::connect(&database_url).await?;
let user_store = PostgresUserStore { pool };

App::new()
    .auto_configure()
    .mount(auth_routes_with_store(auth_config, user_store))
    .run()
    .await?;
```

## Password Hashing

rapid-rs uses Argon2id for password hashing (the recommended algorithm):

```rust
use rapid_rs::auth::{hash_password, verify_password, AuthConfig};

let config = AuthConfig::default();

// Hash a password
let hash = hash_password("my-password", &config)?;

// Verify a password
let is_valid = verify_password("my-password", &hash)?;
```

### Password Validation

```rust
use rapid_rs::auth::password::{validate_password_strength, PasswordValidator};

// Default validation (8+ chars, upper, lower, digit)
validate_password_strength("SecurePass1")?;

// Custom validation
let validator = PasswordValidator::new()
    .min_length(12)
    .require_special(true);

validator.validate("SecurePass1!")?;
```

## API Reference

### Login

```bash
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123"
}
```

Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "name": "John Doe",
    "roles": ["user"]
  }
}
```

### Register

```bash
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123",
  "name": "John Doe"
}
```

### Refresh Token

```bash
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}
```

### Access Protected Routes

```bash
GET /protected
Authorization: Bearer <access_token>
```

## Security Best Practices

1. **Use strong JWT secrets** - At least 32 characters, random
2. **Store secrets securely** - Use environment variables, not code
3. **Use HTTPS** - Always in production
4. **Short access tokens** - 15 minutes or less
5. **Implement token refresh** - Don't make users re-login frequently
6. **Implement rate limiting** - Prevent brute force attacks
7. **Log security events** - Failed logins, token refreshes, etc.

## Disabling Auth

If you don't need authentication:

```toml
[dependencies]
rapid-rs = { version = "0.1", default-features = false, features = ["swagger-ui"] }
```
