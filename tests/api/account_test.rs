use actix_web::{test, web, App};
use aptos_sdk::types::account_address::AccountAddress;
use backend::{
    api::account::{self, BalanceResponse, ModulesResponse, ResourcesResponse},
    error::ClientError,
};
use crate::helpers::MockTestClientInterface;

#[tokio::test]
async fn test_get_balance_success() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_account_balance()
        .with(mockall::predicate::eq(AccountAddress::from_hex_literal("0x1").unwrap()))
        .returning(|_| Ok(1000));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: BalanceResponse = test::read_body_json(resp).await;
    assert_eq!(body.address, "0x1");
    assert_eq!(body.balance, 1000);
}

#[tokio::test]
async fn test_get_balance_not_found() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_account_balance()
        .returning(|_| Err(ClientError::ResourceNotFound("Account not found".to_string())));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_modules_core_account() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_core_account_modules()
        .returning(|| Ok(vec!["0x1::coin".to_string()]));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/modules")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: ModulesResponse = test::read_body_json(resp).await;
    assert_eq!(body.address, "0x1");
    assert_eq!(body.modules, vec!["0x1::coin"]);
}

#[tokio::test]
async fn test_invalid_address() {
    let mock_client = MockTestClientInterface::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/invalid_address/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_get_resources_not_implemented() {
    let mock_client = MockTestClientInterface::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/resources")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 501); // Not Implemented
}

#[tokio::test]
async fn test_get_modules_non_core_address() {
    let mock_client = MockTestClientInterface::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x123/modules")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 501); // Not Implemented
}

#[tokio::test]
async fn test_get_balance_rate_limited() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_account_balance()
        .returning(|_| Err(ClientError::RateLimitExceeded));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429); // Too Many Requests
}

#[tokio::test]
async fn test_get_balance_retry_failed() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_account_balance()
        .returning(|_| Err(ClientError::ConnectionError("Connection failed".to_string())));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500); // Internal Server Error
}

#[tokio::test]
async fn test_get_balance_health_check_failed() {
    let mut mock_client = MockTestClientInterface::new();
    mock_client
        .expect_get_account_balance()
        .returning(|_| Err(ClientError::HealthCheckFailed("No healthy nodes".to_string())));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_client))
            .configure(account::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/accounts/0x1/balance")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 503); // Service Unavailable
} 