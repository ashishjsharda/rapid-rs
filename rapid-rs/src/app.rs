use axum::{http::Method, Router};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;

#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;

/// Main application builder
pub struct App {
    router: Router,
    config: Option<AppConfig>,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            config: None,
        }
    }

    /// Auto-configure the application with sensible defaults:
    /// - Loads configuration from files and environment
    /// - Sets up structured logging with tracing
    /// - Configures CORS with permissive defaults
    /// - Adds health check endpoint
    /// - Enables Swagger UI at /docs
    pub fn auto_configure(mut self) -> Self {
        // Initialize logging
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,rapid_rs=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        tracing::info!("🚀 Initializing rapid-rs application");

        // Load configuration
        let config = AppConfig::load().expect("Failed to load configuration");
        tracing::info!("✅ Configuration loaded");

        // Setup CORS
        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_origin(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        // Add health endpoint
        let health_router = Router::new().route(
            "/health",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "status": "healthy",
                    "timestamp": chrono::Utc::now()
                }))
            }),
        );

        // Setup Swagger UI with a basic OpenAPI spec
        #[derive(OpenApi)]
        #[openapi(
            info(
                title = "rapid-rs API",
                version = "0.1.0",
                description = "API built with rapid-rs"
            ),
            paths(),
            components(schemas())
        )]
        struct ApiDoc;

        // Add Swagger UI if feature is enabled
        #[cfg(feature = "swagger-ui")]
        let swagger = SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi());

        // Build the router with middleware
        #[cfg(feature = "swagger-ui")]
        let router_with_docs = Router::new().merge(swagger).merge(health_router);

        #[cfg(not(feature = "swagger-ui"))]
        let router_with_docs = health_router;

        self.router = router_with_docs
            .merge(self.router)
            .layer(TraceLayer::new_for_http())
            .layer(cors);

        self.config = Some(config);

        tracing::info!("✅ Auto-configuration complete");
        self
    }

    /// Mount additional routes
    pub fn mount(mut self, router: Router) -> Self {
        self.router = self.router.merge(router);
        self
    }

    /// Add a route manually
    pub fn route(mut self, path: &str, method_router: axum::routing::MethodRouter) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    /// Run the application
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.unwrap_or_default();
        let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

        tracing::info!("🎯 Server starting on http://{}", addr);

        #[cfg(feature = "swagger-ui")]
        tracing::info!("📚 Swagger UI available at http://{}/docs", addr);

        #[cfg(not(feature = "swagger-ui"))]
        tracing::info!("💡 Tip: Enable 'swagger-ui' feature for API docs at /docs");

        tracing::info!("💚 Health check available at http://{}/health", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
