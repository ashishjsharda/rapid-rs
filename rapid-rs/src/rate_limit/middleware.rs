//! Rate limiting middleware implementation

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use serde::Serialize;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Number of requests allowed per period
    pub requests_per_period: u32,
    
    /// Time period for rate limiting
    pub period: Duration,
    
    /// Burst size (max requests in a short burst)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_period: 100,
            period: Duration::from_secs(60),
            burst_size: 10,
        }
    }
}

/// Rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::with_period(config.period)
            .unwrap()
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());
        
        Self {
            limiter: Arc::new(GovernorRateLimiter::direct(quota)),
        }
    }
    
    /// Check if request is allowed
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[derive(Serialize)]
struct RateLimitError {
    code: String,
    message: String,
    retry_after_seconds: u64,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    if limiter.check() {
        next.run(request).await
    } else {
        let error = RateLimitError {
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: "Too many requests. Please try again later.".to_string(),
            retry_after_seconds: 60,
        };
        
        (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_period: 2,
            period: Duration::from_secs(1),
            burst_size: 2,
        };
        
        let limiter = RateLimiter::new(config);
        
        // First two requests should pass
        assert!(limiter.check());
        assert!(limiter.check());
        
        // Third should fail
        assert!(!limiter.check());
    }
}