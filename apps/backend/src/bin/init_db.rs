use backend::db;
use backend::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let _pool = db::create_pool().await?;
    println!("Database initialized successfully at data/windfall.db");
    Ok(())
} 