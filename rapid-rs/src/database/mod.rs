//! Database utilities and helpers

pub mod migrations;
pub mod backends;

pub use sqlx::{PgPool, Postgres, Transaction};
pub use migrations::{MigrationConfig, run_migrations, connect_and_migrate, ensure_database_exists};

#[cfg(feature = "db-sqlite")]
pub use backends::sqlite;

#[cfg(feature = "db-mysql")]
pub use backends::mysql;
