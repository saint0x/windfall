pub mod assets;
pub mod funds;
pub mod proposals;
pub mod investments;

use actix_web::{test, web, App};
use backend::{
    AppState,
    api::routes,
    db::operations,
    error::Result,
    Client,
};
use sqlx::sqlite::SqlitePool;
use crate::{setup_test_db, AptosRestClientInterface};
use mockall::Mock;

// Helper function to create test app state
pub async fn create_test_app_state() -> (AppState, SqlitePool) {
    let pool = setup_test_db().await;
    let mock_client = Client::mock();
    
    let state = AppState {
        db: pool.clone(),
        client: mock_client,
    };
    
    (state, pool)
}

// Helper function to create test app
pub async fn create_test_app(state: web::Data<AppState>) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(state)
            .configure(routes::configure)
    ).await
} 