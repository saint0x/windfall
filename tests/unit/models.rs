use super::*;
use chrono::Utc;
use backend::db::schema::*;

#[test]
fn test_fund_model() {
    let now = Utc::now();
    let fund = Fund {
        id: 1,
        name: "Test Fund".to_string(),
        executor_address: "0x1234".to_string(),
        created_at: now.into(),
        updated_at: now.into(),
    };
    
    assert_eq!(fund.id, 1);
    assert_eq!(fund.name, "Test Fund");
    assert_eq!(fund.executor_address, "0x1234");
}

#[test]
fn test_fund_member_model() {
    let now = Utc::now();
    let member = FundMember {
        id: 1,
        fund_id: 1,
        member_address: "0x1234".to_string(),
        share: 5000,
        created_at: now.into(),
        updated_at: now.into(),
    };
    
    assert_eq!(member.id, 1);
    assert_eq!(member.fund_id, 1);
    assert_eq!(member.member_address, "0x1234");
    assert_eq!(member.share, 5000);
}

#[test]
fn test_proposal_model() {
    let now = Utc::now();
    let proposal = Proposal {
        id: 1,
        title: "Test Proposal".to_string(),
        description: "Test Description".to_string(),
        end_time: now.into(),
        executed: false,
        vetoed: false,
        chain_id: 0,
        synced: false,
        proposer_address: None,
        created_at: now.into(),
        updated_at: now.into(),
    };
    
    assert_eq!(proposal.id, 1);
    assert_eq!(proposal.title, "Test Proposal");
    assert_eq!(proposal.description, "Test Description");
    assert!(!proposal.executed);
    assert!(!proposal.vetoed);
}

#[test]
fn test_investment_model() {
    let now = Utc::now();
    let investment = Investment {
        id: 1,
        fund_id: 1,
        asset_id: 1,
        amount: 1000,
        withdrawn_amount: 0,
        investor_address: "0x1234".to_string(),
        created_at: now.into(),
        updated_at: now.into(),
    };
    
    assert_eq!(investment.id, 1);
    assert_eq!(investment.fund_id, 1);
    assert_eq!(investment.asset_id, 1);
    assert_eq!(investment.amount, 1000);
    assert_eq!(investment.withdrawn_amount, 0);
    assert_eq!(investment.investor_address, "0x1234");
} 