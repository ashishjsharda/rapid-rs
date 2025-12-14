//! Testing utilities for rapid-rs applications
//!
//! Provides helpers for testing API endpoints, database interactions,
//! and authentication flows.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde::{de::DeserializeOwned, Serialize};
use tower::ServiceExt;

/// Test client for making requests to your API
pub struct TestClient {
    app: Router,
}

impl TestClient {
    /// Create a new test client with the given router
    pub fn new(app: Router) -> Self {
        Self { app }
    }
    
    /// Make a GET request
    pub async fn get(&self, uri: &str) -> TestResponse {
        self.request(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
    }
    
    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&self, uri: &str, body: &T) -> TestResponse {
        let json_body = serde_json::to_string(body).unwrap();
        
        self.request(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(json_body))
                .unwrap(),
        )
        .await
    }
    
    /// Make a PUT request with JSON body
    pub async fn put<T: Serialize>(&self, uri: &str, body: &T) -> TestResponse {
        let json_body = serde_json::to_string(body).unwrap();
        
        self.request(
            Request::builder()
                .uri(uri)
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(json_body))
                .unwrap(),
        )
        .await
    }
    
    /// Make a PATCH request with JSON body
    pub async fn patch<T: Serialize>(&self, uri: &str, body: &T) -> TestResponse {
        let json_body = serde_json::to_string(body).unwrap();
        
        self.request(
            Request::builder()
                .uri(uri)
                .method("PATCH")
                .header("content-type", "application/json")
                .body(Body::from(json_body))
                .unwrap(),
        )
        .await
    }
    
    /// Make a DELETE request
    pub async fn delete(&self, uri: &str) -> TestResponse {
        self.request(
            Request::builder()
                .uri(uri)
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }
    
    /// Make a request with authorization header
    pub async fn authorized_get(&self, uri: &str, token: &str) -> TestResponse {
        self.request(
            Request::builder()
                .uri(uri)
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }
    
    /// Make an authorized POST request
    pub async fn authorized_post<T: Serialize>(
        &self,
        uri: &str,
        token: &str,
        body: &T,
    ) -> TestResponse {
        let json_body = serde_json::to_string(body).unwrap();
        
        self.request(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(json_body))
                .unwrap(),
        )
        .await
    }
    
    /// Make a custom request
    async fn request(&self, req: Request<Body>) -> TestResponse {
        let response = self
            .app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to send request");
        
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        
        TestResponse {
            status,
            headers,
            body: body_bytes.to_vec(),
        }
    }
}

/// Test response wrapper
pub struct TestResponse {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    body: Vec<u8>,
}

impl TestResponse {
    /// Get the response body as a string
    pub fn text(&self) -> String {
        String::from_utf8(self.body.clone()).expect("Response body is not valid UTF-8")
    }
    
    /// Deserialize JSON response body
    pub fn json<T: DeserializeOwned>(&self) -> T {
        serde_json::from_slice(&self.body).expect("Failed to deserialize JSON response")
    }
    
    /// Assert the status code
    pub fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status, expected,
            "Expected status {}, got {}. Body: {}",
            expected,
            self.status,
            self.text()
        );
        self
    }
    
    /// Assert response contains text
    pub fn assert_text_contains(&self, expected: &str) -> &Self {
        let body = self.text();
        assert!(
            body.contains(expected),
            "Expected body to contain '{}', got: {}",
            expected, body
        );
        self
    }
    
    /// Check if status is success (2xx)
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

/// Database test utilities
#[cfg(feature = "db-tests")]
pub mod db {
    use sqlx::PgPool;
    
    /// Create a test database pool
    /// 
    /// This uses a test database URL from the TEST_DATABASE_URL environment variable.
    pub async fn test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/rapid_rs_test".to_string());
        
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }
    
    /// Clean up test database (truncate all tables)
    pub async fn cleanup(pool: &PgPool) {
        // This is a simple implementation - you might want to make it more sophisticated
        sqlx::query("TRUNCATE TABLE users CASCADE")
            .execute(pool)
            .await
            .ok(); // Ignore errors if table doesn't exist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Json, Router};
    use serde_json::json;
    
    async fn hello() -> Json<serde_json::Value> {
        Json(json!({"message": "Hello, World!"}))
    }
    
    async fn echo(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
        Json(body)
    }
    
    #[tokio::test]
    async fn test_client_get() {
        let app = Router::new().route("/hello", get(hello));
        let client = TestClient::new(app);
        
        let response = client.get("/hello").await;
        
        response.assert_status(StatusCode::OK);
        let json: serde_json::Value = response.json();
        assert_eq!(json["message"], "Hello, World!");
    }
    
    #[tokio::test]
    async fn test_client_post() {
        let app = Router::new().route("/echo", axum::routing::post(echo));
        let client = TestClient::new(app);
        
        let body = json!({"test": "data"});
        let response = client.post("/echo", &body).await;
        
        response.assert_status(StatusCode::OK);
        let json: serde_json::Value = response.json();
        assert_eq!(json["test"], "data");
    }
}
