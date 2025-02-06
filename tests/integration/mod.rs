use backend::{
    Client,
    db::{operations, schema::*},
    error::Result,
};
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload},
    },
};
use crate::{setup_test_db, test_helpers::create_mock_pending_transaction};
use sqlx::sqlite::SqlitePool;

#[tokio::test]
async fn test_fund_creation_with_members() {
    let pool = setup_test_db().await;
    let mock_client = Client::mock();
    
    // Create fund
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    // Add members
    let member1 = operations::create_fund_member(
        &pool,
        fund.id,
        "0x5678",
        5000,
    )
    .await
    .expect("Failed to add member1");
    
    let member2 = operations::create_fund_member(
        &pool,
        fund.id,
        "0x9abc",
        5000,
    )
    .await
    .expect("Failed to add member2");
    
    // Verify fund and members
    let members = operations::get_fund_members(&pool, fund.id)
        .await
        .expect("Failed to get members");
    
    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|m| m.member_address == member1.member_address));
    assert!(members.iter().any(|m| m.member_address == member2.member_address));
}

#[tokio::test]
async fn test_proposal_lifecycle() {
    let pool = setup_test_db().await;
    let mock_client = Client::mock();
    
    // Create fund
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    // Create proposal
    let proposal = operations::create_proposal(
        &pool,
        "Test Proposal",
        "Test Description",
        chrono::Utc::now(),
    )
    .await
    .expect("Failed to create proposal");
    
    // Add votes
    let voters = vec![
        ("0x5678", true),
        ("0x9abc", true),
        ("0xdef0", false),
    ];
    
    for (voter, vote_type) in voters {
        let vote = operations::vote_on_proposal(
            &pool,
            proposal.id,
            AccountAddress::from_hex_literal(voter).unwrap(),
            vote_type,
        )
        .await
        .expect("Failed to create vote");
        
        assert_eq!(vote.voter_address, voter);
        assert_eq!(vote.vote_type, vote_type);
    }
    
    // Verify final proposal state
    let final_proposal = operations::get_proposal_by_id(&pool, proposal.id)
        .await
        .expect("Failed to get proposal");
    
    assert!(!final_proposal.executed);
    assert!(!final_proposal.vetoed);
} 