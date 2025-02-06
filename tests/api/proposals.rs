use super::*;
use actix_web::test;
use backend::api::routes::proposals::{
    CreateProposalRequest,
    VoteRequest,
    EmergencyVetoRequest,
};
use chrono::{Utc, Duration};

#[tokio::test]
async fn test_create_proposal() {
    let (state, _) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund first
    let fund = crate::test_helpers::create_test_fund(&app.app_data::<web::Data<AppState>>().unwrap().db, "Test Fund").await.unwrap();

    // Create proposal request
    let req = CreateProposalRequest {
        title: "Test Proposal".to_string(),
        description: "Test Description".to_string(),
        proposer_address: "0x123".to_string(),
        end_time: Utc::now() + Duration::days(7),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_vote_on_proposal() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and proposal
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let proposal = crate::test_helpers::create_test_proposal(&pool, "Test Proposal").await.unwrap();

    // Create vote request
    let req = VoteRequest {
        voter_address: "0x123".to_string(),
        vote_type: true, // Yes vote
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals/{}/votes", fund.id, proposal.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_emergency_veto() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and proposal
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let proposal = crate::test_helpers::create_test_proposal(&pool, "Test Proposal").await.unwrap();

    // Create veto request
    let req = EmergencyVetoRequest {
        initiator_address: "0x123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals/{}/emergency-veto", fund.id, proposal.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify proposal is vetoed
    let vetoed_proposal = operations::get_proposal_by_id(&pool, proposal.id).await.unwrap();
    assert!(vetoed_proposal.vetoed);
}

#[tokio::test]
async fn test_get_proposal() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and proposal
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let proposal = crate::test_helpers::create_test_proposal(&pool, "Test Proposal").await.unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/funds/{}/proposals/{}", fund.id, proposal.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_vote_on_nonexistent_proposal() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();

    // Try to vote on nonexistent proposal
    let req = VoteRequest {
        voter_address: "0x123".to_string(),
        vote_type: true,
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals/999/votes", fund.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_double_vote() {
    let (state, pool) = create_test_app_state().await;
    let app = create_test_app(web::Data::new(state)).await;

    // Create test fund and proposal
    let fund = crate::test_helpers::create_test_fund(&pool, "Test Fund").await.unwrap();
    let proposal = crate::test_helpers::create_test_proposal(&pool, "Test Proposal").await.unwrap();

    // First vote
    let req = VoteRequest {
        voter_address: "0x123".to_string(),
        vote_type: true,
    };

    let first_vote = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals/{}/votes", fund.id, proposal.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, first_vote).await;
    assert!(resp.status().is_success());

    // Try to vote again
    let second_vote = test::TestRequest::post()
        .uri(&format!("/api/v1/funds/{}/proposals/{}/votes", fund.id, proposal.id))
        .set_json(&req)
        .to_request();

    let resp = test::call_service(&app, second_vote).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad request - already voted
} 