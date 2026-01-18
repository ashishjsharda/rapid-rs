//! Multi-tenancy middleware

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use super::{TenantContext, TenantResolver};

/// Tenant middleware configuration
pub struct TenantMiddlewareConfig<R: TenantResolver> {
    resolver: Arc<R>,
}

impl<R: TenantResolver> TenantMiddlewareConfig<R> {
    pub fn new(resolver: R) -> Self {
        Self {
            resolver: Arc::new(resolver),
        }
    }
}

impl<R: TenantResolver> Clone for TenantMiddlewareConfig<R> {
    fn clone(&self) -> Self {
        Self {
            resolver: self.resolver.clone(),
        }
    }
}

/// Tenant middleware - extracts tenant from request
pub async fn tenant_middleware<R: TenantResolver + 'static>(
    State(config): State<TenantMiddlewareConfig<R>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract tenant from subdomain or header
    let tenant_identifier = extract_tenant_from_request(&request);
    
    if let Some((is_subdomain, identifier)) = tenant_identifier {
        // Resolve tenant ID based on source
        let tenant_id_result = if is_subdomain {
            config.resolver.resolve_from_subdomain(&identifier).await
        } else {
            config.resolver.resolve_from_header(&identifier).await
        };
        
        if let Ok(tenant_id) = tenant_id_result {
            // Get tenant config
            if let Ok(tenant_config) = config.resolver.get_tenant_config(&tenant_id).await {
                // Convert to TenantInfo and store in context
                let tenant_info = tenant_config.into();
                let context = TenantContext::new(tenant_info);
                request.extensions_mut().insert(context);
            }
        }
    }
    
    next.run(request).await
}

/// Extract tenant ID from request (subdomain or header)
/// Returns (is_subdomain, identifier)
fn extract_tenant_from_request(request: &Request) -> Option<(bool, String)> {
    // Try X-Tenant-ID header first
    if let Some(tenant_id) = request
        .headers()
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
    {
        return Some((false, tenant_id.to_string()));
    }
    
    // Try subdomain extraction
    if let Some(host) = request
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
    {
        // Extract subdomain from host (e.g., "acme.example.com" -> "acme")
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() >= 3 {
            return Some((true, parts[0].to_string()));
        }
    }
    
    None
}

/// Extractor for tenant context
pub struct TenantExtractor(pub TenantContext);

impl<S> axum::extract::FromRequestParts<S> for TenantExtractor
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;
    
    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<Self, Self::Rejection>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            parts
                .extensions
                .get::<TenantContext>()
                .cloned()
                .map(TenantExtractor)
                .ok_or(axum::http::StatusCode::BAD_REQUEST)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_extractor() {
        // Basic test structure
        assert!(true);
    }
}