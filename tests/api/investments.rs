use super::*;
use actix_web::test;
use backend::api::routes::investments::{
    CreateInvestmentRequest,
    WithdrawInvestmentRequest,
};

#[tokio::test]
async fn test_create_investment() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and asset
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();

    let req = CreateInvestmentRequest {
        asset_id: asset.id,
        amount: 1000,
        investor_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/investments", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_withdraw_investment() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund, asset and investment
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();
    let investment = crate::test_helpers::create_test_investment(&pool, fund.id, asset.id).await.unwrap();

    let req = WithdrawInvestmentRequest {
        amount: 500,
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/investments/{}/withdraw", fund.id, investment.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_get_investment() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund, asset and investment
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();
    let investment = crate::test_helpers::create_test_investment(&pool, fund.id, asset.id).await.unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/funds/{}/investments/{}", fund.id, investment.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_withdraw_more_than_invested() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund, asset and investment
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();
    let investment = crate::test_helpers::create_test_investment(&pool, fund.id, asset.id).await.unwrap();

    let req = WithdrawInvestmentRequest {
        amount: investment.amount + 1, // Try to withdraw more than invested
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/investments/{}/withdraw", fund.id, investment.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad request
}

#[tokio::test]
async fn test_get_nonexistent_investment() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/funds/{}/investments/999", fund.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404); // Not found
}

#[tokio::test]
async fn test_create_investment_nonexistent_asset() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    let req = CreateInvestmentRequest {
        asset_id: 999, // Nonexistent asset
        amount: 1000,
        investor_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/investments", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404); // Not found
} 