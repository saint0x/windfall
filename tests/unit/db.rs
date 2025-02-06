use super::*;
use chrono::Utc;
use backend::db::{operations, schema::*};

#[tokio::test]
async fn test_create_fund() {
    let pool = setup_test_db().await;
    
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    assert_eq!(fund.name, "Test Fund");
    assert_eq!(fund.executor_address, "0x1234");
}

#[tokio::test]
async fn test_create_fund_wallet() {
    let pool = setup_test_db().await;
    
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let wallet = operations::create_fund_wallet(
        &pool,
        fund.id,
        "0x5678",
    )
    .await
    .expect("Failed to create fund wallet");
    
    assert_eq!(wallet.fund_id, fund.id);
    assert_eq!(wallet.wallet_address, "0x5678");
}

#[tokio::test]
async fn test_create_proposal() {
    let pool = setup_test_db().await;
    
    let proposal = operations::create_proposal(
        &pool,
        "Test Proposal",
        "Test Description",
        Utc::now(),
    )
    .await
    .expect("Failed to create proposal");
    
    assert_eq!(proposal.title, "Test Proposal");
    assert_eq!(proposal.description, "Test Description");
    assert!(!proposal.executed);
    assert!(!proposal.vetoed);
}

#[tokio::test]
async fn test_vote_on_proposal() {
    let pool = setup_test_db().await;
    
    let proposal = operations::create_proposal(
        &pool,
        "Test Proposal",
        "Test Description",
        Utc::now(),
    )
    .await
    .expect("Failed to create proposal");
    
    let voter = AccountAddress::from_hex_literal("0x1234").unwrap();
    let vote = operations::vote_on_proposal(
        &pool,
        proposal.id,
        voter,
        true,
    )
    .await
    .expect("Failed to create vote");
    
    assert_eq!(vote.proposal_id, proposal.id);
    assert_eq!(vote.voter_address, "0x1234");
    assert!(vote.vote_type);
} 