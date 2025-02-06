use super::*;
use actix_web::test;
use backend::api::routes::assets::{
    CreateFundWalletRequest,
    MemberInput,
    InvestmentRequest,
    WithdrawRequest,
    UpdateShareRequest,
};

#[tokio::test]
async fn test_create_fund_wallet() {
    let (state, _) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund first
    let fund = crate::test_helpers::create_test_fund(&app.app_data::<web::Data<AppState>>().unwrap().db, "Test Fund").await.unwrap();

    // Create fund wallet request
    let req = CreateFundWalletRequest {
        actuator_address: "0x123".to_string(),
        members: vec![
            MemberInput {
                address: "0x456".to_string(),
                ownership_share: 5000, // 50%
            },
            MemberInput {
                address: "0x789".to_string(),
                ownership_share: 5000, // 50%
            },
        ],
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/wallet", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_create_fund_wallet_invalid_shares() {
    let (state, _) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund first
    let fund = crate::test_helpers::create_test_fund(&app.app_data::<web::Data<AppState>>().unwrap().db, "Test Fund").await.unwrap();

    // Create fund wallet request with invalid shares (not 100%)
    let req = CreateFundWalletRequest {
        actuator_address: "0x123".to_string(),
        members: vec![
            MemberInput {
                address: "0x456".to_string(),
                ownership_share: 3000, // 30%
            },
            MemberInput {
                address: "0x789".to_string(),
                ownership_share: 3000, // 30%
            },
        ],
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/wallet", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn test_invest_in_fund() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and asset
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();

    // Create investment request
    let req = InvestmentRequest {
        target_address: "0x123".to_string(),
        amount: 1000,
        asset_id: asset.id,
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/wallet/invest", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_withdraw_profits() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and investment
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let asset = crate::test_helpers::create_test_asset(&pool).await.unwrap();
    let investment = crate::test_helpers::create_test_investment(&pool, fund.id, asset.id).await.unwrap();

    // Create withdrawal request
    let req = WithdrawRequest {
        amount: 500, // Withdraw half
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/wallet/withdraw", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_update_member_share() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and member
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let member = crate::test_helpers::create_test_member(&pool, fund.id, 5000).await.unwrap();

    // Create share update request
    let req = UpdateShareRequest {
        new_share: 6000,
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/wallet/members/{}/share", fund.id, member.member_address))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
} 