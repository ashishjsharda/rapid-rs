//! Authentication middleware for protecting routes

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use super::config::AuthConfig;
use super::jwt::verify_access_token;

/// Middleware that injects AuthConfig into request extensions
/// 
/// This must be applied before using AuthUser extractor.
pub async fn inject_auth_config(
    config: AuthConfig,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    request.extensions_mut().insert(config);
    next.run(request).await
}

/// Middleware layer for requiring authentication
/// 
/// Use this to protect entire route groups.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::{RequireAuth, AuthConfig};
/// use axum::{Router, routing::get, middleware};
/// 
/// let config = AuthConfig::default();
/// 
/// let protected_routes = Router::new()
///     .route("/profile", get(get_profile))
///     .route("/settings", get(get_settings))
///     .layer(middleware::from_fn_with_state(
///         config.clone(),
///         RequireAuth::middleware,
///     ));
/// ```
pub struct RequireAuth;

#[derive(Serialize)]
struct AuthErrorResponse {
    code: String,
    message: String,
}

impl RequireAuth {
    /// Middleware function that requires a valid JWT token
    pub async fn middleware(
        config: axum::extract::State<AuthConfig>,
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Extract Authorization header
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok());
        
        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                &header[7..]
            }
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "MISSING_TOKEN".to_string(),
                        message: "Authorization header missing or invalid".to_string(),
                    }),
                ).into_response();
            }
        };
        
        // Verify token
        match verify_access_token(token, &config) {
            Ok(_claims) => {
                // Token is valid, proceed with request
                next.run(request).await
            }
            Err(_) => {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "INVALID_TOKEN".to_string(),
                        message: "Invalid or expired token".to_string(),
                    }),
                ).into_response()
            }
        }
    }
}

/// Middleware that requires specific roles
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::RequireRoles;
/// use axum::{Router, routing::get, middleware};
/// 
/// let admin_routes = Router::new()
///     .route("/admin/users", get(list_users))
///     .layer(RequireRoles::new(vec!["admin"]));
/// ```
#[derive(Clone)]
pub struct RequireRoles {
    roles: Vec<String>,
    require_all: bool,
}

impl RequireRoles {
    /// Create a new RequireRoles middleware requiring any of the specified roles
    pub fn any(roles: Vec<impl Into<String>>) -> Self {
        Self {
            roles: roles.into_iter().map(|r| r.into()).collect(),
            require_all: false,
        }
    }
    
    /// Create a new RequireRoles middleware requiring all of the specified roles
    pub fn all(roles: Vec<impl Into<String>>) -> Self {
        Self {
            roles: roles.into_iter().map(|r| r.into()).collect(),
            require_all: true,
        }
    }
    
    /// Middleware function
    pub async fn middleware(
        roles: Vec<String>,
        require_all: bool,
        config: axum::extract::State<AuthConfig>,
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Extract Authorization header
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok());
        
        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                &header[7..]
            }
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "MISSING_TOKEN".to_string(),
                        message: "Authorization header missing or invalid".to_string(),
                    }),
                ).into_response();
            }
        };
        
        // Verify token
        let claims = match verify_access_token(token, &config) {
            Ok(claims) => claims,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "INVALID_TOKEN".to_string(),
                        message: "Invalid or expired token".to_string(),
                    }),
                ).into_response();
            }
        };
        
        // Check roles
        let has_required_roles = if require_all {
            roles.iter().all(|role| claims.roles.contains(role))
        } else {
            roles.iter().any(|role| claims.roles.contains(role))
        };
        
        if !has_required_roles {
            return (
                StatusCode::FORBIDDEN,
                Json(AuthErrorResponse {
                    code: "FORBIDDEN".to_string(),
                    message: format!(
                        "Required roles: {:?} ({})",
                        roles,
                        if require_all { "all" } else { "any" }
                    ),
                }),
            ).into_response();
        }
        
        next.run(request).await
    }
}

/// Extension trait for Router to easily add auth protection
pub trait AuthRouterExt {
    /// Protect all routes with authentication
    fn require_auth(self, config: AuthConfig) -> Self;
    
    /// Protect all routes requiring specific roles
    fn require_roles(self, config: AuthConfig, roles: Vec<&str>, require_all: bool) -> Self;
}
