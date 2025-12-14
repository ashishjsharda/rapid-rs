use std::fs;
use std::path::Path;

pub fn generate_graphql_template(base: &Path, name: &str) -> anyhow::Result<()> {
    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rapid-rs = "0.2"
tokio = {{ version = "1", features = ["full"] }}
async-graphql = "7.0"
async-graphql-axum = "7.0"
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
sqlx = {{ version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }}
"#,
        name
    );
    fs::write(base.join("Cargo.toml"), cargo_toml)?;

    // main.rs
    let main_rs = r#"use async_graphql::{http::GraphiQLSource, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use rapid_rs::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

// GraphQL Types
#[derive(SimpleObject, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Query Root
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello from GraphQL!".to_string()
    }

    async fn user(&self, id: String) -> Option<User> {
        // Mock implementation - replace with real database queries
        Some(User {
            id: id.clone(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        })
    }

    async fn users(&self) -> Vec<User> {
        // Mock implementation
        vec![
            User {
                id: Uuid::new_v4().to_string(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: Uuid::new_v4().to_string(),
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ]
    }
}

// Mutation Root
struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, name: String, email: String) -> User {
        // Mock implementation - replace with real database insert
        User {
            id: Uuid::new_v4().to_string(),
            name,
            email,
        }
    }
}

// GraphQL handler
async fn graphql_handler(
    schema: Extension<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// GraphiQL IDE handler
async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    // Create GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    // Build GraphQL routes
    let graphql_routes = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    println!("🚀 GraphQL server starting...");
    println!("📊 GraphiQL IDE: http://localhost:8080/graphql");

    // Start server
    App::new()
        .auto_configure()
        .mount(graphql_routes)
        .run()
        .await
        .unwrap();
}
"#;
    fs::write(base.join("src/main.rs"), main_rs)?;

    // README
    let readme = format!(
        r#"# {} - GraphQL API

A rapid-rs GraphQL API project.

## Quick Start

```bash
cargo run
```

Then visit: http://localhost:8080/graphql

## Example Queries

### Get all users
```graphql
query {{
  users {{
    id
    name
    email
  }}
}}
```

### Get specific user
```graphql
query {{
  user(id: "123") {{
    id
    name
    email
  }}
}}
```

### Create a user
```graphql
mutation {{
  createUser(name: "Jane Doe", email: "jane@example.com") {{
    id
    name
    email
  }}
}}
```

## Project Structure

- `src/main.rs` - GraphQL schema and server setup
- Add your types, queries, and mutations in separate modules as your API grows

## Next Steps

1. Connect to a real database (PostgreSQL recommended)
2. Add authentication using rapid-rs auth
3. Implement subscriptions for real-time updates
4. Add custom scalars for validation
"#,
        name
    );
    fs::write(base.join("README.md"), readme)?;

    Ok(())
}
