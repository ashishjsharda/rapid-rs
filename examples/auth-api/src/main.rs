//! Auth API Example

use axum::{middleware::from_fn_with_state, response::IntoResponse};
use rapid_rs::auth::{
    auth_routes_with_store, create_token_pair, hash_password,
    middleware::{inject_auth_config, RequireRoles},
    models::AuthUserInfo,
    AuthAppState, AuthConfig, AuthResponse, AuthUser, CreateUserData, InMemoryUserStore,
    RegisterRequest, UserStore,
};
use rapid_rs::prelude::*;

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
    user.require_role("admin")
        .map_err(|_| ApiError::Forbidden)?;

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

/// This handler logic is only reached if the Middleware allows it
async fn middleware_admin() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "success",
        "message": "You have the ADMIN role, so you can see this!",
    }))
}

/// This handler logic is only reached if the Middleware allows it
async fn middleware_user() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "success",
        "message": "You have the USER role, so you can see this!",
    }))
}

//  Hack: Special endpoint to register an ADMIN for testing purposes
async fn register_admin(
    State(state): State<AuthAppState<InMemoryUserStore>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let password_hash = hash_password(&payload.password, &state.config)?;

    let user = state
        .user_store
        .create(CreateUserData {
            email: payload.email,
            name: payload.name,
            password_hash,
        })
        .await?;

    let token_pair = create_token_pair(
        &user.id,
        &user.email,
        vec!["admin".to_string()],
        &state.config,
    )?;

    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: vec!["admin".to_string()],
        },
    }))
}

#[tokio::main]
async fn main() {
    // Make sure AUTH_JWT_SECRET is set in environment
    // For development, you can use the default, but set it in production!
    let mut auth_config = AuthConfig::from_env();
    auth_config.jwt_secret =
        "rapid-rs-dev-secret-change-me-in-production-make-it-at-least-32-chars".to_string();

    let user_store = InMemoryUserStore::new();

    let app_state = AuthAppState {
        config: auth_config.clone(),
        user_store: user_store.clone(),
    };

    let admin_middleware_routes = Router::new()
        .route("/middleware/admin", get(middleware_admin))
        .layer(RequireRoles::any(vec!["admin"]));

    let user_middleware_routes = Router::new()
        .route("/middleware/user", get(middleware_user))
        .layer(RequireRoles::any(vec!["user"]));

    // Build routes
    let protected_routes = Router::new()
        .route("/protected", get(protected_route))
        .route("/admin", get(admin_route))
        .route("/public", get(public_route));

    // Admin Registration Route
    let auth_extras = Router::new()
        .route("/auth/register-admin", post(register_admin))
        .with_state(app_state);

    println!("🔐 Auth API Example");
    println!("==================");
    println!();
    println!("📝 Register a user:");
    println!("   curl -X POST http://localhost:3000/auth/register -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"user@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\",\\\"name\\\":\\\"John Doe\\\"}}\"");
    println!();
    println!("🔑 Login:");
    println!("   curl -X POST http://localhost:3000/auth/login -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"user@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\"}}\"");
    println!();
    println!("🔒 Access protected route:");
    println!("   curl -X GET http://localhost:3000/protected -H \"Authorization: Bearer <access_token>\"");
    println!();
    println!("Scenario 1: Regular User");
    println!("> Register a regular User:");
    println!("  curl -X POST http://localhost:3000/auth/register -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"user@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\",\\\"name\\\":\\\"Basic Joe\\\"}}\"");
    println!();
    println!("> Test Admin Route (Should fail with code 403 Forbidden):");
    println!("   curl -X GET http://localhost:3000/middleware/admin -H \"Authorization: Bearer <access_token>\"");
    println!();
    println!("Scenario 2: Admin User");
    println!("> Register an ADMIN User:");
    println!("  curl -X POST http://localhost:3000/auth/register-admin -H \"Content-Type: application/json\" -d \"{{\\\"email\\\":\\\"admin@example.com\\\",\\\"password\\\":\\\"SecurePass123\\\",\\\"name\\\":\\\"Admin Joe\\\"}}\"");
    println!();
    println!("> Test Admin Route (Should succed):");
    println!("   curl -X GET http://localhost:3000/middleware/admin -H \"Authorization: Bearer <access_token>\"");

    println!();

    let api_routes = Router::new()
        .merge(protected_routes)
        .merge(admin_middleware_routes)
        .merge(user_middleware_routes)
        .merge(auth_extras)
        .layer(from_fn_with_state(auth_config.clone(), inject_auth_config));

    App::new()
        .auto_configure()
        .mount(auth_routes_with_store(auth_config, user_store))
        .mount(api_routes)
        .run()
        .await
        .unwrap();
}
