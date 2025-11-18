use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl AppConfig {
    /// Load configuration from files and environment variables
    /// 
    /// Loads in this order:
    /// 1. config/default.toml (if exists)
    /// 2. config/local.toml (if exists)
    /// 3. Environment variables (prefixed with APP_)
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("database.url", "postgres://localhost/rapid_rs")?
            .set_default("database.max_connections", 10)?
            // Try to load config files (won't fail if they don't exist)
            .add_source(
                config::File::with_name("config/default")
                    .required(false)
            )
            .add_source(
                config::File::with_name("config/local")
                    .required(false)
            )
            // Environment variables override everything
            // APP_SERVER__PORT=8080 -> server.port
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
            )
            .build()?;

        config.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "postgres://localhost/rapid_rs".to_string(),
                max_connections: 10,
            },
        }
    }
}
