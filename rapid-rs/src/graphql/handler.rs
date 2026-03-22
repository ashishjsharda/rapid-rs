//! Axum handlers for GraphQL
//!
//! Custom integration using axum 0.7 primitives to avoid the
//! async-graphql-axum version conflict (v6 = axum 0.6, v7 = axum 0.8).

use async_graphql::{ObjectType, SubscriptionType, Schema};
use async_graphql::http::GraphiQLSource;
use axum::{
    body::Bytes,
    extract::Extension,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

/// Handle GraphQL POST requests
pub async fn graphql_handler<Q, M, S>(
    Extension(schema): Extension<Schema<Q, M, S>>,
    body: Bytes,
) -> Response
where
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
    let request: async_graphql::Request = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };

    let response = schema.execute(request).await;

    match serde_json::to_string(&response) {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Serve the GraphiQL playground UI
pub async fn graphiql_handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Create GraphQL routes for a given schema
///
/// Mounts:
/// - POST /graphql - GraphQL endpoint
/// - GET /graphql/playground - GraphiQL playground UI
pub fn graphql_routes<Q, M, S>(schema: Schema<Q, M, S>) -> Router
where
    Q: ObjectType + Clone + 'static,
    M: ObjectType + Clone + 'static,
    S: SubscriptionType + Clone + 'static,
{
    Router::new()
        .route("/graphql", post(graphql_handler::<Q, M, S>))
        .route("/graphql/playground", get(graphiql_handler))
        .layer(Extension(schema))
}
