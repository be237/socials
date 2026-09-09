use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &Path) -> Result<Self, sqlx::Error> {
        let database_url = format!("sqlite:{}?mode=rwc", database_path.display());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // Run migrations
        sqlx::query(include_str!("../../../migrations/001_initial_schema.sql"))
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
