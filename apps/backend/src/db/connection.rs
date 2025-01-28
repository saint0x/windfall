use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use anyhow::Result;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    // Create database file if it doesn't exist
    if let Some(path) = database_url.strip_prefix("sqlite:") {
        if !Path::new(path).exists() {
            std::fs::File::create(path)?;
        }
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./src/db/migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

pub async fn initialize_db(pool: &SqlitePool) -> Result<()> {
    // Enable foreign key support
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await?;

    Ok(())
} 