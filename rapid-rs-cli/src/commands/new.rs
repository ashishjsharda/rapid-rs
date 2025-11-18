use std::fs;
use std::path::Path;

pub fn create_project(name: &str, template: &str) -> anyhow::Result<()> {
    println!("🚀 Creating new Rapid-RS project: {}", name);
    println!("📦 Using template: {}", template);

    // Create project directory
    let project_path = Path::new(name);
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_path)?;

    // Create directory structure
    create_directory_structure(project_path)?;

    // Generate files from templates
    match template {
        "rest-api" => generate_rest_api_template(project_path, name)?,
        _ => anyhow::bail!("Unknown template: {}", template),
    }

    println!("\n✅ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  cargo run");
    println!("\nAPI docs will be available at http://localhost:3000/docs");

    Ok(())
}

fn create_directory_structure(base: &Path) -> anyhow::Result<()> {
    let dirs = vec![
        "src/routes",
        "src/models",
        "src/services",
        "migrations",
        "config",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(base.join(dir))?;
    }

    Ok(())
}

fn generate_rest_api_template(base: &Path, name: &str) -> anyhow::Result<()> {
    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rapid-rs = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
uuid = {{ version = "1", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
validator = {{ version = "0.18", features = ["derive"] }}
sqlx = {{ version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }}
utoipa = {{ version = "4", features = ["axum_extras", "uuid", "chrono"] }}
"#,
        name
    );
    fs::write(base.join("Cargo.toml"), cargo_toml)?;

    // .env
    let env_content = format!(
        r#"DATABASE_URL=postgres://postgres:postgres@localhost/{}
APP_HOST=0.0.0.0
APP_PORT=3000
APP_ENVIRONMENT=development
APP_LOG_LEVEL=info
"#,
        name.replace('-', "_")
    );
    fs::write(base.join(".env"), env_content)?;

    // .gitignore
    let gitignore = r#"/target
/Cargo.lock
.env
.env.local
config/local.toml
"#;
    fs::write(base.join(".gitignore"), gitignore)?;

    // config/default.toml
    let config = r#"host = "0.0.0.0"
port = 3000
environment = "development"
log_level = "info"
database_url = "postgres://postgres:postgres@localhost/myapp"
"#;
    fs::write(base.join("config/default.toml"), config)?;

    // src/main.rs
    let main_rs = r#"use rapid_rs::prelude::*;

mod routes;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    App::new()
        .auto_configure()
        .with_database()
        .await?
        .routes(routes::users::routes())
        .run()
        .await
}
"#;
    fs::write(base.join("src/main.rs"), main_rs)?;

    // src/models/mod.rs
    let models_mod = r#"pub mod user;
"#;
    fs::write(base.join("src/models/mod.rs"), models_mod)?;

    // src/models/user.rs
    let user_model = r#"use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}
"#;
    fs::write(base.join("src/models/user.rs"), user_model)?;

    // src/routes/mod.rs
    let routes_mod = r#"pub mod users;
"#;
    fs::write(base.join("src/routes/mod.rs"), routes_mod)?;

    // src/routes/users.rs
    let users_routes = r#"use rapid_rs::prelude::*;
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User};

pub fn routes() -> Router<rapid_rs::app::AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).patch(update_user).delete(delete_user))
}

/// List all users
async fn list_users(
    State(state): State<rapid_rs::app::AppState>,
) -> ApiResult<Json<Vec<User>>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users))
}

/// Get a user by ID
async fn get_user(
    State(state): State<rapid_rs::app::AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    Ok(Json(user))
}

/// Create a new user
async fn create_user(
    State(state): State<rapid_rs::app::AppState>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> ApiResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, name)
        VALUES ($1, $2, $3)
        RETURNING id, email, name, created_at, updated_at
        "#,
        Uuid::new_v4(),
        payload.email,
        payload.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

/// Update a user
async fn update_user(
    State(state): State<rapid_rs::app::AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> ApiResult<Json<User>> {
    // First check if user exists
    let _ = get_user(State(state.clone()), Path(id)).await?;

    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET email = COALESCE($2, email),
            name = COALESCE($3, name),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, name, created_at, updated_at
        "#,
        id,
        payload.email,
        payload.name
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

/// Delete a user
async fn delete_user(
    State(state): State<rapid_rs::app::AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("User with id {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
"#;
    fs::write(base.join("src/routes/users.rs"), users_routes)?;

    // migrations/20240101000000_create_users.sql
    let migration = r#"-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on email for faster lookups
CREATE INDEX idx_users_email ON users(email);
"#;
    fs::write(
        base.join("migrations/20240101000000_create_users.sql"),
        migration,
    )?;

    // README.md
    let readme = format!(
        r#"# {}

A Rapid-RS REST API project.

## Quick Start

1. Install dependencies:
```bash
cargo build
```

2. Set up your database:
```bash
# Create database
createdb {}

# The migrations will run automatically on startup
```

3. Run the application:
```bash
cargo run
```

4. Visit the API docs:
```
http://localhost:3000/docs
```

## Available Endpoints

- `GET /users` - List all users
- `POST /users` - Create a new user
- `GET /users/:id` - Get a user by ID
- `PATCH /users/:id` - Update a user
- `DELETE /users/:id` - Delete a user

## Health Check

```
http://localhost:3000/health
```

## Configuration

Configuration is loaded from:
1. `config/default.toml` - Default configuration
2. `config/{{environment}}.toml` - Environment-specific config
3. `.env` - Environment variables (gitignored)

You can override any setting using environment variables prefixed with `APP_`:
- `APP_HOST`
- `APP_PORT`
- `APP_DATABASE_URL`
"#,
        name,
        name.replace('-', "_")
    );
    fs::write(base.join("README.md"), readme)?;

    Ok(())
}
"#;
    fs::write(base.join("rapid-rs-cli/src/commands/new.rs"), new_rs)?;

    Ok(())
}
