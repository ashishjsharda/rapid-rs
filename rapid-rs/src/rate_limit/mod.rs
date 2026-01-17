//! Rate limiting middleware

pub mod middleware;

pub use middleware::{RateLimiter, RateLimitConfig, rate_limit_middleware};

use std::time::Duration;

impl RateLimitConfig {
    pub fn per_minute(requests: u32) -> Self {
        Self {
            requests_per_period: requests,
            period: Duration::from_secs(60),
            burst_size: requests,
        }
    }
    
    pub fn per_hour(requests: u32) -> Self {
        Self {
            requests_per_period: requests,
            period: Duration::from_secs(3600),
            burst_size: requests / 60,
        }
    }
}