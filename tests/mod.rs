pub mod api;
pub mod integration;
pub mod unit;

use sqlx::sqlite::SqlitePool;
use aptos_sdk::{
    rest_client::{Client as AptosRestClient, Response, PendingTransaction},
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload},
        HashValue,
    },
    crypto::{ed25519::Ed25519PrivateKey, Uniform},
};
use mockall::automock;
use backend::{
    AppState,
    db::{operations, schema::*},
    error::Result,
    Client,
};

// Mock interfaces
#[automock]
pub trait AptosRestClientInterface {
    async fn get_account_balance(&self, address: AccountAddress) -> Result<u64>;
    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64>;
    async fn get_account_resources(&self, address: AccountAddress) -> Result<Vec<serde_json::Value>>;
    async fn get_account_modules(&self, address: AccountAddress) -> Result<Vec<String>>;
    async fn get_events(&self, key: String, start: u64, limit: u64) -> Result<Vec<serde_json::Value>>;
    async fn submit_transaction(&self, txn: SignedTransaction) -> Result<Response<PendingTransaction>>;
}

// Test utilities
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

// Helper functions for creating test data
pub mod test_helpers {
    use super::*;
    use chrono::Utc;

    pub async fn create_test_fund(pool: &SqlitePool, name: &str) -> Result<Fund> {
        operations::create_fund(
            pool,
            name.to_string(),
            "0x1".to_string(),
        ).await
    }

    pub async fn create_test_fund_wallet(pool: &SqlitePool, fund_id: i64) -> Result<FundWallet> {
        operations::create_fund_wallet(
            pool,
            fund_id,
            "0x2",
        ).await
    }

    pub async fn create_test_member(pool: &SqlitePool, fund_id: i64, share: i64) -> Result<FundMember> {
        operations::create_fund_member(
            pool,
            fund_id,
            "0x3",
            share,
        ).await
    }

    pub async fn create_test_proposal(pool: &SqlitePool, title: &str) -> Result<Proposal> {
        operations::create_proposal(
            pool,
            title,
            "Test description",
            Utc::now(),
        ).await
    }

    pub async fn create_test_asset(pool: &SqlitePool) -> Result<Asset> {
        operations::create_asset(
            pool,
            "TEST".to_string(),
            "Test Asset".to_string(),
            8,
        ).await
    }

    pub async fn create_test_investment(
        pool: &SqlitePool,
        fund_id: i64,
        asset_id: i64,
    ) -> Result<Investment> {
        operations::create_investment(
            pool,
            fund_id,
            asset_id,
            1000,
            "0x4",
        ).await
    }

    pub fn create_mock_pending_transaction() -> PendingTransaction {
        PendingTransaction {
            hash: HashValue::zero(),
            request: UserTransactionRequest::default(),
        }
    }
} 