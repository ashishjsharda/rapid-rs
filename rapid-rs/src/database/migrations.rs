//! Database migration management
//!
//! Provides automatic migration running and management using sqlx's built-in
//! migration system.

use sqlx::{migrate::MigrateDatabase, Postgres, PgPool};
use std::path::Path;

use crate::error::ApiError;

/// Migration configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Path to migrations directory (default: "./migrations")
    pub migrations_path: String,
    
    /// Whether to run migrations automatically on startup
    pub auto_migrate: bool,
    
    /// Whether to create the database if it doesn't exist
    pub create_db_if_missing: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            migrations_path: "./migrations".to_string(),
            auto_migrate: true,
            create_db_if_missing: true,
        }
    }
}

impl MigrationConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn migrations_path(mut self, path: impl Into<String>) -> Self {
        self.migrations_path = path.into();
        self
    }
    
    pub fn auto_migrate(mut self, auto: bool) -> Self {
        self.auto_migrate = auto;
        self
    }
    
    pub fn create_db_if_missing(mut self, create: bool) -> Self {
        self.create_db_if_missing = create;
        self
    }
}

/// Run pending migrations
pub async fn run_migrations(
    pool: &PgPool,
    config: &MigrationConfig,
) -> Result<(), ApiError> {
    let migrations_path = Path::new(&config.migrations_path);
    
    if !migrations_path.exists() {
        tracing::warn!(
            "Migrations directory '{}' does not exist, skipping migrations",
            config.migrations_path
        );
        return Ok(());
    }
    
    tracing::info!("Running database migrations from '{}'", config.migrations_path);
    
    let migrator = sqlx::migrate::Migrator::new(migrations_path)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to load migrations: {}", e)))?;
    
    migrator
        .run(pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Migration failed: {}", e)))?;
    
    tracing::info!("✅ Database migrations completed successfully");
    
    Ok(())
}

/// Create database if it doesn't exist
pub async fn ensure_database_exists(database_url: &str) -> Result<(), ApiError> {
    if !Postgres::database_exists(database_url)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to check database: {}", e)))?
    {
        tracing::info!("Database does not exist, creating...");
        
        Postgres::create_database(database_url)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create database: {}", e)))?;
        
        tracing::info!("✅ Database created successfully");
    }
    
    Ok(())
}

/// Connect to database and optionally run migrations
pub async fn connect_and_migrate(
    database_url: &str,
    config: MigrationConfig,
) -> Result<PgPool, ApiError> {
    // Create database if needed
    if config.create_db_if_missing {
        ensure_database_exists(database_url).await?;
    }
    
    // Connect to database
    tracing::info!("Connecting to database...");
    let pool = PgPool::connect(database_url)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to connect to database: {}", e)))?;
    
    tracing::info!("✅ Connected to database");
    
    // Run migrations if configured
    if config.auto_migrate {
        run_migrations(&pool, &config).await?;
    }
    
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migration_config_builder() {
        let config = MigrationConfig::new()
            .migrations_path("./custom/migrations")
            .auto_migrate(false)
            .create_db_if_missing(false);
        
        assert_eq!(config.migrations_path, "./custom/migrations");
        assert!(!config.auto_migrate);
        assert!(!config.create_db_if_missing);
    }
}
