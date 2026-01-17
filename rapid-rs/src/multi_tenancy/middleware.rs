//! Multi-tenancy middleware

use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{header::HOST, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use super::{TenantContext, TenantId, TenantInfo, TenantResolver};
use crate::error::ApiError;

/// Tenant extractor for handlers
pub struct TenantExtractor(pub TenantContext);

#[async_trait]
impl<S> FromRequestParts<S> for TenantExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;
    
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TenantContext>()
            .cloned()
            .map(Self)
            .ok_or_else(|| {
                let error = TenantError {
                    code: "MISSING_TENANT".to_string(),
                    message: "Tenant context not found. Did you forget the tenant middleware?".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            })
    }
}

#[derive(Serialize)]
struct TenantError {
    code: String,
    message: String,
}

/// Tenant middleware configuration
pub struct TenantMiddlewareConfig<R: TenantResolver> {
    pub resolver: Arc<R>,
    pub tenant_header: String,
    pub resolve_from_subdomain: bool,
}

impl<R: TenantResolver> TenantMiddlewareConfig<R> {
    pub fn new(resolver: R) -> Self {
        Self {
            resolver: Arc::new(resolver),
            tenant_header: "X-Tenant-ID".to_string(),
            resolve_from_subdomain: true,
        }
    }
    
    pub fn with_header(mut self, header: String) -> Self {
        self.tenant_header = header;
        self
    }
    
    pub fn resolve_from_subdomain(mut self, enable: bool) -> Self {
        self.resolve_from_subdomain = enable;
        self
    }
}

/// Tenant middleware
pub async fn tenant_middleware<R: TenantResolver + 'static>(
    State(config): State<Arc<TenantMiddlewareConfig<R>>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to resolve tenant from header first
    let tenant_id = if let Some(header_value) = request.headers().get(&config.tenant_header) {
        if let Ok(value) = header_value.to_str() {
            match config.resolver.resolve_from_header(value).await {
                Ok(id) => Some(id),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to resolve tenant from header");
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // If not found and subdomain resolution is enabled, try subdomain
    let tenant_id = if tenant_id.is_none() && config.resolve_from_subdomain {
        if let Some(host) = request.headers().get(HOST) {
            if let Ok(host_str) = host.to_str() {
                if let Some(subdomain) = extract_subdomain(host_str) {
                    match config.resolver.resolve_from_subdomain(&subdomain).await {
                        Ok(id) => Some(id),
                        Err(e) => {
                            tracing::warn!(subdomain = %subdomain, error = %e, "Failed to resolve tenant from subdomain");
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        tenant_id
    };
    
    // Get tenant configuration
    let tenant_id = match tenant_id {
        Some(id) => id,
        None => {
            let error = TenantError {
                code: "TENANT_NOT_FOUND".to_string(),
                message: "Could not identify tenant from request".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };
    
    let tenant_config = match config.resolver.get_tenant_config(&tenant_id).await {
        Ok(config) => config,
        Err(e) => {
            tracing::error!(tenant_id = %tenant_id, error = %e, "Failed to load tenant config");
            let error = TenantError {
                code: "TENANT_CONFIG_ERROR".to_string(),
                message: format!("Failed to load tenant configuration: {}", e),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
        }
    };
    
    // Check if tenant is active
    if !tenant_config.is_active {
        let error = TenantError {
            code: "TENANT_INACTIVE".to_string(),
            message: "This tenant account is not active".to_string(),
        };
        return (StatusCode::FORBIDDEN, Json(error)).into_response();
    }
    
    // Create tenant context
    let tenant_context = TenantContext::new(TenantInfo::from(tenant_config));
    
    tracing::info!(
        tenant_id = %tenant_context.tenant_id(),
        tenant_name = %tenant_context.tenant_name(),
        "Request processing for tenant"
    );
    
    // Insert tenant context into request extensions
    request.extensions_mut().insert(tenant_context);
    
    next.run(request).await
}

/// Extract subdomain from host header
fn extract_subdomain(host: &str) -> Option<String> {
    // Remove port if present
    let host = host.split(':').next().unwrap_or(host);
    
    // Split by dots
    let parts: Vec<&str> = host.split('.').collect();
    
    // Need at least 3 parts for subdomain (subdomain.domain.tld)
    if parts.len() >= 3 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_subdomain() {
        assert_eq!(
            extract_subdomain("acme.example.com"),
            Some("acme".to_string())
        );
        assert_eq!(
            extract_subdomain("acme.example.com:3000"),
            Some("acme".to_string())
        );
        assert_eq!(extract_subdomain("example.com"), None);
        assert_eq!(extract_subdomain("localhost"), None);
    }
}
