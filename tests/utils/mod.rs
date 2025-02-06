mod rate_limiter_test;
mod health_checker_test;

use aptos_sdk::{
    rest_client::{Client as AptosRestClient, Response, PendingTransaction},
    types::{
        account_address::AccountAddress,
        transaction::SignedTransaction,
    },
};
use backend::{
    error::Result,
    Client,
};
use mockall::mock;

mock! {
    pub AptosClient {
        pub async fn get_account_balance(&self, address: AccountAddress) -> Result<u64>;
        pub async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64>;
        pub async fn get_account_resources(&self, address: AccountAddress) -> Result<Vec<serde_json::Value>>;
        pub async fn get_account_modules(&self, address: AccountAddress) -> Result<Vec<String>>;
        pub async fn get_events(&self, key: String, start: u64, limit: u64) -> Result<Vec<serde_json::Value>>;
        pub async fn submit_transaction(&self, txn: SignedTransaction) -> Result<Response<PendingTransaction>>;
    }
}

impl Client {
    pub fn mock() -> Self {
        Self::new(MockAptosClient::new())
    }
} 