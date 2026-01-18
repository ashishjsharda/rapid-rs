//! Metrics and observability
//!
//! Provides Prometheus metrics export, request tracking, and performance monitoring.

#[cfg(feature = "observability")]
pub mod prometheus;

#[cfg(feature = "observability")]
pub use prometheus::{
    MetricsExporter, 
    MetricsConfig, 
    record_request,
    record_counter,
    record_gauge,
    record_histogram,
    metrics_middleware,
};

use std::time::Instant;

/// Request metrics helper for manual tracking
pub struct RequestMetrics {
    start: Instant,
    path: String,
    method: String,
}

impl RequestMetrics {
    /// Create a new request metrics tracker
    pub fn new(method: String, path: String) -> Self {
        Self {
            start: Instant::now(),
            path,
            method,
        }
    }
    
    /// Finish tracking and record the metrics
    #[cfg(feature = "observability")]
    pub fn finish(self, status_code: u16) {
        let duration = self.start.elapsed();
        record_request(&self.method, &self.path, status_code, duration);
    }
    
    #[cfg(not(feature = "observability"))]
    pub fn finish(self, _status_code: u16) {
        // No-op when observability feature is disabled
    }
    
    /// Get the elapsed time without finishing
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

/// Metrics summary for health checks and monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_response_time: f64,
    pub requests_per_second: f64,
}

impl Default for MetricsSummary {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            avg_response_time: 0.0,
            requests_per_second: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_metrics() {
        let metrics = RequestMetrics::new("GET".to_string(), "/test".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(metrics.elapsed().as_millis() >= 10);
        metrics.finish(200);
    }
}