//! Authentication middleware for protecting routes

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Json};
use serde::Serialize;
use tower::{Layer, Service};

use crate::auth::Claims;

use super::config::AuthConfig;
use super::jwt::verify_access_token;

/// Middleware that injects AuthConfig into request extensions
///
/// This must be applied before using AuthUser extractor.
pub async fn inject_auth_config(
    State(config): State<AuthConfig>,
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
        mut request: Request,
        next: Next,
    ) -> impl IntoResponse {
        let auth_header = request
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "MISSING_TOKEN".to_string(),
                        message: "Authorization header missing or invalid".to_string(),
                    }),
                )
                    .into_response();
            }
        };

        match verify_access_token(token, &config) {
            Ok(claims) => {
                // Store claims so RequireRoles doesn't have to decode again
                request.extensions_mut().insert(claims);
                next.run(request).await
            }
            Err(_) => (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    code: "INVALID_TOKEN".to_string(),
                    message: "Invalid or expired token".to_string(),
                }),
            )
                .into_response(),
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
}

impl<S> Layer<S> for RequireRoles {
    type Service = RequireRolesService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireRolesService {
            inner,
            roles: self.roles.clone(),
            require_all: self.require_all,
        }
    }
}

#[derive(Clone)]
pub struct RequireRolesService<S> {
    inner: S,
    roles: Vec<String>,
    require_all: bool,
}

impl<S> Service<Request> for RequireRolesService<S>
where
    S: Service<Request, Response = axum::response::Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let roles = self.roles.clone();
        let require_all = self.require_all;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let config = req
                .extensions()
                .get::<AuthConfig>()
                .cloned()
                .unwrap_or_else(AuthConfig::from_env);

            // Check for claims already decoded by RequireAuth
            let claims = if let Some(claims) = req.extensions().get::<Claims>() {
                claims.clone()
            } else {
                // Fallback: Decode token manually
                let auth_header = req
                    .headers()
                    .get(AUTHORIZATION)
                    .and_then(|v| v.to_str().ok());

                let token = match auth_header {
                    Some(h) if h.starts_with("Bearer ") => &h[7..],
                    _ => {
                        return Ok(unauthorized_response(
                            "MISSING_TOKEN",
                            "Authorization header missing",
                        ))
                    }
                };

                match verify_access_token(token, &config) {
                    Ok(c) => c,
                    Err(_) => {
                        return Ok(unauthorized_response(
                            "INVALID_TOKEN",
                            "Invalid or expired token",
                        ))
                    }
                }
            };

            let has_roles = if require_all {
                roles.iter().all(|r| claims.roles.contains(r))
            } else {
                roles.iter().any(|r| claims.roles.contains(r))
            };

            if !has_roles {
                return Ok((
                    StatusCode::FORBIDDEN,
                    Json(AuthErrorResponse {
                        code: "FORBIDDEN".to_string(),
                        message: format!("Required roles: {:?}", roles),
                    }),
                )
                    .into_response());
            }

            inner.call(req).await
        })
    }
}

fn unauthorized_response(code: &str, message: &str) -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AuthErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}
