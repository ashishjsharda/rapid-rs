//! Prometheus metrics exporter

#[cfg(feature = "observability")]
use axum::{routing::get, Router};
#[cfg(feature = "observability")]
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
#[cfg(feature = "observability")]
use std::time::Duration;

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub endpoint: String,
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
#[cfg(feature = "observability")]
pub struct MetricsExporter {
    handle: PrometheusHandle,
    config: MetricsConfig,
}

#[cfg(feature = "observability")]
impl MetricsExporter {
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }
    
    pub fn with_config(config: MetricsConfig) -> Self {
        let builder = PrometheusBuilder::new();
        
        let builder = builder
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".to_string()),
                &config.latency_buckets,
            )
            .unwrap();
        
        let handle = builder
            .install_recorder()
            .expect("Failed to install Prometheus recorder");
        
        tracing::info!("Metrics exporter initialized at {}", config.endpoint);
        
        Self { handle, config }
    }
    
    pub fn render(&self) -> String {
        self.handle.render()
    }
    
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

#[cfg(feature = "observability")]
impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Record an HTTP request
#[cfg(feature = "observability")]
pub fn record_request(method: &str, path: &str, status_code: u16, duration: Duration) {
    use metrics::{counter, histogram};
    
    // Correct syntax for metrics 0.22
    counter!("http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status_code.to_string()
    ).increment(1);
    
    histogram!("http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status_code.to_string()
    ).record(duration.as_secs_f64());
    
    if status_code >= 500 {
        counter!("http_requests_errors_total",
            "method" => method.to_string(),
            "path" => path.to_string(),
            "status" => status_code.to_string()
        ).increment(1);
    }
}

#[cfg(feature = "observability")]
pub fn record_counter(name: &'static str, value: u64, labels: &[(&'static str, String)]) {
    use metrics::counter;
    
    if labels.is_empty() {
        counter!(name).increment(value);
    } else {
        // Build labels dynamically
        let mut c = counter!(name);
        for (key, val) in labels {
            c = counter!(name, *key => val.clone());
        }
        c.increment(value);
    }
}

#[cfg(feature = "observability")]
pub fn record_gauge(name: &'static str, value: f64, labels: &[(&'static str, String)]) {
    use metrics::gauge;
    
    if labels.is_empty() {
        gauge!(name).set(value);
    } else {
        let mut g = gauge!(name);
        for (key, val) in labels {
            g = gauge!(name, *key => val.clone());
        }
        g.set(value);
    }
}

#[cfg(feature = "observability")]
pub fn record_histogram(name: &'static str, value: f64, labels: &[(&'static str, String)]) {
    use metrics::histogram;
    
    if labels.is_empty() {
        histogram!(name).record(value);
    } else {
        let mut h = histogram!(name);
        for (key, val) in labels {
            h = histogram!(name, *key => val.clone());
        }
        h.record(value);
    }
}

#[cfg(feature = "observability")]
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
#[cfg(feature = "observability")]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_exporter() {
        let exporter = MetricsExporter::new();
        record_counter("test_counter", 1, &[("label", "value".to_string())]);
        record_gauge("test_gauge", 42.0, &[]);
        let output = exporter.render();
        assert!(output.contains("test_counter"));
    }
}