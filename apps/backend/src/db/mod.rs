use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use crate::error::Result;

pub mod operations;
pub mod schema;
pub mod types;

// Re-export types
pub use schema::*;
pub use types::DbDateTime;
pub use operations::*;

pub type Pool = SqlitePool;

pub async fn create_pool() -> Result<Pool> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    // Configure the connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Enable foreign keys for all connections
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    
    Ok(pool)
}

// Re-export sqlx for internal use
pub(crate) use sqlx; 