//! Database connection management

use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use std::time::Duration;
use std::path::Path;
use crate::error::Result;

pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Ensure the database directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(30))
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        // Run migrations manually
        sqlx::query(include_str!("../../migrations/001_initial_schema.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.unwrap();

        // Verify the database file was created
        assert!(db_path.exists());

        // Test migration
        db.migrate().await.unwrap();

        // Verify we can execute a simple query
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blocks")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert_eq!(result.0, 0);

        // Cleanup
        drop(db);
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_database_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("deep");
        let db_path = nested_dir.join("test.db");

        let db = Database::new(&db_path).await.unwrap();

        // Verify the directory was created
        assert!(nested_dir.exists());
        assert!(db_path.exists());

        // Cleanup
        drop(db);
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_invalid_database_url() {
        let result = Database::new(Path::new("invalid_url")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_migration_failure() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.unwrap();

        // Manually create a table that conflicts with migration
        sqlx::query("CREATE TABLE blocks (id INTEGER PRIMARY KEY)")
            .execute(db.pool())
            .await
            .unwrap();

        // Migration should fail due to conflicting table
        let result = db.migrate().await;
        assert!(result.is_err());

        // Cleanup
        drop(db);
        temp_dir.close().unwrap();
    }
}
