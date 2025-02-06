use super::*;
use actix_web::test;
use backend::api::routes::funds::{
    CreateFundRequest,
    AddMemberRequest,
    RemoveMemberRequest,
};

#[tokio::test]
async fn test_create_fund() {
    let (state, _) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    let req = CreateFundRequest {
        name: "Test Fund".to_string(),
        executor_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/funds")
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_get_fund() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund first
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/funds/{}", fund.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_add_member() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund first
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    let req = AddMemberRequest {
        member_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/members", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_remove_member() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and member
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let member = crate::test_helpers::create_test_member(&pool, fund.id, 5000).await.unwrap();

    let req = RemoveMemberRequest {
        member_address: member.member_address.clone(),
    };

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/funds/{}/members", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_get_fund_not_found() {
    let (state, _) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/funds/999")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_create_fund_duplicate_name() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create first fund
    let _ = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    // Try to create another fund with same name
    let req = CreateFundRequest {
        name: "Test Fund".to_string(),
        executor_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/funds")
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad request - duplicate name
}

#[tokio::test]
async fn test_remove_nonexistent_member() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    let req = RemoveMemberRequest {
        member_address: "0x999".to_string(),
    };

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/funds/{}/members", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404); // Not found
} 