#  Testing Guide

rapid-rs includes comprehensive testing utilities to make testing your APIs easy and enjoyable.

## Quick Start

```rust
use rapid_rs::testing::TestClient;
use rapid_rs::prelude::*;

#[tokio::test]
async fn test_create_user() {
    let app = setup_test_app();
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

## Test Client

### Making Requests

```rust
use rapid_rs::testing::TestClient;

let client = TestClient::new(app);

// GET request
let response = client.get("/users").await;

// POST request with JSON body
let body = json!({"name": "John", "email": "john@example.com"});
let response = client.post("/users", &body).await;

// PUT request
let response = client.put("/users/123", &body).await;

// PATCH request
let response = client.patch("/users/123", &json!({"name": "Jane"})).await;

// DELETE request
let response = client.delete("/users/123").await;
```

### Authenticated Requests

```rust
// Get a token (from login endpoint or create directly)
let login_response = client.post("/auth/login", &json!({
    "email": "admin@example.com",
    "password": "password"
})).await;

let token = login_response.json::<AuthResponse>().access_token;

// Use token for authenticated requests
let response = client.authorized_get("/admin/users", &token).await;

let response = client.authorized_post(
    "/admin/users",
    &token,
    &json!({"name": "New Admin"})
).await;
```

## Test Responses

### Status Assertions

```rust
response.assert_status(StatusCode::OK);
response.assert_status(StatusCode::CREATED);
response.assert_status(StatusCode::NOT_FOUND);
```

### JSON Responses

```rust
// Deserialize to a type
let user: User = response.json();
assert_eq!(user.email, "test@example.com");

// Work with raw JSON
let json: serde_json::Value = response.json();
assert_eq!(json["message"], "Success");
```

### Text Responses

```rust
let text = response.text();
response.assert_text_contains("expected string");
```

### Check Success

```rust
assert!(response.is_success());
```

## Database Testing

### Setup Test Database

```rust
use rapid_rs::database::connect_and_migrate;

#[tokio::test]
async fn test_with_database() {
    let pool = connect_and_migrate(
        "postgres://localhost/test_db",
        MigrationConfig::default()
    ).await.unwrap();
    
    // Your test here
}
```

### Test Isolation

Use transactions for test isolation:

```rust
#[tokio::test]
async fn test_user_creation() {
    let pool = test_pool().await;
    
    let mut tx = pool.begin().await.unwrap();
    
    // Create test data
    sqlx::query!("INSERT INTO users ...")
        .execute(&mut *tx)
        .await
        .unwrap();
    
    // Run your test
    let result = get_user(&mut tx, "user-id").await;
    assert!(result.is_ok());
    
    // Rollback (automatically on drop)
    tx.rollback().await.ok();
}
```

### Database Fixtures

Create reusable test fixtures:

```rust
async fn create_test_user(pool: &PgPool) -> User {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, name, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, email, name, created_at, updated_at
        "#,
        "test@example.com",
        "Test User",
        "hashed_password"
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn test_something() {
    let pool = test_pool().await;
    let user = create_test_user(&pool).await;
    
    // Test with the user
}
```

## Testing Authenticated Endpoints

### Helper for Auth Setup

```rust
async fn setup_auth_test(client: &TestClient) -> String {
    // Register a user
    client.post("/auth/register", &json!({
        "email": "test@example.com",
        "password": "TestPass123",
        "name": "Test User"
    })).await;
    
    // Login and get token
    let response = client.post("/auth/login", &json!({
        "email": "test@example.com",
        "password": "TestPass123"
    })).await;
    
    let auth: AuthResponse = response.json();
    auth.access_token
}

#[tokio::test]
async fn test_protected_route() {
    let app = setup_app();
    let client = TestClient::new(app);
    let token = setup_auth_test(&client).await;
    
    let response = client.authorized_get("/protected", &token).await;
    response.assert_status(StatusCode::OK);
}
```

## Testing Validation

```rust
#[tokio::test]
async fn test_validation_errors() {
    let client = TestClient::new(app);
    
    // Invalid email
    let response = client.post("/users", &json!({
        "email": "not-an-email",
        "name": "Test"
    })).await;
    
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    response.assert_text_contains("Invalid email");
}
```

## Integration Test Structure

### Recommended Structure

```
myapp/
├── tests/
│   ├── common/
│   │   ├── mod.rs         # Shared test utilities
│   │   └── fixtures.rs    # Test data fixtures
│   ├── auth_tests.rs      # Authentication tests
│   ├── user_tests.rs      # User endpoints tests
│   └── integration_test.rs # Full integration tests
├── src/
└── Cargo.toml
```

### Common Test Utilities

`tests/common/mod.rs`:
```rust
use rapid_rs::prelude::*;
use rapid_rs::testing::TestClient;

pub fn setup_app() -> Router {
    // Build your app for testing
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        // ... more routes
}

pub async fn test_pool() -> PgPool {
    connect_and_migrate(
        &std::env::var("TEST_DATABASE_URL")
            .unwrap_or("postgres://localhost/test_db".to_string()),
        MigrationConfig::default()
    )
    .await
    .unwrap()
}
```

### Integration Test Example

`tests/user_tests.rs`:
```rust
mod common;

use common::*;
use rapid_rs::testing::TestClient;

#[tokio::test]
async fn test_user_crud() {
    let app = setup_app();
    let client = TestClient::new(app);
    
    // Create
    let response = client.post("/users", &json!({
        "email": "test@example.com",
        "name": "Test User"
    })).await;
    
    response.assert_status(StatusCode::CREATED);
    let user: User = response.json();
    let user_id = user.id;
    
    // Read
    let response = client.get(&format!("/users/{}", user_id)).await;
    response.assert_status(StatusCode::OK);
    
    // Update
    let response = client.patch(
        &format!("/users/{}", user_id),
        &json!({"name": "Updated Name"})
    ).await;
    response.assert_status(StatusCode::OK);
    
    // Delete
    let response = client.delete(&format!("/users/{}", user_id)).await;
    response.assert_status(StatusCode::NO_CONTENT);
}
```

## Mocking External Services

### Using mockito

```toml
[dev-dependencies]
mockito = "1.0"
```

```rust
#[tokio::test]
async fn test_external_api_call() {
    let mut server = mockito::Server::new_async().await;
    
    let mock = server.mock("GET", "/api/data")
        .with_status(200)
        .with_body(r#"{"result": "success"}"#)
        .create();
    
    // Test your code that calls this endpoint
    let result = fetch_data(&server.url()).await;
    
    mock.assert();
    assert_eq!(result.unwrap(), "success");
}
```

## Performance Testing

### Basic Benchmarking

```toml
[dev-dependencies]
criterion = "0.5"
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_user_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("create_user", |b| {
        b.to_async(&rt).iter(|| async {
            let client = TestClient::new(setup_app());
            client.post("/users", &json!({
                "email": "bench@example.com",
                "name": "Bench User"
            })).await
        });
    });
}

criterion_group!(benches, benchmark_user_creation);
criterion_main!(benches);
```

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage
```

### View Coverage

Open `coverage/index.html` in your browser.

## Best Practices

### 1. Test Naming

```rust
#[tokio::test]
async fn test_create_user_with_valid_data_returns_201() { }

#[tokio::test]
async fn test_create_user_with_invalid_email_returns_422() { }

#[tokio::test]
async fn test_get_nonexistent_user_returns_404() { }
```

### 2. Arrange-Act-Assert Pattern

```rust
#[tokio::test]
async fn test_user_update() {
    // Arrange
    let client = TestClient::new(setup_app());
    let user = create_test_user().await;
    
    // Act
    let response = client.patch(
        &format!("/users/{}", user.id),
        &json!({"name": "New Name"})
    ).await;
    
    // Assert
    response.assert_status(StatusCode::OK);
    let updated: User = response.json();
    assert_eq!(updated.name, "New Name");
}
```

### 3. Test Independence

Each test should be independent and not rely on other tests:

```rust
// Good: Self-contained
#[tokio::test]
async fn test_something() {
    let pool = test_pool().await;
    let user = create_test_user(&pool).await;
    // Test with user
}

// Bad: Depends on previous test
#[tokio::test]
async fn test_1_create_user() { /* creates user */ }

#[tokio::test]
async fn test_2_get_user() { /* assumes user from test_1 exists */ }
```

### 4. Use Test Fixtures

```rust
struct TestContext {
    client: TestClient,
    pool: PgPool,
    admin_token: String,
}

impl TestContext {
    async fn new() -> Self {
        let pool = test_pool().await;
        let app = setup_app_with_pool(pool.clone());
        let client = TestClient::new(app);
        let admin_token = create_admin_and_login(&client).await;
        
        Self { client, pool, admin_token }
    }
}

#[tokio::test]
async fn test_with_context() {
    let ctx = TestContext::new().await;
    
    let response = ctx.client
        .authorized_get("/admin/stats", &ctx.admin_token)
        .await;
    
    response.assert_status(StatusCode::OK);
}
```

### 5. Clean Up Test Data

```rust
#[tokio::test]
async fn test_with_cleanup() {
    let pool = test_pool().await;
    
    // Create test data
    let user_id = create_test_user(&pool).await.id;
    
    // Run test
    let result = do_something(&pool, &user_id).await;
    assert!(result.is_ok());
    
    // Clean up
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .unwrap();
}
```

## Continuous Integration

### GitHub Actions Example

`.github/workflows/test.yml`:
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        env:
          TEST_DATABASE_URL: postgres://postgres:postgres@localhost/test_db
        run: cargo test
```

## See Also

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Database Testing Best Practices](MIGRATIONS.md)
