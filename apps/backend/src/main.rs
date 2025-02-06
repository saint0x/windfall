use actix_web::{web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use log::{info, error};
use std::sync::Arc;
use tokio::time::Duration;

use backend::{
    api::{routes, events::EventListener},
    db::{create_pool, schema::initialize_database},
    config::ClientConfig,
    sync::BlockchainSynchronizer,
    Client,
    AppState,
};

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting application initialization...");

    // Initialize database
    info!("Initializing database connection...");
    let pool = create_pool().await.map_err(|e| anyhow::anyhow!(e))?;
    
    info!("Running database migrations...");
    if let Err(e) = initialize_database(&pool).await {
        error!("Failed to initialize database: {}", e);
        return Err(anyhow::anyhow!(e));
    }
    info!("Database initialization completed successfully");

    // Initialize Aptos client
    info!("Initializing Aptos client...");
    let config = ClientConfig::default();
    let client = Client::new(config).await.map_err(|e| anyhow::anyhow!(e))?;
    info!("Aptos client initialized successfully");

    // Create shared application state
    let state = Arc::new(AppState { 
        db: pool.clone(),
        client: client.clone(),
    });

    // Start event listener
    let event_listener_state = state.clone();
    tokio::spawn(async move {
        let mut listener = EventListener::new(event_listener_state);
        if let Err(e) = listener.start().await {
            error!("Event listener error: {}", e);
        }
    });

    // Start blockchain synchronizer
    let sync_state = state.clone();
    tokio::spawn(async move {
        let synchronizer = BlockchainSynchronizer::new(
            (*sync_state).clone(),
            Duration::from_secs(10), // More conservative sync interval
        );
        if let Err(e) = synchronizer.start().await {
            error!("Blockchain synchronizer error: {}", e);
        }
    });

    info!("Starting server at http://127.0.0.1:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
            .service(
                web::scope("/api/v1")
                    .service(routes::funds::scope())
                    .service(routes::proposals::scope())
                    .service(routes::assets::scope())
                    .service(routes::members::scope())
                    .service(routes::messages::scope())
                    .service(routes::transactions::scope())
            )
    })
    .bind("127.0.0.1:8080").map_err(|e| anyhow::anyhow!(e))?
    .run()
    .await.map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
} 