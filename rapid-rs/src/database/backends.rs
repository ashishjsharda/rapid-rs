//! Additional database backends

/// SQLite connection helpers
#[cfg(feature = "db-sqlite")]
pub mod sqlite {
    use sqlx::SqlitePool;
    use crate::error::ApiError;

    /// Connect to a SQLite database
    pub async fn connect(url: &str) -> Result<SqlitePool, ApiError> {
        SqlitePool::connect(url).await
            .map_err(|e| ApiError::InternalServerError(format!("SQLite connection failed: {}", e)))
    }

    /// Connect to an in-memory SQLite database (useful for testing)
    pub async fn connect_in_memory() -> Result<SqlitePool, ApiError> {
        connect("sqlite::memory:").await
    }

    /// Connect to a SQLite file database, creating it if it doesn't exist
    pub async fn connect_file(path: &str) -> Result<SqlitePool, ApiError> {
        let url = format!("sqlite://{}?mode=rwc", path);
        connect(&url).await
    }
}

/// MySQL connection helpers
#[cfg(feature = "db-mysql")]
pub mod mysql {
    use sqlx::MySqlPool;
    use crate::error::ApiError;

    /// Connect to a MySQL database
    pub async fn connect(url: &str) -> Result<MySqlPool, ApiError> {
        MySqlPool::connect(url).await
            .map_err(|e| ApiError::InternalServerError(format!("MySQL connection failed: {}", e)))
    }

    /// Connect to a MySQL database with a connection pool size
    pub async fn connect_with_pool_size(url: &str, max_connections: u32) -> Result<MySqlPool, ApiError> {
        sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("MySQL connection failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "db-sqlite")]
    #[tokio::test]
    async fn test_sqlite_in_memory() {
        use super::sqlite;
        let pool = sqlite::connect_in_memory().await.unwrap();
        let result: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(result.0, 1);
    }
}
