use aptos_sdk::{
    rest_client::{Client as AptosRestClient, PendingTransaction, Transaction, Response, ViewRequest},
    types::{
        account_address::AccountAddress,
        account_config::CORE_CODE_ADDRESS,
        transaction::{
            SignedTransaction, TransactionPayload, EntryFunction,
            RawTransaction, TransactionExpiration,
        },
        chain_id::ChainId,
    },
    move_types::{
        language_storage::ModuleId,
        identifier::Identifier,
    },
    crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
        HashValue,
    },
    bcs,
};
use url::Url;
use hex::FromHex;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sqlx::{Pool, Postgres};
use chrono::{DateTime, Utc};
pub use client::Client;
pub use config::ClientConfig;
pub use error::{ClientError, Result as ApiResult};

pub mod client;
pub mod config;
pub mod error;
pub mod utils;
pub mod db;

// Re-export commonly used types
pub use aptos_sdk::types as aptos_types;

#[derive(Debug)]
pub struct SigningAccount {
    pub address: AccountAddress,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub struct AptosClient {
    client: AptosRestClient,
}

impl AptosClient {
    pub async fn new(node_url: &str) -> Result<Self> {
        let url = Url::parse(node_url)?;
        let client = AptosRestClient::new(url);
        Ok(Self { client })
    }

    pub async fn get_account_balance(&self, address: AccountAddress) -> Result<u64> {
        let account_resource = self.client
            .get_account_resource(
                address,
                "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>"
            )
            .await
            .map_err(|e| ClientError::ResourceNotFound(e.to_string()))?;

        let coin_store = account_resource
            .into_inner()
            .ok_or_else(|| ClientError::ResourceNotFound("Coin store not found".to_string()))?;

        let balance = coin_store
            .data
            .get("coin")
            .and_then(|coin| coin.get("value"))
            .and_then(|value| value.as_str())
            .and_then(|value_str| value_str.parse::<u64>().ok())
            .ok_or_else(|| ClientError::ResourceNotFound("Balance not found".to_string()))?;

        Ok(balance)
    }

    pub async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        let account = self.client
            .get_account(address)
            .await
            .map_err(|e| ClientError::ResourceNotFound(e.to_string()))?;
        
        Ok(account.into_inner().sequence_number)
    }

    pub async fn get_chain_id(&self) -> Result<u8> {
        Ok(self.client
            .get_ledger_information()
            .await
            .map_err(|e| ClientError::ConnectionError(e.to_string()))?
            .into_inner()
            .chain_id)
    }

    pub async fn submit_transaction(
        &self,
        signed_txn: SignedTransaction
    ) -> Result<PendingTransaction> {
        let pending_txn = self.client
            .submit(&signed_txn)
            .await
            .map_err(|e| ClientError::TransactionError(e.to_string()))?;

        Ok(pending_txn.into_inner())
    }

    pub async fn get_transaction_status(
        &self,
        txn_hash: &str
    ) -> Result<Transaction> {
        let hash_str = txn_hash.strip_prefix("0x").unwrap_or(txn_hash);
        if hash_str.len() != 64 {
            return Err(ClientError::InvalidInput("Invalid transaction hash length".to_string()));
        }

        let hash_bytes = Vec::from_hex(hash_str)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;
            
        let hash = HashValue::from_slice(&hash_bytes)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;

        let txn_resp = self.client
            .get_transaction_by_hash(hash)
            .await
            .map_err(|e| ClientError::TransactionError(e.to_string()))?;

        Ok(txn_resp.into_inner())
    }

    pub async fn get_core_account_modules(&self) -> Result<Vec<String>> {
        let modules = self.client
            .get_account_modules(CORE_CODE_ADDRESS)
            .await
            .map_err(|e| ClientError::ResourceNotFound(e.to_string()))?;

        Ok(modules
            .into_inner()
            .into_iter()
            .map(|module| module.abi.unwrap().name.to_string())
            .collect())
    }

    pub async fn wait_for_transaction(
        &self,
        pending_transaction: &PendingTransaction
    ) -> Result<Transaction> {
        let txn = self.client
            .wait_for_transaction(pending_transaction)
            .await
            .map_err(|e| ClientError::TransactionError(e.to_string()))?;

        Ok(txn.into_inner())
    }

    pub async fn get_transaction_by_version(
        &self,
        version: u64
    ) -> Result<Transaction> {
        let txn = self.client
            .get_transaction_by_version(version)
            .await
            .map_err(|e| ClientError::TransactionError(e.to_string()))?;

        Ok(txn.into_inner())
    }

    /// Query if a user is active in the registry
    pub async fn is_user_active(&self, user_address: AccountAddress) -> Result<bool> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("registry")?),
            Identifier::new("is_active")?,
            vec![],
            vec![bcs::to_bytes(&user_address)?],
        ));

        let request = ViewRequest::new(payload);
        let response = self.client
            .view(&request, None)
            .await
            .context("Failed to query user status")?;

        let active: bool = bcs::from_bytes(&response[0])?;
        Ok(active)
    }

    /// Get position details
    pub async fn get_position_details(&self, position_id: u64) -> Result<PositionInfo> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("position")?),
            Identifier::new("get_position_info")?,
            vec![],
            vec![bcs::to_bytes(&position_id)?],
        ));

        let request = ViewRequest::new(payload.clone())
            .with_ledger_version(None);
        
        let response = self.client
            .view(&request)
            .await
            .context("Failed to query position")?;

        let position: PositionInfo = bcs::from_bytes(&response[0])?;
        Ok(position)
    }

    /// Get user's shares in a position
    pub async fn get_user_shares(&self, position_id: u64, user_address: AccountAddress) -> Result<u64> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("position")?),
            Identifier::new("get_user_shares")?,
            vec![],
            vec![bcs::to_bytes(&position_id)?, bcs::to_bytes(&user_address)?],
        ));

        let request = ViewRequest::new(payload.clone())
            .with_ledger_version(None);
        
        let response = self.client
            .view(&request)
            .await
            .context("Failed to query shares")?;

        let shares: u64 = bcs::from_bytes(&response[0])?;
        Ok(shares)
    }

    /// Get proposal details
    pub async fn get_proposal_details(&self, proposal_id: u64) -> Result<ProposalInfo> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("governance")?),
            Identifier::new("get_proposal_info")?,
            vec![],
            vec![bcs::to_bytes(&proposal_id)?],
        ));

        let request = ViewRequest::new(payload.clone())
            .with_ledger_version(None);
        
        let response = self.client
            .view(&request)
            .await
            .context("Failed to query proposal")?;

        let proposal: ProposalInfo = bcs::from_bytes(&response[0])?;
        Ok(proposal)
    }

    /// Check if a user has voted on a proposal
    pub async fn has_user_voted(&self, proposal_id: u64, user_address: AccountAddress) -> Result<bool> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("governance")?),
            Identifier::new("has_voted")?,
            vec![],
            vec![bcs::to_bytes(&proposal_id)?, bcs::to_bytes(&user_address)?],
        ));

        let request = ViewRequest::new(payload.clone())
            .with_ledger_version(None);
        
        let response = self.client
            .view(&request)
            .await
            .context("Failed to query vote status")?;

        let has_voted: bool = bcs::from_bytes(&response[0])?;
        Ok(has_voted)
    }

    /// Get fund details
    pub async fn get_fund_details(&self, fund_id: u64) -> Result<Fund> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1").expect("Invalid address"), Identifier::new("asset")?),
            Identifier::new("get_fund_info")?,
            vec![],
            vec![bcs::to_bytes(&fund_id)?],
        ));

        let request = ViewRequest::new(payload.clone())
            .with_ledger_version(None);
        
        let response = self.client
            .view(&request)
            .await
            .context("Failed to query fund")?;

        let fund: Fund = bcs::from_bytes(&response[0])?;
        Ok(fund)
    }

    /// Add message to fund chat (off-chain)
    pub async fn add_message(
        &self,
        pool: &Pool<Postgres>,
        fund_id: i64,
        sender: AccountAddress,
        content: String,
    ) -> Result<Message> {
        // First verify sender is a member of the fund
        let fund = self.get_fund_details(fund_id as u64).await?;
        if !fund.members.contains(&sender) {
            return Err(ClientError::NotAuthorized("Sender is not a fund member".to_string()));
        }

        // Insert message into database
        let message = sqlx::query_as!(
            Message,
            r#"
            INSERT INTO messages (fund_id, sender, content, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, fund_id, sender, content, created_at
            "#,
            fund_id,
            sender.to_string(),
            content,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| ClientError::DatabaseError(e.to_string()))?;

        Ok(message)
    }

    /// Get fund messages (off-chain)
    pub async fn get_fund_messages(
        &self,
        pool: &Pool<Postgres>,
        fund_id: i64,
        limit: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<Message>> {
        let messages = if let Some(before) = before_id {
            sqlx::query_as!(
                Message,
                r#"
                SELECT id, fund_id, sender, content, created_at
                FROM messages
                WHERE fund_id = $1 AND id < $2
                ORDER BY id DESC
                LIMIT $3
                "#,
                fund_id,
                before,
                limit
            )
        } else {
            sqlx::query_as!(
                Message,
                r#"
                SELECT id, fund_id, sender, content, created_at
                FROM messages
                WHERE fund_id = $1
                ORDER BY id DESC
                LIMIT $2
                "#,
                fund_id,
                limit
            )
        }
        .fetch_all(pool)
        .await
        .map_err(|e| ClientError::DatabaseError(e.to_string()))?;

        Ok(messages)
    }

    /// Create a new fund
    pub async fn create_fund(
        &self,
        fund_type: u8,
        name: String,
        executor: AccountAddress,
        initial_members: Vec<AccountAddress>,
        admin: &SigningAccount,
    ) -> Result<PendingTransaction> {
        let payload = TransactionPayload::EntryFunction(EntryFunction::new(
            ModuleId::new(AccountAddress::from_hex_literal("0x1"), Identifier::new("asset")?),
            Identifier::new("create_fund")?,
            vec![],
            vec![
                bcs::to_bytes(&fund_type)?,
                bcs::to_bytes(&name)?,
                bcs::to_bytes(&executor)?,
                bcs::to_bytes(&initial_members)?,
            ],
        ));

        let signed_txn = self.create_signed_transaction(admin, payload).await?;
        self.submit_transaction(signed_txn).await
    }

    async fn create_signed_transaction(
        &self,
        account: &SigningAccount,
        payload: TransactionPayload,
    ) -> Result<SignedTransaction> {
        let chain_id = self.get_chain_id().await?;
        let sequence_number = self.get_sequence_number(account.address).await?;
        
        let expiration_timestamp = Utc::now().timestamp() as u64 + 600; // 10 minutes
        
        let transaction = RawTransaction::new(
            account.address,
            sequence_number,
            payload,
            10_000, // max gas amount
            100,    // gas unit price
            expiration_timestamp,
            ChainId::new(chain_id),
        );

        let signing_key = Ed25519PrivateKey::try_from(account.private_key.as_slice())
            .map_err(|e| ClientError::InvalidInput(format!("Invalid private key: {}", e)))?;
        
        let public_key = Ed25519PublicKey::from(&signing_key);
        let signature = signing_key.sign(&transaction.clone().into_bytes())?;
            
        Ok(SignedTransaction::new_with_authenticator(
            transaction,
            Ed25519Signature::new(signature.to_bytes().to_vec(), public_key),
        ))
    }
}

/// Position information returned by view function
#[derive(Debug, Serialize, Deserialize)]
pub struct PositionInfo {
    pub size: u64,
    pub price: u64,
    pub is_long: bool,
}

/// Proposal information returned by view function
#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalInfo {
    pub proposal_type: u8,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub end_time: u64,
    pub executed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fund {
    pub fund_type: u8,
    pub name: String,
    pub executor: AccountAddress,
    pub members: Vec<AccountAddress>,
    pub created_at: u64,
}

#[derive(Debug)]
pub struct Message {
    pub id: i64,
    pub fund_id: i64,
    pub sender: AccountAddress,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVNET_URL: &str = "https://fullnode.devnet.aptoslabs.com";
    const TEST_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn test_client_creation() {
        let client = AptosClient::new(DEVNET_URL).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_chain_id() {
        let client = AptosClient::new(DEVNET_URL).await.unwrap();
        let chain_id = client.get_chain_id().await;
        assert!(chain_id.is_ok());
    }

    #[tokio::test]
    async fn test_core_account_exists() {
        let client = AptosClient::new(DEVNET_URL).await.unwrap();
        let seq_num = client.get_sequence_number(CORE_CODE_ADDRESS).await;
        assert!(seq_num.is_ok());
    }

    #[tokio::test]
    async fn test_core_modules() {
        let client = AptosClient::new(DEVNET_URL).await.unwrap();
        let modules = client.get_core_account_modules().await;
        assert!(modules.is_ok());
        assert!(!modules.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_transaction_hash_parsing() {
        let client = AptosClient::new(DEVNET_URL).await.unwrap();
        let result = client.get_transaction_status(TEST_HASH).await;
        // Note: This will likely fail as the test hash doesn't exist,
        // but it tests the hash parsing logic
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_by_version() {
        let client = AptosClient::new(DEVNET_URL).await.unwrap();
        let result = client.get_transaction_by_version(0).await;
        assert!(result.is_ok());
    }
}
