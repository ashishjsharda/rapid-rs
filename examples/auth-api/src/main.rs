//! Auth API Example
//! 
//! Demonstrates JWT authentication with rapid-rs.
//! 
//! ## Endpoints:
//! - POST /auth/register - Register a new user
//! - POST /auth/login - Login and get tokens
//! - POST /auth/refresh - Refresh access token
//! - POST /auth/logout - Logout (client-side token discard)
//! - GET /auth/me - Get current user info (protected)
//! 
//! ## Protected Routes:
//! - GET /protected - Requires authentication
//! - GET /admin - Requires "admin" role

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
    // Check for admin role
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
    // Load auth config from environment or use defaults
    // In production, set AUTH_JWT_SECRET environment variable!
    let auth_config = AuthConfig::from_env();
    
    // For demo purposes, we're using an in-memory user store
    // In production, implement UserStore trait for your database
    let user_store = InMemoryUserStore::new();
    
    // Build protected routes
    let protected_routes = Router::new()
        .route("/protected", get(protected_route))
        .route("/admin", get(admin_route));
    
    // Build public routes
    let public_routes = Router::new()
        .route("/public", get(public_route));
    
    println!("🔐 Auth API Example");
    println!("==================");
    println!();
    println!("📝 Register a user:");
    println!("   curl -X POST http://localhost:8080/auth/register \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"email\": \"user@example.com\", \"password\": \"SecurePass123\", \"name\": \"John Doe\"}}'");
    println!();
    println!("🔑 Login:");
    println!("   curl -X POST http://localhost:8080/auth/login \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"email\": \"user@example.com\", \"password\": \"SecurePass123\"}}'");
    println!();
    println!("🔒 Access protected route:");
    println!("   curl http://localhost:8080/protected \\");
    println!("     -H 'Authorization: Bearer <access_token>'");
    println!();
    println!("🔄 Refresh token:");
    println!("   curl -X POST http://localhost:8080/auth/refresh \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"refresh_token\": \"<refresh_token>\"}}'");
    println!();

    // Build and run the app
    App::new()
        .auto_configure()
        .mount(auth_routes_with_store(auth_config.clone(), user_store))
        .mount(protected_routes)
        .mount(public_routes)
        .run()
        .await
        .unwrap();
}
