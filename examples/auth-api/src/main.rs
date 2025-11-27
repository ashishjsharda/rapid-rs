//! Auth API Example

use rapid_rs::prelude::*;
use rapid_rs::auth::{
    AuthConfig, AuthUser,
    InMemoryUserStore, auth_routes_with_store,
};
use axum::response::IntoResponse;

/// Protected route - requires any valid JWT
async fn protected_route(user: AuthUser) -> impl IntoResponse {
    Json(serde_json::json!({
        "message": format!("Hello, {}! You are authenticated.", user.email),
        "user_id": user.id,
        "roles": user.roles,
    }))
}

/// Admin-only route - requires "admin" role
async fn admin_route(user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    user.require_role("admin").map_err(|_| ApiError::Forbidden)?;
    
    Ok(Json(serde_json::json!({
        "message": "Welcome to the admin panel!",
        "admin_id": user.id,
    })))
}

/// Public route - no authentication required
async fn public_route() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "This is a public endpoint. Anyone can access it!",
    }))
}

#[tokio::main]
async fn main() {
    // Make sure AUTH_JWT_SECRET is set in environment
    // For development, you can use the default, but set it in production!
    std::env::set_var("AUTH_JWT_SECRET", "rapid-rs-dev-secret-change-me-in-production-make-it-at-least-32-chars");
    
    let auth_config = AuthConfig::from_env();
    let user_store = InMemoryUserStore::new();
    
    // Build routes
    let protected_routes = Router::new()
        .route("/protected", get(protected_route))
        .route("/admin", get(admin_route))
        .route("/public", get(public_route));
    
    println!("🔐 Auth API Example");
    println!("==================");
    println!();
    println!("📝 Register a user:");
    println!("   curl -X POST http://localhost:8080/auth/register -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"user@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\",\\\"name\\\":\\\"John Doe\\\"}}\"");
    println!();
    println!("🔑 Login:");
    println!("   curl -X POST http://localhost:8080/auth/login -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"user@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\"}}\"");
    println!();
    println!("🔒 Access protected route:");
    println!("   curl -X GET http://localhost:8080/protected -H \"Authorization: Bearer <access_token>\"");
    println!();

    App::new()
        .auto_configure()
        .mount(auth_routes_with_store(auth_config, user_store))
        .mount(protected_routes)
        .run()
        .await
        .unwrap();
}