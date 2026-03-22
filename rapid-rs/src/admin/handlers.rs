//! Admin dashboard API handlers

use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use super::{AdminConfig, get_error_count, get_request_count, get_uptime_seconds};

/// System information
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
    pub rapid_rs_version: String,
    pub features: Vec<String>,
}

/// Admin statistics
#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub uptime_seconds: u64,
    pub uptime_human: String,
    pub total_requests: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub system: SystemInfo,
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn enabled_features() -> Vec<String> {
    let mut features = vec!["rest-api".to_string(), "openapi".to_string()];

    #[cfg(feature = "auth")]
    features.push("auth".to_string());

    #[cfg(feature = "jobs")]
    features.push("jobs".to_string());

    #[cfg(feature = "websocket")]
    features.push("websocket".to_string());

    #[cfg(feature = "cache")]
    features.push("cache".to_string());

    #[cfg(feature = "cache-redis")]
    features.push("cache-redis".to_string());

    #[cfg(feature = "rate-limit")]
    features.push("rate-limit".to_string());

    #[cfg(feature = "observability")]
    features.push("observability".to_string());

    #[cfg(feature = "feature-flags")]
    features.push("feature-flags".to_string());

    #[cfg(feature = "multi-tenancy")]
    features.push("multi-tenancy".to_string());

    #[cfg(feature = "graphql")]
    features.push("graphql".to_string());

    #[cfg(feature = "notifications")]
    features.push("notifications".to_string());

    #[cfg(feature = "notifications-sms")]
    features.push("notifications-sms".to_string());

    #[cfg(feature = "file-uploads")]
    features.push("file-uploads".to_string());

    #[cfg(feature = "admin")]
    features.push("admin".to_string());

    #[cfg(feature = "db-sqlite")]
    features.push("db-sqlite".to_string());

    #[cfg(feature = "db-mysql")]
    features.push("db-mysql".to_string());

    features
}

/// GET /admin/stats - Get admin statistics
pub async fn get_stats(
    State(_config): State<Arc<AdminConfig>>,
) -> Json<AdminStats> {
    let uptime = get_uptime_seconds();
    let requests = get_request_count();
    let errors = get_error_count();
    let error_rate = if requests > 0 {
        (errors as f64 / requests as f64) * 100.0
    } else {
        0.0
    };

    Json(AdminStats {
        uptime_seconds: uptime,
        uptime_human: format_uptime(uptime),
        total_requests: requests,
        total_errors: errors,
        error_rate,
        system: SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: "stable".to_string(),
            rapid_rs_version: env!("CARGO_PKG_VERSION").to_string(),
            features: enabled_features(),
        },
    })
}

/// GET /admin/health - Detailed health check
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "uptime_seconds": get_uptime_seconds(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /admin - Serve the admin dashboard HTML
pub async fn admin_dashboard(
    State(config): State<Arc<AdminConfig>>,
) -> impl IntoResponse {
    use axum::response::Html;
    Html(super::ui::render_dashboard(&config.app_name, &config.app_version))
}

/// Create admin routes
pub fn admin_routes(config: AdminConfig) -> Router {
    let config = Arc::new(config);
    let base = config.base_path.clone();

    Router::new()
        .route(&base, get(admin_dashboard))
        .route(&format!("{}/stats", base), get(get_stats))
        .route(&format!("{}/health", base), get(health_check))
        .with_state(config)
}
