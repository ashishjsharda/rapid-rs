//! Prometheus metrics exporter

#[cfg(feature = "metrics")]
use axum::{
    routing::get,
    Router,
};
#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
#[cfg(feature = "metrics")]
use std::time::Duration;

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Endpoint path for metrics
    pub endpoint: String,
    
    /// Histogram buckets for latency metrics
    pub latency_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            endpoint: "/metrics".to_string(),
            latency_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        }
    }
}

/// Metrics exporter
#[cfg(feature = "metrics")]
pub struct MetricsExporter {
    handle: PrometheusHandle,
    config: MetricsConfig,
}

#[cfg(feature = "metrics")]
impl MetricsExporter {
    /// Create a new metrics exporter
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        let builder = PrometheusBuilder::new();
        
        let builder = builder.set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &config.latency_buckets,
        ).unwrap();
        
        let handle = builder
            .install_recorder()
            .expect("Failed to install Prometheus recorder");
        
        tracing::info!("Metrics exporter initialized at {}", config.endpoint);
        
        Self { handle, config }
    }
    
    /// Get metrics as text
    pub fn render(&self) -> String {
        self.handle.render()
    }
    
    /// Create routes for metrics endpoint
    pub fn routes(&self) -> Router {
        let handle = self.handle.clone();
        
        Router::new().route(
            &self.config.endpoint,
            get(move || {
                let handle = handle.clone();
                async move { handle.render() }
            }),
        )
    }
}

#[cfg(feature = "metrics")]
impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Record an HTTP request
#[cfg(feature = "metrics")]
pub fn record_request(method: &str, path: &str, status_code: u16, duration: Duration) {
    use metrics::{counter, histogram};
    
    let labels = [
        ("method", method.to_string()),
        ("path", path.to_string()),
        ("status", status_code.to_string()),
    ];
    
    // Increment request counter
    counter!("http_requests_total", &labels).increment(1);
    
    // Record request duration
    histogram!("http_request_duration_seconds", &labels).record(duration.as_secs_f64());
    
    // Record status code distribution
    if status_code >= 500 {
        counter!("http_requests_errors_total", &labels).increment(1);
    }
}

/// Record a custom metric
#[cfg(feature = "metrics")]
pub fn record_counter(name: &str, value: u64, labels: &[(&str, String)]) {
    use metrics::counter;
    counter!(name, labels).increment(value);
}

/// Record a gauge metric
#[cfg(feature = "metrics")]
pub fn record_gauge(name: &str, value: f64, labels: &[(&str, String)]) {
    use metrics::gauge;
    gauge!(name, labels).set(value);
}

/// Record a histogram metric
#[cfg(feature = "metrics")]
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, String)]) {
    use metrics::histogram;
    histogram!(name, labels).record(value);
}

/// Middleware for automatic request metrics
#[cfg(feature = "metrics")]
pub async fn metrics_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start = std::time::Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status_code = response.status().as_u16();
    
    record_request(&method, &path, status_code, duration);
    
    response
}

#[cfg(test)]
#[cfg(feature = "metrics")]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_exporter() {
        let exporter = MetricsExporter::new();
        
        // Record some metrics
        record_counter("test_counter", 1, &[("label", "value".to_string())]);
        record_gauge("test_gauge", 42.0, &[]);
        
        // Render metrics
        let output = exporter.render();
        assert!(output.contains("test_counter"));
    }
}
