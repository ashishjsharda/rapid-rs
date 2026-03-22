//! GraphQL API Example
//! Run with:
//!   cargo run --example graphql-api --features graphql

#[cfg(feature = "graphql")]
use rapid_rs::graphql::{graphql_routes, EmptyMutation, EmptySubscription, Object, Schema};

#[cfg(feature = "graphql")]
struct QueryRoot;

#[cfg(feature = "graphql")]
#[Object]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello from rapid-rs GraphQL!"
    }

    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn users(&self) -> Vec<User> {
        vec![
            User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
            User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
        ]
    }
}

#[cfg(feature = "graphql")]
use rapid_rs::graphql::SimpleObject;

#[cfg(feature = "graphql")]
#[derive(SimpleObject)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[cfg(feature = "graphql")]
use rapid_rs::prelude::*;

#[tokio::main]
async fn main() {
    #[cfg(feature = "graphql")]
    {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

        App::new()
            .auto_configure()
            .mount(graphql_routes(schema))
            .run()
            .await
            .unwrap();
    }

    #[cfg(not(feature = "graphql"))]
    {
        eprintln!("Please enable the 'graphql' feature: cargo run --example graphql-api --features graphql");
    }
}
