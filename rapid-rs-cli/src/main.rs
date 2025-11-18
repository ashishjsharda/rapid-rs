use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "rapid")]
#[command(about = "CLI tool for rapid-rs framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new rapid-rs project
    New {
        /// Project name
        name: String,
        
        /// Template to use (rest-api, graphql, grpc)
        #[arg(short, long, default_value = "rest-api")]
        template: String,
    },
    
    /// Run the project in development mode with hot reload
    Dev,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template } => {
            create_project(&name, &template)?;
        }
        Commands::Dev => {
            run_dev_mode()?;
        }
    }

    Ok(())
}

fn create_project(name: &str, template: &str) -> anyhow::Result<()> {
    println!("🚀 Creating new rapid-rs project: {}", name);
    
    if template != "rest-api" {
        anyhow::bail!("Only 'rest-api' template is currently supported");
    }

    let project_path = Path::new(name);
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create project structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("config"))?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rapid-rs = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
validator = {{ version = "0.18", features = ["derive"] }}
"#,
        name
    );
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Create main.rs with full example
    let main_rs = r#"use rapid_rs::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: Uuid,
    email: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    
    #[validate(length(min = 2, max = 100))]
    name: String,
}

type Database = Arc<Mutex<HashMap<Uuid, User>>>;

async fn create_user(
    State(db): State<Database>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> ApiResult<User> {
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        name: payload.name,
        created_at: Utc::now(),
    };

    db.lock().unwrap().insert(user.id, user.clone());
    Ok(Json(user))
}

async fn list_users(State(db): State<Database>) -> ApiResult<Vec<User>> {
    let users: Vec<User> = db.lock().unwrap().values().cloned().collect();
    Ok(Json(users))
}

async fn get_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let user = db
        .lock()
        .unwrap()
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User {} not found", id)))?
        .clone();
    Ok(Json(user))
}

fn routes() -> Router<Database> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
}

#[tokio::main]
async fn main() {
    let db: Database = Arc::new(Mutex::new(HashMap::new()));

    App::new()
        .auto_configure()
        .mount(routes().with_state(db))
        .run()
        .await
        .unwrap();
}
"#;
    fs::write(project_path.join("src/main.rs"), main_rs)?;

    // Create config files
    let default_config = r#"[server]
host = "0.0.0.0"
port = 3000

[database]
url = "postgres://localhost/rapid_rs"
max_connections = 10
"#;
    fs::write(project_path.join("config/default.toml"), default_config)?;

    let local_config = r#"# Override settings for local development
# This file is gitignored by default

[server]
port = 3000
"#;
    fs::write(project_path.join("config/local.toml"), local_config)?;

    // Create .gitignore
    let gitignore = r#"/target
/config/local.toml
.env
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    // Create README
    let readme = format!(
        r#"# {}

A rapid-rs API project.

## Getting Started

```bash
# Run the server
cargo run

# The server will start at http://localhost:3000
# Swagger UI: http://localhost:3000/docs
# Health check: http://localhost:3000/health
```

## API Endpoints

- `POST /users` - Create a new user
- `GET /users` - List all users
- `GET /users/:id` - Get a user by ID

## Configuration

Configuration is loaded from:
1. `config/default.toml` - Default settings
2. `config/local.toml` - Local overrides (gitignored)
3. Environment variables (prefixed with `APP__`)

Example:
```bash
APP__SERVER__PORT=8080 cargo run
```

## Development

```bash
# Run with hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```
"#,
        name
    );
    fs::write(project_path.join("README.md"), readme)?;

    println!("✅ Project created successfully!");
    println!("\n📦 Next steps:");
    println!("   cd {}", name);
    println!("   cargo run");
    println!("\n🌐 Your API will be available at:");
    println!("   http://localhost:3000");
    println!("   http://localhost:3000/docs (Swagger UI)");

    Ok(())
}

fn run_dev_mode() -> anyhow::Result<()> {
    println!("🔥 Starting development mode with hot reload...");
    
    // Check if cargo-watch is installed
    let status = Command::new("cargo")
        .args(&["watch", "--version"])
        .output();
    
    if status.is_err() {
        println!("⚠️  cargo-watch is not installed.");
        println!("Installing cargo-watch...");
        
        let install_status = Command::new("cargo")
            .args(&["install", "cargo-watch"])
            .status()?;
        
        if !install_status.success() {
            anyhow::bail!("Failed to install cargo-watch");
        }
    }

    // Run cargo watch
    let status = Command::new("cargo")
        .args(&["watch", "-x", "run"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Development server exited with error");
    }

    Ok(())
}
