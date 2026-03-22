//! GraphQL support for rapid-rs
//!
//! Provides GraphQL schema building, query execution, and Axum integration
//! powered by async-graphql.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::graphql::{GraphQLConfig, graphql_routes};
//! use async_graphql::{Object, Schema, EmptyMutation, EmptySubscription};
//!
//! struct QueryRoot;
//!
//! #[Object]
//! impl QueryRoot {
//!     async fn hello(&self) -> &str {
//!         "Hello from rapid-rs GraphQL!"
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
//!         .finish();
//!
//!     App::new()
//!         .auto_configure()
//!         .mount(graphql_routes(schema))
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod handler;
pub mod schema;

pub use handler::graphql_routes;
pub use schema::SchemaBuilder;

pub use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, Subscription,
    InputObject, Enum, Union, Interface, Result as GraphQLResult,
};

/// GraphQL configuration
#[derive(Debug, Clone)]
pub struct GraphQLConfig {
    /// Path for GraphQL endpoint (default: /graphql)
    pub endpoint: String,
    /// Path for GraphQL Playground UI (default: /graphql/playground)
    pub playground_path: String,
    /// Whether to enable the playground UI (default: true)
    pub enable_playground: bool,
    /// Maximum query depth (default: 10)
    pub max_depth: Option<usize>,
    /// Maximum query complexity (default: None)
    pub max_complexity: Option<usize>,
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            endpoint: "/graphql".to_string(),
            playground_path: "/graphql/playground".to_string(),
            enable_playground: true,
            max_depth: Some(10),
            max_complexity: None,
        }
    }
}

impl GraphQLConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn with_playground_path(mut self, path: impl Into<String>) -> Self {
        self.playground_path = path.into();
        self
    }

    pub fn disable_playground(mut self) -> Self {
        self.enable_playground = false;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_max_complexity(mut self, complexity: usize) -> Self {
        self.max_complexity = Some(complexity);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_config_defaults() {
        let config = GraphQLConfig::default();
        assert_eq!(config.endpoint, "/graphql");
        assert_eq!(config.playground_path, "/graphql/playground");
        assert!(config.enable_playground);
        assert_eq!(config.max_depth, Some(10));
    }

    #[test]
    fn test_graphql_config_builder() {
        let config = GraphQLConfig::new()
            .with_endpoint("/api/graphql")
            .disable_playground()
            .with_max_depth(5)
            .with_max_complexity(100);

        assert_eq!(config.endpoint, "/api/graphql");
        assert!(!config.enable_playground);
        assert_eq!(config.max_depth, Some(5));
        assert_eq!(config.max_complexity, Some(100));
    }
}
