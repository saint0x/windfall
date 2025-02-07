# Jockey Image 

Generated: 02-05-2025 at 16:40:28

## Repository Structure

```
backend
│   ├── src
│   │   ├── error.rs
│   │   ├── utils
│   │       └── rate_limiter.rs
│   │   ├── db
│   │       └── mod.rs
│       └── config.rs
│   ├── migrations
│       └── 20240101000000_initial.sql
    └── Cargo.toml
```

## File: /Users/saint/Desktop/windfall/apps/backend/Cargo.toml

```toml
[package]
name = "backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.4.0"
anyhow = "1.0.75"
serde = { version = "1.0.189", features = ["derive"] }
serde_json = "1.0.107"
sqlx = { version = "0.7.2", features = ["runtime-tokio", "sqlite", "chrono"] }
tokio = { version = "1.33.0", features = ["full"] }
chrono = { version = "0.4.31", features = ["serde"] }
dotenv = "0.15.0"
env_logger = "0.10.0"
log = "0.4.20"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
aptos-sdk = { git = "https://github.com/aptos-labs/aptos-core", branch = "devnet" }
hex = "0.4"
url = "2.4"
thiserror = "1.0"

[dev-dependencies]
mockall = "0.11"

[patch.crates-io]
merlin = { git = "https://github.com/aptos-labs/merlin" }
x25519-dalek = { git = "https://github.com/aptos-labs/x25519-dalek", branch = "zeroize_v1" }
```

## File: /Users/saint/Desktop/windfall/apps/backend/migrations/20240101000000_initial.sql

```sql
-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Create funds table
CREATE TABLE IF NOT EXISTS funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    executor_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create fund_metadata table
CREATE TABLE IF NOT EXISTS fund_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, key)
);

-- Create fund_members table
CREATE TABLE IF NOT EXISTS fund_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    member_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, member_address)
);

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    sender_address TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create positions table
CREATE TABLE IF NOT EXISTS positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    entry_price INTEGER NOT NULL,
    is_long BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    FOREIGN KEY (asset_id) REFERENCES assets(id)
);

-- Create proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    proposer_address TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    end_time DATETIME NOT NULL,
    executed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    voter_address TEXT NOT NULL,
    vote_type BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id),
    UNIQUE(proposal_id, voter_address)
);

-- Create assets table
CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(symbol)
);

-- Create balances table
CREATE TABLE IF NOT EXISTS balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    holder_address TEXT NOT NULL,
    amount INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    UNIQUE(asset_id, holder_address)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_funds_executor ON funds(executor_address);
CREATE INDEX IF NOT EXISTS idx_fund_metadata_fund_id ON fund_metadata(fund_id);
CREATE INDEX IF NOT EXISTS idx_fund_members_fund_id ON fund_members(fund_id);
CREATE INDEX IF NOT EXISTS idx_messages_fund_id ON messages(fund_id);
CREATE INDEX IF NOT EXISTS idx_positions_fund_id ON positions(fund_id);
CREATE INDEX IF NOT EXISTS idx_proposals_fund_id ON proposals(fund_id);
CREATE INDEX IF NOT EXISTS idx_votes_proposal_id ON votes(proposal_id);
CREATE INDEX IF NOT EXISTS idx_balances_asset_id ON balances(asset_id);
CREATE INDEX IF NOT EXISTS idx_balances_holder ON balances(holder_address); 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/client.rs

```rs
use aptos_sdk::{
    rest_client::{Client as AptosRestClient, PendingTransaction, Transaction},
    types::{
        account_address::AccountAddress,
        account_config::CORE_CODE_ADDRESS,
        transaction::SignedTransaction,
        chain_id::ChainId,
    },
    crypto::HashValue,
};
use std::sync::Arc;
use hex::FromHex;
use crate::{
    config::ClientConfig,
    error::{ClientError, Result},
    utils::{RateLimiter, HealthChecker},
};
use tracing::{info, warn};
use async_trait::async_trait;
use tokio::time::sleep;

#[async_trait]
pub trait ClientInterface: Send + Sync {
    async fn get_account_balance(&self, address: AccountAddress) -> Result<u64>;
    async fn get_core_account_modules(&self) -> Result<Vec<String>>;
    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64>;
    async fn submit_transaction(&self, txn: SignedTransaction) -> Result<PendingTransaction>;
    async fn get_transaction_status(&self, txn_hash: &str) -> Result<Transaction>;
    async fn wait_for_transaction(&self, pending_transaction: &PendingTransaction) -> Result<Transaction>;
    async fn get_account_events(
        &self,
        address: AccountAddress,
        event_handle: &str,
        field: &str,
        start: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<serde_json::Value>>;
    async fn get_resource<T: serde::de::DeserializeOwned + Send>(
        &self,
        address: AccountAddress,
        resource_type: &str,
    ) -> Result<T>;
    async fn simulate_transaction(&self, txn: &SignedTransaction) -> Result<Vec<serde_json::Value>>;
    async fn get_chain_id(&self) -> Result<ChainId>;
}

#[derive(Clone)]
pub struct Client {
    config: ClientConfig,
    health_checker: Arc<HealthChecker>,
    rate_limiter: Arc<RateLimiter>,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        info!("Initializing Aptos client with config: {:?}", config);
        let health_checker = Arc::new(HealthChecker::new());
        let rate_limiter = Arc::new(RateLimiter::new(
            config.rate_limit.requests_per_second,
            config.rate_limit.burst_limit,
        ));

        // Initialize health checker with all nodes
        health_checker.add_node(&config.primary_node).await?;
        for node in &config.fallback_nodes {
            if let Err(e) = health_checker.add_node(node).await {
                warn!("Failed to add fallback node {}: {}", node.url, e);
            }
        }

        Ok(Self {
            config,
            health_checker,
            rate_limiter,
        })
    }

    async fn get_client(&self) -> Result<AptosRestClient> {
        self.rate_limiter.acquire_permit().await?;
        self.health_checker.get_healthy_client().await
    }

    async fn execute_with_retry<'a, F, Fut, T>(&'a self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync + 'a,
        Fut: std::future::Future<Output = Result<T>> + Send + 'a,
        T: 'a,
    {
        let mut attempts = 0;
        let mut last_error = None;
        let mut delay = self.config.retry_config.base_delay;

        while attempts < self.config.retry_config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < self.config.retry_config.max_attempts {
                        sleep(delay).await;
                        delay = std::cmp::min(
                            delay * 2,
                            self.config.retry_config.max_delay
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or(ClientError::Internal("Retry failed with no error".to_string())))
    }

    pub async fn get_account_balance(&self, address: AccountAddress) -> Result<u64> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let account_resource = client
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
        }).await
    }

    pub async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let account = client
                .get_account(address)
                .await
                .map_err(|e| ClientError::ResourceNotFound(e.to_string()))?;
            
            Ok(account.into_inner().sequence_number)
        }).await
    }

    pub async fn submit_transaction(&self, txn: SignedTransaction) -> Result<PendingTransaction> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let response = client
                .submit(&txn)
                .await
                .map_err(|e| ClientError::TransactionError(format!("Failed to submit transaction: {}", e)))?;
            Ok(response.into_inner())
        }).await
    }

    pub async fn get_transaction_status(&self, txn_hash: &str) -> Result<Transaction> {
        let hash_str = txn_hash.strip_prefix("0x").unwrap_or(txn_hash);
        if hash_str.len() != 64 {
            return Err(ClientError::InvalidInput("Invalid transaction hash length".to_string()));
        }

        let hash_bytes = Vec::from_hex(hash_str)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;
            
        let hash = HashValue::from_slice(&hash_bytes)
            .map_err(|e| ClientError::InvalidInput(e.to_string()))?;

        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let txn_resp = client
                .get_transaction_by_hash(hash)
                .await
                .map_err(|e| ClientError::TransactionError(e.to_string()))?;

            Ok(txn_resp.into_inner())
        }).await
    }

    pub async fn wait_for_transaction(
        &self,
        pending_transaction: &PendingTransaction,
    ) -> Result<Transaction> {
        let pending_transaction = pending_transaction.clone();
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let txn = client
                .wait_for_transaction(&pending_transaction)
                .await
                .map_err(|e| ClientError::TransactionError(e.to_string()))?;

            Ok(txn.into_inner())
        }).await
    }

    pub async fn get_core_account_modules(&self) -> Result<Vec<String>> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            
            let modules = client
                .get_account_modules(CORE_CODE_ADDRESS)
                .await
                .map_err(|e| ClientError::ResourceNotFound(format!("Failed to get modules: {}", e)))?;

            let modules_inner = modules.into_inner();
            info!("Retrieved {} raw modules from core account", modules_inner.len());
            
            let result: Vec<String> = modules_inner
                .into_iter()
                .filter_map(|module| {
                    // Get the module name from the ABI
                    let parsed = module.try_parse_abi().ok()?;
                    let abi = parsed.abi.as_ref()?;
                    Some(format!("0x1::{}", abi.name))
                })
                .collect();

            if result.is_empty() {
                Err(ClientError::ResourceNotFound("No modules found in core account".to_string()))
            } else {
                Ok(result)
            }
        }).await
    }

    pub async fn get_account_events(
        &self,
        address: AccountAddress,
        event_handle: &str,
        field: &str,
        start: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<serde_json::Value>> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let events = client
                .get_account_events(address, event_handle, field, start, limit)
                .await
                .map_err(|e| ClientError::ResourceNotFound(format!("Failed to get events: {}", e)))?;

            Ok(events.into_inner().into_iter().map(|e| e.data).collect())
        }).await
    }

    pub async fn get_resource<T: serde::de::DeserializeOwned + Send>(
        &self,
        address: AccountAddress,
        resource_type: &str,
    ) -> Result<T> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let resource = client
                .get_account_resource(address, resource_type)
                .await
                .map_err(|e| ClientError::ResourceNotFound(format!("Failed to get resource: {}", e)))?;

            let data = resource
                .into_inner()
                .ok_or_else(|| ClientError::ResourceNotFound(format!("Resource {} not found", resource_type)))?;

            serde_json::from_value(data.data)
                .map_err(|e| ClientError::DeserializationError(format!("Failed to deserialize resource: {}", e)))
        }).await
    }

    pub async fn simulate_transaction(&self, txn: &SignedTransaction) -> Result<Vec<serde_json::Value>> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let response = client
                .simulate(txn)
                .await
                .map_err(|e| ClientError::TransactionError(format!("Failed to simulate transaction: {}", e)))?;

            Ok(response.into_inner().into_iter().map(|txn| serde_json::to_value(txn).unwrap()).collect())
        }).await
    }

    pub async fn get_chain_id(&self) -> Result<ChainId> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let info = client
                .get_ledger_information()
                .await
                .map_err(|e| ClientError::NetworkError(format!("Failed to get chain ID: {}", e)))?;

            Ok(ChainId::new(info.into_inner().chain_id))
        }).await
    }
}

#[async_trait]
impl ClientInterface for Client {
    async fn get_account_balance(&self, address: AccountAddress) -> Result<u64> {
        self.get_account_balance(address).await
    }

    async fn get_core_account_modules(&self) -> Result<Vec<String>> {
        self.get_core_account_modules().await
    }

    async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.get_sequence_number(address).await
    }

    async fn submit_transaction(&self, txn: SignedTransaction) -> Result<PendingTransaction> {
        self.submit_transaction(txn).await
    }

    async fn get_transaction_status(&self, txn_hash: &str) -> Result<Transaction> {
        self.get_transaction_status(txn_hash).await
    }

    async fn wait_for_transaction(&self, pending_transaction: &PendingTransaction) -> Result<Transaction> {
        self.wait_for_transaction(pending_transaction).await
    }

    async fn get_account_events(
        &self,
        address: AccountAddress,
        event_handle: &str,
        field: &str,
        start: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<serde_json::Value>> {
        self.get_account_events(address, event_handle, field, start, limit).await
    }

    async fn get_resource<T: serde::de::DeserializeOwned + Send>(
        &self,
        address: AccountAddress,
        resource_type: &str,
    ) -> Result<T> {
        self.get_resource(address, resource_type).await
    }

    async fn simulate_transaction(&self, txn: &SignedTransaction) -> Result<Vec<serde_json::Value>> {
        self.simulate_transaction(txn).await
    }

    async fn get_chain_id(&self) -> Result<ChainId> {
        self.get_chain_id().await
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/error.rs

```rs
use thiserror::Error;
use url::ParseError;
use sqlx::Error as SqlxError;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Node connection error: {0}")]
    ConnectionError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Node health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("All nodes are unhealthy")]
    NoHealthyNodes,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not authorized: {0}")]
    NotAuthorized(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Event subscription error: {0}")]
    EventSubscriptionError(String),

    #[error("Chain ID mismatch: expected {expected}, got {actual}")]
    ChainIdMismatch { expected: u8, actual: u8 },

    #[error("Account error: {0}")]
    AccountError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database transaction error: {0}")]
    TransactionIsolationError(String),
}

impl From<ParseError> for ClientError {
    fn from(error: ParseError) -> Self {
        ClientError::ConfigError(error.to_string())
    }
}

impl From<SqlxError> for ClientError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::Database(e) => ClientError::DatabaseError(e.to_string()),
            SqlxError::Io(e) => ClientError::Internal(format!("IO error: {}", e)),
            SqlxError::Tls(e) => ClientError::Internal(format!("TLS error: {}", e)),
            SqlxError::Protocol(e) => ClientError::Internal(format!("Protocol error: {}", e)),
            SqlxError::RowNotFound => ClientError::ResourceNotFound("Row not found".to_string()),
            SqlxError::TypeNotFound { type_name } => 
                ClientError::Internal(format!("Type not found: {}", type_name)),
            _ => ClientError::Internal("Unknown database error".to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, ClientError>; 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/config.rs

```rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub url: String,
    pub health_check_interval: Duration,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub primary_node: NodeConfig,
    pub fallback_nodes: Vec<NodeConfig>,
    pub retry_config: RetryConfig,
    pub rate_limit: RateLimitConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            primary_node: NodeConfig {
                url: "https://fullnode.mainnet.aptoslabs.com".to_string(),
                health_check_interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
            },
            fallback_nodes: vec![
                NodeConfig {
                    url: "https://fullnode.testnet.aptoslabs.com".to_string(),
                    health_check_interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(10),
                },
            ],
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(5),
            },
            rate_limit: RateLimitConfig {
                requests_per_second: 50,
                burst_limit: 100,
            },
        }
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/lib.rs

```rs
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
    },
};

pub mod client;
pub mod config;
pub mod error;
pub mod utils;
pub mod db;
pub mod api;

// Re-export commonly used types
pub use aptos_sdk::types as aptos_types;
pub use error::{ClientError, Result as ApiResult};
pub use client::Client;
pub use config::ClientConfig;
pub use db::{create_pool, Pool};

#[derive(Debug)]
pub struct SigningAccount {
    pub address: AccountAddress,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub client: Client,
}
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/client_tests.rs

```rs
use crate::{
    Client,
    config::{ClientConfig, NodeConfig, RetryConfig, RateLimitConfig},
    error::Result,
};
use std::time::Duration;
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload, EntryFunction},
    },
    rest_client::Event,
};
use mockall::predicate::*;
use serde_json::json;

fn get_test_config() -> ClientConfig {
    ClientConfig {
        primary_node: NodeConfig {
            url: "https://fullnode.devnet.aptoslabs.com".to_string(),
            health_check_interval: Duration::from_secs(5),
            timeout: Duration::from_secs(20), // Increased timeout for devnet
        },
        fallback_nodes: vec![],
        retry_config: RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(10),
        },
        rate_limit: RateLimitConfig {
            requests_per_second: 50, // Reduced to avoid rate limiting
            burst_limit: 100,
        },
    }
}

#[tokio::test]
async fn test_get_core_modules() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    let result = client.get_core_account_modules().await;
    match result {
        Ok(modules) => {
            println!("Successfully retrieved {} modules from devnet", modules.len());
            println!("First few modules: {:?}", &modules.iter().take(5).collect::<Vec<_>>());
            assert!(!modules.is_empty(), "Module list should not be empty");
            
            // Verify we got some expected core modules
            let has_coin_module = modules.iter().any(|name| name.contains("coin"));
            let has_account_module = modules.iter().any(|name| name.contains("account"));
            
            if !has_coin_module && !has_account_module {
                println!("All modules: {:?}", modules);
                panic!("Expected to find at least coin or account module");
            }
        }
        Err(e) => {
            panic!("Failed to get core modules: {}\nThis test requires a working devnet connection.", e);
        }
    }
}

#[tokio::test]
async fn test_get_account_balance() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    // Test with core account (which should always exist)
    let core_address = AccountAddress::from_hex_literal("0x1").unwrap();
    let result = client.get_account_balance(core_address).await;
    assert!(result.is_ok(), "Failed to get core account balance: {:?}", result);
}

#[tokio::test]
async fn test_get_events() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    // Test with a known event key from core account
    let event_key = "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>";
    let result = client.get_events(event_key, 0, 10).await;
    assert!(result.is_ok(), "Failed to get events: {:?}", result);
}

#[tokio::test]
async fn test_get_resource() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    // Test with core account's CoinStore resource
    let core_address = AccountAddress::from_hex_literal("0x1").unwrap();
    let result = client
        .get_resource::<serde_json::Value>(
            core_address,
            "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>",
        )
        .await;
    assert!(result.is_ok(), "Failed to get resource: {:?}", result);
}

#[tokio::test]
async fn test_get_sequence_number() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    // Test with core account
    let core_address = AccountAddress::from_hex_literal("0x1").unwrap();
    let result = client.get_sequence_number(core_address).await;
    assert!(result.is_ok(), "Failed to get sequence number: {:?}", result);
}

#[tokio::test]
async fn test_get_chain_id() {
    let config = get_test_config();
    let client = Client::new(config).await.unwrap();
    
    let result = client.get_chain_id().await;
    assert!(result.is_ok(), "Failed to get chain ID: {:?}", result);
    assert!(result.unwrap().id() > 0, "Chain ID should be positive");
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/lib_test.rs

```rs
use crate::*;
use std::str::FromStr;
use aptos_sdk::crypto::Uniform;
use std::time::Duration;
use crate::config::{NodeConfig, RetryConfig, RateLimitConfig};

#[tokio::test]
async fn test_client_initialization() {
    let _private_key = Ed25519PrivateKey::generate_for_testing();
    let config = config::ClientConfig {
        primary_node: NodeConfig {
            url: "https://testnet.aptoslabs.com".to_string(),
            health_check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
        },
        fallback_nodes: vec![],
        retry_config: RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
        },
        rate_limit: RateLimitConfig {
            requests_per_second: 50,
            burst_limit: 100,
        },
    };
    let result = Client::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_account_address_conversion() {
    let address_str = "0x1";
    let address = AccountAddress::from_str(address_str).unwrap();
    assert_eq!(address.to_string(), address_str);
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/mod.rs

```rs
pub mod aptos_client_tests;
pub mod client_tests;
pub mod db_tests;
pub mod models_tests;
pub mod integration_tests;
pub mod api_tests;

use sqlx::sqlite::SqlitePool;
use aptos_sdk::{
    rest_client::Client as AptosRestClient,
    crypto::ed25519::Ed25519PrivateKey,
};
use mockall::automock;
use crate::{AptosClient, SigningAccount};

// Mock AptosRestClient for testing
#[automock]
pub trait AptosRestClientInterface {
    async fn get_account_balance(&self, address: String) -> Result<u64, String>;
    async fn get_sequence_number(&self, address: String) -> Result<u64, String>;
    async fn get_account_resources(&self, address: String) -> Result<Vec<serde_json::Value>, String>;
    async fn get_account_modules(&self, address: String) -> Result<Vec<String>, String>;
    async fn get_events(&self, key: String, start: u64, limit: u64) -> Result<Vec<serde_json::Value>, String>;
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

pub fn create_test_signing_account() -> SigningAccount {
    // Create a deterministic test account
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key().to_bytes().to_vec();
    let address = private_key.public_key().derived_address();
    
    SigningAccount {
        address,
        public_key,
        private_key: private_key.to_bytes().to_vec(),
    }
}

// Helper function to create a mock AptosClient
pub fn create_mock_aptos_client() -> MockAptosRestClientInterface {
    MockAptosRestClientInterface::new()
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use chrono::{DateTime, Utc};
    
    pub fn create_test_fund(id: i64) -> crate::db::schema::Fund {
        crate::db::schema::Fund {
            id,
            name: format!("Test Fund {}", id),
            description: "Test Description".to_string(),
            executor: "0x1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn create_test_position(id: i64, fund_id: i64) -> crate::db::schema::Position {
        crate::db::schema::Position {
            id,
            fund_id,
            size: 1000,
            price: 100,
            is_long: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn create_test_proposal(id: i64, fund_id: i64) -> crate::db::schema::Proposal {
        crate::db::schema::Proposal {
            id,
            fund_id,
            proposal_type: 1,
            votes_yes: 0,
            votes_no: 0,
            end_time: Utc::now(),
            executed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/aptos_client_tests.rs

```rs
use super::*;
use crate::error::ClientError;
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload},
    },
    rest_client::PendingTransaction,
};
use mockall::predicate::*;
use std::str::FromStr;

#[tokio::test]
async fn test_get_account_balance() {
    let mut mock_client = create_mock_aptos_client();
    let test_address = "0x1234";
    let expected_balance = 1000u64;
    
    mock_client
        .expect_get_account_balance()
        .with(eq(test_address.to_string()))
        .returning(move |_| Ok(expected_balance));
    
    let address = AccountAddress::from_str(test_address).unwrap();
    let result = mock_client.get_account_balance(test_address.to_string()).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_balance);
}

#[tokio::test]
async fn test_get_sequence_number() {
    let mut mock_client = create_mock_aptos_client();
    let test_address = "0x1234";
    let expected_sequence = 5u64;
    
    mock_client
        .expect_get_sequence_number()
        .with(eq(test_address.to_string()))
        .returning(move |_| Ok(expected_sequence));
    
    let result = mock_client.get_sequence_number(test_address.to_string()).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_sequence);
}

#[tokio::test]
async fn test_get_account_balance_error() {
    let mut mock_client = create_mock_aptos_client();
    let test_address = "0x1234";
    
    mock_client
        .expect_get_account_balance()
        .with(eq(test_address.to_string()))
        .returning(|_| Err("Resource not found".to_string()));
    
    let result = mock_client.get_account_balance(test_address.to_string()).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Resource not found");
}

#[tokio::test]
async fn test_signing_account_creation() {
    let signing_account = create_test_signing_account();
    
    assert!(!signing_account.private_key.is_empty());
    assert!(!signing_account.public_key.is_empty());
    assert_ne!(signing_account.address, AccountAddress::ZERO);
}

// Test helper functions
fn create_mock_pending_transaction() -> PendingTransaction {
    PendingTransaction {
        hash: "0x123456789abcdef".to_string(),
        request: None,
    }
}

#[cfg(test)]
mod integration_style_tests {
    use super::*;
    use crate::AptosClient;
    
    #[tokio::test]
    async fn test_client_initialization() {
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let result = AptosClient::new("https://testnet.aptoslabs.com", private_key).await;
        
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(!client.public_key.to_bytes().is_empty());
    }
    
    #[tokio::test]
    async fn test_client_initialization_invalid_url() {
        let private_key = Ed25519PrivateKey::generate_for_testing();
        let result = AptosClient::new("invalid-url", private_key).await;
        
        assert!(result.is_err());
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/api_tests.rs

```rs
use actix_web::{test, web, App};
use crate::{
    api::{handlers, ApiState},
    Client,
    config::ClientConfig,
};
use std::sync::Arc;
use serde_json::json;

async fn setup_test_app() -> test::TestApp {
    let config = super::client_tests::get_test_config();
    let client = Arc::new(Client::new(config).await.unwrap());
    
    test::init_service(
        App::new()
            .app_data(web::Data::new(ApiState {
                client: client.clone(),
            }))
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/accounts")
                            .route("/{address}/balance", web::get().to(handlers::accounts::get_balance))
                            .route("/{address}/resources", web::get().to(handlers::accounts::get_resources))
                            .route("/{address}/modules", web::get().to(handlers::accounts::get_modules))
                    )
            )
    ).await
}

#[actix_web::test]
async fn test_get_balance_endpoint() {
    let app = setup_test_app().await;
    
    // Test with core account
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/0x1/balance")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("balance").is_some());
    assert_eq!(body.get("address").unwrap().as_str().unwrap(), "0x1");
}

#[actix_web::test]
async fn test_get_resources_endpoint() {
    let app = setup_test_app().await;
    
    // Test with core account
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/0x1/resources")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("resources").is_some());
    assert!(body.get("resources").unwrap().as_array().unwrap().len() > 0);
}

#[actix_web::test]
async fn test_get_modules_endpoint() {
    let app = setup_test_app().await;
    
    // Test with core account
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/0x1/modules")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("modules").is_some());
    assert!(body.get("modules").unwrap().as_array().unwrap().len() > 0);
}

#[actix_web::test]
async fn test_invalid_address() {
    let app = setup_test_app().await;
    
    // Test with invalid address
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/invalid_address/balance")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_nonexistent_account() {
    let app = setup_test_app().await;
    
    // Test with non-existent account
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/0x123456789/balance")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/models_tests.rs

```rs
use super::*;
use crate::db::schema::*;
use aptos_sdk::types::account_address::AccountAddress;
use chrono::Utc;
use std::str::FromStr;

#[test]
fn test_fund_executor_address() {
    let fund = Fund {
        id: 1,
        name: "Test Fund".to_string(),
        description: "Test Description".to_string(),
        executor: "0x1234".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let executor_address = fund.executor_address();
    assert_eq!(
        executor_address,
        AccountAddress::from_str("0x1234").unwrap()
    );
}

#[test]
fn test_fund_member_address() {
    let member = FundMember {
        id: 1,
        fund_id: 1,
        member_address: "0x5678".to_string(),
        created_at: Utc::now(),
    };
    
    let address = member.member_address();
    assert_eq!(
        address,
        AccountAddress::from_str("0x5678").unwrap()
    );
}

#[test]
fn test_message_sender_address() {
    let message = Message {
        id: 1,
        fund_id: 1,
        sender: "0x9abc".to_string(),
        content: "Test message".to_string(),
        created_at: Utc::now(),
    };
    
    let sender_address = message.sender_address();
    assert_eq!(
        sender_address,
        AccountAddress::from_str("0x9abc").unwrap()
    );
}

#[test]
fn test_vote_voter_address() {
    let vote = Vote {
        id: 1,
        proposal_id: 1,
        voter: "0xdef0".to_string(),
        vote_yes: true,
        created_at: Utc::now(),
    };
    
    let voter_address = vote.voter_address();
    assert_eq!(
        voter_address,
        AccountAddress::from_str("0xdef0").unwrap()
    );
}

#[test]
fn test_balance_holder_address() {
    let balance = Balance {
        id: 1,
        asset_id: 1,
        holder: "0x1111".to_string(),
        amount: 1000,
        created_at: Utc::now(),
        last_updated: Utc::now(),
    };
    
    let holder_address = balance.holder_address();
    assert_eq!(
        holder_address,
        AccountAddress::from_str("0x1111").unwrap()
    );
}

#[test]
#[should_panic]
fn test_invalid_address_conversion() {
    let fund = Fund {
        id: 1,
        name: "Test Fund".to_string(),
        description: "Test Description".to_string(),
        executor: "invalid_address".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // This should panic
    let _ = fund.executor_address();
}

#[test]
fn test_position_creation() {
    let position = Position {
        id: 1,
        fund_id: 1,
        size: 1000,
        price: 100,
        is_long: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(position.size, 1000);
    assert_eq!(position.price, 100);
    assert!(position.is_long);
}

#[test]
fn test_proposal_state() {
    let proposal = Proposal {
        id: 1,
        fund_id: 1,
        proposal_type: 1,
        votes_yes: 5,
        votes_no: 3,
        end_time: Utc::now(),
        executed: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(proposal.votes_yes, 5);
    assert_eq!(proposal.votes_no, 3);
    assert!(!proposal.executed);
}

#[test]
fn test_asset_creation() {
    let asset = Asset {
        id: 1,
        symbol: "BTC".to_string(),
        name: "Bitcoin".to_string(),
        decimals: 8,
        total_supply: 21_000_000,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(asset.symbol, "BTC");
    assert_eq!(asset.name, "Bitcoin");
    assert_eq!(asset.decimals, 8);
    assert_eq!(asset.total_supply, 21_000_000);
    assert!(asset.is_active);
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/db_tests.rs

```rs
use super::*;
use crate::db::{schema::*, operations};
use sqlx::sqlite::SqlitePool;
use chrono::Utc;
use std::str::FromStr;

#[tokio::test]
async fn test_create_and_get_fund() {
    let pool = setup_test_db().await;
    
    let fund_name = "Test Fund";
    let description = "Test Description";
    let executor = "0x1234";
    
    // Create fund
    let fund = operations::create_fund(
        &pool,
        fund_name.to_string(),
        description.to_string(),
        executor.to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    assert_eq!(fund.name, fund_name);
    assert_eq!(fund.description, description);
    assert_eq!(fund.executor, executor);
    
    // Get fund
    let retrieved_fund = operations::get_fund_by_id(&pool, fund.id)
        .await
        .expect("Failed to get fund");
    
    assert_eq!(retrieved_fund.id, fund.id);
    assert_eq!(retrieved_fund.name, fund_name);
}

#[tokio::test]
async fn test_create_and_get_fund_member() {
    let pool = setup_test_db().await;
    
    // Create fund first
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let member_address = "0x5678";
    
    // Add member
    let member = operations::add_fund_member(
        &pool,
        fund.id,
        member_address.to_string(),
    )
    .await
    .expect("Failed to add fund member");
    
    assert_eq!(member.fund_id, fund.id);
    assert_eq!(member.member_address, member_address);
    
    // Get members
    let members = operations::get_fund_members(&pool, fund.id)
        .await
        .expect("Failed to get fund members");
    
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].member_address, member_address);
}

#[tokio::test]
async fn test_create_and_get_message() {
    let pool = setup_test_db().await;
    
    // Create fund first
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let sender = "0x5678";
    let content = "Test message content";
    
    // Create message
    let message = operations::create_message(
        &pool,
        fund.id,
        sender.to_string(),
        content.to_string(),
    )
    .await
    .expect("Failed to create message");
    
    assert_eq!(message.fund_id, fund.id);
    assert_eq!(message.sender, sender);
    assert_eq!(message.content, content);
    
    // Get messages
    let messages = operations::get_fund_messages(&pool, fund.id, 10, None)
        .await
        .expect("Failed to get fund messages");
    
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, content);
}

#[tokio::test]
async fn test_create_and_get_position() {
    let pool = setup_test_db().await;
    
    // Create fund first
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let size = 1000;
    let price = 100;
    let is_long = true;
    
    // Create position
    let position = operations::create_position(
        &pool,
        fund.id,
        size,
        price,
        is_long,
    )
    .await
    .expect("Failed to create position");
    
    assert_eq!(position.fund_id, fund.id);
    assert_eq!(position.size, size);
    assert_eq!(position.price, price);
    assert_eq!(position.is_long, is_long);
    
    // Get position
    let retrieved_position = operations::get_position_by_id(&pool, position.id)
        .await
        .expect("Failed to get position");
    
    assert_eq!(retrieved_position.id, position.id);
    assert_eq!(retrieved_position.size, size);
}

#[tokio::test]
async fn test_create_and_get_proposal() {
    let pool = setup_test_db().await;
    
    // Create fund first
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let proposal_type = 1;
    let end_time = Utc::now();
    
    // Create proposal
    let proposal = operations::create_proposal(
        &pool,
        fund.id,
        proposal_type,
        end_time,
    )
    .await
    .expect("Failed to create proposal");
    
    assert_eq!(proposal.fund_id, fund.id);
    assert_eq!(proposal.proposal_type, proposal_type);
    assert_eq!(proposal.votes_yes, 0);
    assert_eq!(proposal.votes_no, 0);
    assert!(!proposal.executed);
    
    // Get proposal
    let retrieved_proposal = operations::get_proposal_by_id(&pool, proposal.id)
        .await
        .expect("Failed to get proposal");
    
    assert_eq!(retrieved_proposal.id, proposal.id);
    assert_eq!(retrieved_proposal.proposal_type, proposal_type);
}

#[tokio::test]
async fn test_vote_on_proposal() {
    let pool = setup_test_db().await;
    
    // Create fund and proposal
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    let proposal = operations::create_proposal(
        &pool,
        fund.id,
        1,
        Utc::now(),
    )
    .await
    .expect("Failed to create proposal");
    
    let voter = "0x5678";
    let vote_yes = true;
    
    // Cast vote
    let vote = operations::create_vote(
        &pool,
        proposal.id,
        voter.to_string(),
        vote_yes,
    )
    .await
    .expect("Failed to create vote");
    
    assert_eq!(vote.proposal_id, proposal.id);
    assert_eq!(vote.voter, voter);
    assert_eq!(vote.vote_yes, vote_yes);
    
    // Get updated proposal
    let updated_proposal = operations::get_proposal_by_id(&pool, proposal.id)
        .await
        .expect("Failed to get proposal");
    
    assert_eq!(updated_proposal.votes_yes, 1);
    assert_eq!(updated_proposal.votes_no, 0);
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/tests/integration_tests.rs

```rs
use super::*;
use crate::{
    AptosClient,
    db::{schema::*, operations},
    error::ClientError,
};
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{TransactionPayload, EntryFunction},
    },
    move_types::{
        language_storage::ModuleId,
        identifier::Identifier,
    },
};
use sqlx::sqlite::SqlitePool;
use std::str::FromStr;
use mockall::predicate::*;

async fn setup_test_environment() -> (SqlitePool, MockAptosRestClientInterface) {
    let pool = setup_test_db().await;
    let mock_client = create_mock_aptos_client();
    (pool, mock_client)
}

#[tokio::test]
async fn test_fund_creation_with_members() {
    let (pool, mut mock_client) = setup_test_environment().await;
    
    // Setup mock expectations
    let executor = "0x1234";
    let member1 = "0x5678";
    let member2 = "0x9abc";
    
    mock_client
        .expect_get_sequence_number()
        .with(eq(executor.to_string()))
        .returning(|_| Ok(0));
    
    // Create fund with members
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        executor.to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    // Add members
    let member1 = operations::add_fund_member(
        &pool,
        fund.id,
        member1.to_string(),
    )
    .await
    .expect("Failed to add member1");
    
    let member2 = operations::add_fund_member(
        &pool,
        fund.id,
        member2.to_string(),
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
    let (pool, mut mock_client) = setup_test_environment().await;
    
    // Create fund
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    // Create proposal
    let proposal = operations::create_proposal(
        &pool,
        fund.id,
        1, // proposal type
        Utc::now(),
    )
    .await
    .expect("Failed to create proposal");
    
    // Add votes
    let voters = vec![
        ("0x5678", true),
        ("0x9abc", true),
        ("0xdef0", false),
    ];
    
    for (voter, vote_yes) in voters {
        let vote = operations::create_vote(
            &pool,
            proposal.id,
            voter.to_string(),
            vote_yes,
        )
        .await
        .expect("Failed to create vote");
        
        assert_eq!(vote.voter, voter);
        assert_eq!(vote.vote_yes, vote_yes);
    }
    
    // Verify final proposal state
    let final_proposal = operations::get_proposal_by_id(&pool, proposal.id)
        .await
        .expect("Failed to get proposal");
    
    assert_eq!(final_proposal.votes_yes, 2);
    assert_eq!(final_proposal.votes_no, 1);
}

#[tokio::test]
async fn test_fund_messaging_system() {
    let (pool, _) = setup_test_environment().await;
    
    // Create fund
    let fund = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    .expect("Failed to create fund");
    
    // Add multiple messages
    let messages = vec![
        ("0x5678", "First message"),
        ("0x9abc", "Second message"),
        ("0x5678", "Third message"),
    ];
    
    for (sender, content) in messages {
        let message = operations::create_message(
            &pool,
            fund.id,
            sender.to_string(),
            content.to_string(),
        )
        .await
        .expect("Failed to create message");
        
        assert_eq!(message.sender, sender);
        assert_eq!(message.content, content);
    }
    
    // Test pagination
    let first_page = operations::get_fund_messages(&pool, fund.id, 2, None)
        .await
        .expect("Failed to get first page");
    
    assert_eq!(first_page.len(), 2);
    
    let second_page = operations::get_fund_messages(
        &pool,
        fund.id,
        2,
        Some(first_page.last().unwrap().id),
    )
    .await
    .expect("Failed to get second page");
    
    assert_eq!(second_page.len(), 1);
}

#[tokio::test]
async fn test_asset_and_balance_management() {
    let (pool, _) = setup_test_environment().await;
    
    // Create asset
    let asset = operations::create_asset(
        &pool,
        "BTC".to_string(),
        "Bitcoin".to_string(),
        8,
        21_000_000,
    )
    .await
    .expect("Failed to create asset");
    
    // Create balances for different holders
    let holders = vec![
        ("0x1234", 1000),
        ("0x5678", 2000),
        ("0x9abc", 3000),
    ];
    
    for (holder, amount) in holders {
        let balance = operations::create_or_update_balance(
            &pool,
            asset.id,
            holder.to_string(),
            amount,
        )
        .await
        .expect("Failed to create balance");
        
        assert_eq!(balance.asset_id, asset.id);
        assert_eq!(balance.holder, holder);
        assert_eq!(balance.amount, amount);
    }
    
    // Verify total supply
    let total_balance: i64 = operations::get_asset_balances(&pool, asset.id)
        .await
        .expect("Failed to get balances")
        .iter()
        .map(|b| b.amount)
        .sum();
    
    assert_eq!(total_balance, 6000);
}

#[tokio::test]
async fn test_error_handling() {
    let (pool, mut mock_client) = setup_test_environment().await;
    
    // Test invalid address error
    let result = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "invalid_address".to_string(),
    )
    .await;
    
    assert!(result.is_err());
    
    // Test non-existent fund error
    let result = operations::get_fund_by_id(&pool, 999)
        .await;
    
    assert!(result.is_err());
    
    // Test duplicate member error
    if let Ok(fund) = operations::create_fund(
        &pool,
        "Test Fund".to_string(),
        "Test Description".to_string(),
        "0x1234".to_string(),
    )
    .await
    {
        let member = "0x5678";
        
        // Add member first time
        let _ = operations::add_fund_member(
            &pool,
            fund.id,
            member.to_string(),
        )
        .await
        .expect("Failed to add member");
        
        // Try to add same member again
        let result = operations::add_fund_member(
            &pool,
            fund.id,
            member.to_string(),
        )
        .await;
        
        assert!(result.is_err());
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/utils/rate_limiter.rs

```rs
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};
use crate::error::{ClientError, Result};

pub struct RateLimiter {
    permits: Arc<Semaphore>,
    replenish_interval: Duration,
    last_replenish: Instant,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32, burst_limit: u32) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(burst_limit as usize)),
            replenish_interval: Duration::from_secs_f64(1.0 / requests_per_second as f64),
            last_replenish: Instant::now(),
        }
    }

    pub async fn acquire_permit(&self) -> Result<()> {
        if let Ok(permit) = self.permits.try_acquire() {
            let elapsed = self.last_replenish.elapsed();
            if elapsed >= self.replenish_interval {
                sleep(self.replenish_interval - elapsed).await;
            }
            permit.forget();
            Ok(())
        } else {
            Err(ClientError::RateLimitExceeded)
        }
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/utils/mod.rs

```rs
mod rate_limiter;
mod health_checker;

pub use rate_limiter::RateLimiter;
pub use health_checker::HealthChecker; 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/utils/health_checker.rs

```rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use aptos_sdk::rest_client::Client as AptosRestClient;
use url::Url;
use crate::error::{ClientError, Result};
use crate::config::NodeConfig;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(60);
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

pub struct HealthChecker {
    nodes: Arc<RwLock<HashMap<String, NodeHealth>>>,
}

struct NodeHealth {
    client: AptosRestClient,
    last_check: Instant,
    healthy: bool,
    consecutive_failures: u32,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_node(&self, config: &NodeConfig) -> Result<()> {
        let url = Url::parse(&config.url)
            .map_err(|e| ClientError::ConfigError(e.to_string()))?;
            
        let client = AptosRestClient::new(url);
        
        // Perform initial health check
        let initial_health = self.check_node_health_internal(&client).await?;
        
        let node_health = NodeHealth {
            client,
            last_check: Instant::now(),
            healthy: initial_health,
            consecutive_failures: 0,
        };

        let mut nodes = self.nodes.write().await;
        nodes.insert(config.url.clone(), node_health);
        Ok(())
    }

    #[cfg(test)]
    pub async fn check_node_health(&self, url: &str) -> Result<bool> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(url) {
            let now = Instant::now();
            
            // Check health by getting ledger information
            match node.client.get_ledger_information().await {
                Ok(_) => {
                    node.healthy = true;
                    node.consecutive_failures = 0;
                    node.last_check = now;
                    Ok(true)
                }
                Err(e) => {
                    node.consecutive_failures += 1;
                    if node.consecutive_failures >= 3 {
                        node.healthy = false;
                    }
                    node.last_check = now;
                    Err(ClientError::HealthCheckFailed(e.to_string()))
                }
            }
        } else {
            Err(ClientError::ConfigError("Node not found".to_string()))
        }
    }

    pub async fn get_healthy_client(&self) -> Result<AptosRestClient> {
        let mut nodes = self.nodes.write().await;
        
        for (_url, node) in nodes.iter_mut() {
            // Check if we need to perform a health check
            if node.last_check.elapsed() >= HEALTH_CHECK_INTERVAL {
                node.healthy = self.check_node_health_internal(&node.client).await.unwrap_or(false);
                node.last_check = Instant::now();
                
                if !node.healthy {
                    node.consecutive_failures += 1;
                } else {
                    node.consecutive_failures = 0;
                }
            }

            // Skip nodes with too many consecutive failures
            if node.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                continue;
            }

            if node.healthy {
                return Ok(node.client.clone());
            }
        }

        Err(ClientError::HealthCheckFailed("No healthy nodes available".to_string()))
    }

    async fn check_node_health_internal(&self, client: &AptosRestClient) -> Result<bool> {
        match client.get_ledger_information().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/migrations/20240321000000_initial.sql

```sql
-- Create funds table
CREATE TABLE IF NOT EXISTS funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    executor TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create fund_metadata table
CREATE TABLE IF NOT EXISTS fund_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create fund_members table
CREATE TABLE IF NOT EXISTS fund_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    member_address TEXT NOT NULL,
    joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create positions table
CREATE TABLE IF NOT EXISTS positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    price INTEGER NOT NULL,
    is_long BOOLEAN NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    proposal_type TEXT NOT NULL,
    end_time DATETIME NOT NULL,
    votes_yes INTEGER DEFAULT 0,
    votes_no INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    voter TEXT NOT NULL,
    vote_yes BOOLEAN NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id)
);

-- Create assets table
CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    total_supply INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create balances table
CREATE TABLE IF NOT EXISTS balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    holder TEXT NOT NULL,
    amount INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id)
); 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/migrations/01_initial.sql

```sql
-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Create funds table
CREATE TABLE funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    executor TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create fund_metadata table
CREATE TABLE fund_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, key)
);

-- Create fund_members table
CREATE TABLE fund_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    member_address TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, member_address)
);

-- Create messages table
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create positions table
CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    price INTEGER NOT NULL,
    is_long BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create proposals table
CREATE TABLE proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    proposal_type INTEGER NOT NULL,
    votes_yes INTEGER NOT NULL DEFAULT 0,
    votes_no INTEGER NOT NULL DEFAULT 0,
    end_time DATETIME NOT NULL,
    executed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create votes table
CREATE TABLE votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    voter TEXT NOT NULL,
    vote_yes BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id),
    UNIQUE(proposal_id, voter)
);

-- Create assets table
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    total_supply INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create balances table
CREATE TABLE balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    holder TEXT NOT NULL,
    amount INTEGER NOT NULL DEFAULT 0,
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    UNIQUE(asset_id, holder)
);

-- Create indexes
CREATE INDEX idx_messages_fund_id ON messages(fund_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_fund_members_fund_id ON fund_members(fund_id);
CREATE INDEX idx_fund_members_member ON fund_members(member_address);
CREATE INDEX idx_positions_fund_id ON positions(fund_id);
CREATE INDEX idx_proposals_fund_id ON proposals(fund_id);
CREATE INDEX idx_votes_proposal_id ON votes(proposal_id);
CREATE INDEX idx_balances_holder ON balances(holder);
CREATE INDEX idx_balances_asset ON balances(asset_id);

-- Add index for fund metadata
CREATE INDEX idx_fund_metadata_fund_id ON fund_metadata(fund_id);
CREATE INDEX idx_fund_metadata_key ON fund_metadata(key); 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/migrations/20240101000000_initial.sql

```sql
-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Create funds table
CREATE TABLE IF NOT EXISTS funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    executor TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create fund_metadata table
CREATE TABLE IF NOT EXISTS fund_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, key)
);

-- Create fund_members table
CREATE TABLE IF NOT EXISTS fund_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    member_address TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id),
    UNIQUE(fund_id, member_address)
);

-- Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create positions table
CREATE TABLE IF NOT EXISTS positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    price INTEGER NOT NULL,
    is_long BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fund_id INTEGER NOT NULL,
    proposal_type INTEGER NOT NULL,
    votes_yes INTEGER NOT NULL DEFAULT 0,
    votes_no INTEGER NOT NULL DEFAULT 0,
    end_time TIMESTAMP NOT NULL,
    executed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (fund_id) REFERENCES funds(id)
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    voter TEXT NOT NULL,
    vote_yes BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(id),
    UNIQUE(proposal_id, voter)
);

-- Create assets table
CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    total_supply INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(symbol)
);

-- Create balances table
CREATE TABLE IF NOT EXISTS balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    holder TEXT NOT NULL,
    amount INTEGER NOT NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    UNIQUE(asset_id, holder)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_funds_executor ON funds(executor);
CREATE INDEX IF NOT EXISTS idx_fund_metadata_fund_id ON fund_metadata(fund_id);
CREATE INDEX IF NOT EXISTS idx_fund_members_fund_id ON fund_members(fund_id);
CREATE INDEX IF NOT EXISTS idx_messages_fund_id ON messages(fund_id);
CREATE INDEX IF NOT EXISTS idx_positions_fund_id ON positions(fund_id);
CREATE INDEX IF NOT EXISTS idx_proposals_fund_id ON proposals(fund_id);
CREATE INDEX IF NOT EXISTS idx_votes_proposal_id ON votes(proposal_id);
CREATE INDEX IF NOT EXISTS idx_balances_asset_id ON balances(asset_id);
CREATE INDEX IF NOT EXISTS idx_balances_holder ON balances(holder); 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/types.rs

```rs
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(transparent)]
pub struct DbDateTime(pub NaiveDateTime);

impl Serialize for DbDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DateTime::<Utc>::from_naive_utc_and_offset(self.0, Utc).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DbDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dt = DateTime::<Utc>::deserialize(deserializer)?;
        Ok(Self(dt.naive_utc()))
    }
}

impl DbDateTime {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt.naive_utc())
    }

    pub fn now() -> Self {
        Self(Utc::now().naive_utc())
    }

    pub fn into_datetime(self) -> DateTime<Utc> {
        DateTime::from_naive_utc_and_offset(self.0, Utc)
    }
}

impl From<DateTime<Utc>> for DbDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::new(dt)
    }
}

impl From<DbDateTime> for DateTime<Utc> {
    fn from(dt: DbDateTime) -> Self {
        dt.into_datetime()
    }
}

impl From<NaiveDateTime> for DbDateTime {
    fn from(dt: NaiveDateTime) -> Self {
        Self(dt)
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/mod.rs

```rs
use anyhow::Result;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use log::info;

mod schema;
mod types;
pub mod operations;

// Re-export types
pub use schema::*;
pub use types::DbDateTime;
pub use operations::*;

pub type Pool = SqlitePool;

pub async fn create_pool(database_url: &str) -> Result<Pool> {
    info!("Creating database pool with URL: {}", database_url);
    
    if !database_url.contains("memory") && database_url.starts_with("sqlite:") {
        let file_part = database_url.trim_start_matches("sqlite:");
        let file_path = std::path::Path::new(file_part);
        if let Some(parent) = file_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::error!("Failed to create directory {:?}: {}", parent, e);
            } else {
                log::info!("Ensured directory {:?} exists.", parent);
            }
        }
    }
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    info!("Database pool created successfully");
    Ok(pool)
}

// Re-export sqlx for internal use
pub(crate) use sqlx; 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/schema.rs

```rs
use sqlx::sqlite::SqlitePool;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::db::types::DbDateTime;
use log::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Fund {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub executor: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundMetadata {
    pub id: i64,
    pub fund_id: i64,
    pub key: String,
    pub value: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundMember {
    pub id: i64,
    pub fund_id: i64,
    pub member_address: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub fund_id: i64,
    pub sender: String,
    pub content: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: i64,
    pub fund_id: i64,
    pub asset_id: i64,
    pub size: i64,
    pub entry_price: i64,
    pub is_long: bool,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub fund_id: i64,
    pub proposer_address: String,
    pub title: String,
    pub description: String,
    pub end_time: DbDateTime,
    pub executed: bool,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub id: i64,
    pub proposal_id: i64,
    pub voter_address: String,
    pub vote_type: bool,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub id: i64,
    pub asset_id: i64,
    pub holder_address: String,
    pub amount: i64,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundAsset {
    pub id: i64,
    pub fund_id: i64,
    pub asset_id: i64,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[allow(dead_code)]
pub async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    info!("Starting database initialization...");

    // Start a transaction
    let mut tx = pool.begin().await.context("Failed to start transaction")?;

    info!("Creating funds table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS funds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            executor TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create funds table")?;

    info!("Creating fund_members table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS fund_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            member_address TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            UNIQUE(fund_id, member_address)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create fund_members table")?;

    info!("Creating messages table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (fund_id) REFERENCES funds(id)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create messages table")?;

    info!("Creating assets table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            name TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(symbol)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create assets table")?;

    info!("Creating positions table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            size INTEGER NOT NULL,
            entry_price INTEGER NOT NULL,
            is_long BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create positions table")?;

    info!("Creating proposals table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS proposals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            proposer_address TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            end_time DATETIME NOT NULL,
            executed BOOLEAN NOT NULL DEFAULT FALSE,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (fund_id) REFERENCES funds(id)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create proposals table")?;

    info!("Creating votes table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            proposal_id INTEGER NOT NULL,
            voter_address TEXT NOT NULL,
            vote_type BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (proposal_id) REFERENCES proposals(id),
            UNIQUE(proposal_id, voter_address)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create votes table")?;

    info!("Creating balances table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS balances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            holder_address TEXT NOT NULL,
            amount INTEGER NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (asset_id) REFERENCES assets(id),
            UNIQUE(asset_id, holder_address)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create balances table")?;

    info!("Creating fund_assets table...");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS fund_assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            FOREIGN KEY (asset_id) REFERENCES assets(id),
            UNIQUE(fund_id, asset_id)
        )
        "#
    )
    .execute(&mut *tx)
    .await
    .context("Failed to create fund_assets table")?;

    // Commit the transaction
    tx.commit().await.context("Failed to commit transaction")?;

    info!("Database initialization completed successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let result = initialize_database(&pool).await;
        assert!(result.is_ok());

        // Verify tables were created
        let tables = sqlx::query!(
            r#"
            SELECT name FROM sqlite_master 
            WHERE type='table' 
            AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let expected_tables = vec![
            "assets",
            "balances",
            "fund_members",
            "funds",
            "messages",
            "positions",
            "proposals",
            "votes",
        ];

        assert_eq!(tables.len(), expected_tables.len());
        for (i, table) in tables.iter().enumerate() {
            assert_eq!(table.name.as_deref().unwrap(), expected_tables[i]);
        }
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/db/operations.rs

```rs
use crate::db::schema::*;
use crate::db::types::DbDateTime;
use anyhow::Result;
use chrono::{DateTime, Utc};
use aptos_sdk::types::account_address::AccountAddress;
use sqlx::{Pool, Sqlite};

// Fund operations
pub async fn create_fund(
    pool: &Pool<Sqlite>,
    name: String,
    description: String,
    executor: String,
) -> Result<Fund> {
    let now = DbDateTime::now();
    
    let fund = sqlx::query_as!(
        Fund,
        r#"
        INSERT INTO funds (name, description, executor, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
        name,
        description,
        executor,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create fund")?;

    Ok(fund)
}

pub async fn get_fund(
    pool: &Pool<Sqlite>,
    fund_id: i64,
) -> Result<Fund> {
    let fund = sqlx::query_as!(
        Fund,
        r#"
        SELECT * FROM funds WHERE id = ?
        "#,
        fund_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get fund")?;

    Ok(fund)
}

// Fund member operations
pub async fn add_fund_member(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    member_address: AccountAddress,
) -> Result<FundMember> {
    let now = DbDateTime::now();
    let member_str = member_address.to_string();
    
    let member = sqlx::query_as!(
        FundMember,
        r#"
        INSERT INTO fund_members (fund_id, member_address, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        member_str,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to add fund member")?;

    Ok(member)
}

pub async fn get_fund_members(
    pool: &Pool<Sqlite>,
    fund_id: i64,
) -> Result<Vec<FundMember>> {
    let members = sqlx::query_as!(
        FundMember,
        r#"
        SELECT id as "id!", fund_id as "fund_id!", member_address as "member_address!",
               created_at as "created_at!", updated_at as "updated_at!"
        FROM fund_members WHERE fund_id = ?
        "#,
        fund_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get fund members")?;

    Ok(members)
}

// Message operations
pub async fn create_message(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    sender: String,
    content: String,
) -> Result<Message> {
    let now = DbDateTime::now();
    
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (fund_id, sender, content, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        sender,
        content,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create message")?;
    
    Ok(message)
}

pub async fn get_messages(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    limit: i64,
    before_id: Option<i64>,
) -> Result<Vec<Message>> {
    let messages = match before_id {
        Some(before) => {
            sqlx::query_as!(
                Message,
                r#"
                SELECT id as "id!", fund_id as "fund_id!", sender as "sender!", 
                       content as "content!", created_at as "created_at!", updated_at as "updated_at!"
                FROM messages 
                WHERE fund_id = ? AND id < ?
                ORDER BY id DESC
                LIMIT ?
                "#,
                fund_id,
                before,
                limit
            )
            .fetch_all(pool)
            .await
        },
        None => {
            sqlx::query_as!(
                Message,
                r#"
                SELECT id as "id!", fund_id as "fund_id!", sender as "sender!", 
                       content as "content!", created_at as "created_at!", updated_at as "updated_at!"
                FROM messages 
                WHERE fund_id = ?
                ORDER BY id DESC
                LIMIT ?
                "#,
                fund_id,
                limit
            )
            .fetch_all(pool)
            .await
        }
    }.context("Failed to get messages")?;

    Ok(messages)
}

// Position operations
pub async fn create_position(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    asset_id: i64,
    size: i64,
    entry_price: i64,
    is_long: bool,
) -> Result<Position> {
    let now = DbDateTime::now();
    let position = sqlx::query_as!(
        Position,
        r#"
        INSERT INTO positions (fund_id, asset_id, size, entry_price, is_long, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        asset_id,
        size,
        entry_price,
        is_long,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create position")?;

    Ok(position)
}

pub async fn get_position_by_id(
    pool: &Pool<Sqlite>,
    position_id: i64,
) -> Result<Position> {
    let position = sqlx::query_as!(
        Position,
        r#"
        SELECT * FROM positions WHERE id = ?
        "#,
        position_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get position")?;

    Ok(position)
}

// Proposal operations
pub async fn create_proposal(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    proposer_address: AccountAddress,
    title: String,
    description: String,
    end_time: DateTime<Utc>,
) -> Result<Proposal> {
    let now = DbDateTime::now();
    let proposer_str = proposer_address.to_string();
    let end_time = DbDateTime::from(end_time);
    
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        INSERT INTO proposals (fund_id, proposer_address, title, description, end_time, executed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        proposer_str,
        title,
        description,
        end_time,
        false,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create proposal")?;

    Ok(proposal)
}

pub async fn get_proposal_by_id(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
) -> Result<Proposal> {
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        SELECT * FROM proposals WHERE id = ?
        "#,
        proposal_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get proposal")?;

    Ok(proposal)
}

pub async fn get_proposal(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
) -> Result<Proposal> {
    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        SELECT * FROM proposals WHERE id = ?
        "#,
        proposal_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to get proposal")?;

    Ok(proposal)
}

pub async fn vote_on_proposal(
    pool: &Pool<Sqlite>,
    proposal_id: i64,
    voter_address: AccountAddress,
    vote_type: bool,
) -> Result<Vote> {
    let now = DbDateTime::now();
    let voter_str = voter_address.to_string();
    
    let vote = sqlx::query_as!(
        Vote,
        r#"
        INSERT INTO votes (proposal_id, voter_address, vote_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
        proposal_id,
        voter_str,
        vote_type,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create vote")?;

    Ok(vote)
}

// Asset operations
pub async fn create_asset(
    pool: &Pool<Sqlite>,
    symbol: String,
    name: String,
    decimals: i32,
) -> Result<Asset> {
    let now = DbDateTime::now();
    
    let asset = sqlx::query_as!(
        Asset,
        r#"
        INSERT INTO assets (symbol, name, decimals, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id as "id!", symbol as "symbol!", name as "name!", 
                  decimals as "decimals!: i32", created_at as "created_at!", updated_at as "updated_at!"
        "#,
        symbol,
        name,
        decimals,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create asset")?;

    Ok(asset)
}

// Balance operations
pub async fn get_asset_balances(
    pool: &Pool<Sqlite>,
    asset_id: i64,
) -> Result<Vec<Balance>> {
    let balances = sqlx::query_as!(
        Balance,
        r#"
        SELECT id as "id!", asset_id as "asset_id!", holder_address as "holder_address!", 
               amount as "amount!", created_at as "created_at!", updated_at as "updated_at!"
        FROM balances WHERE asset_id = ?
        "#,
        asset_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get asset balances")?;

    Ok(balances)
}

#[allow(dead_code)]
pub async fn create_balance(
    pool: &Pool<Sqlite>,
    asset_id: i64,
    holder_address: AccountAddress,
    amount: i64,
) -> Result<Balance> {
    let now = DbDateTime::now();
    let holder_str = holder_address.to_string();
    
    let balance = sqlx::query_as!(
        Balance,
        r#"
        INSERT INTO balances (asset_id, holder_address, amount, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
        asset_id,
        holder_str,
        amount,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to create balance")?;

    Ok(balance)
}

pub async fn add_fund_asset(
    pool: &Pool<Sqlite>,
    fund_id: i64,
    asset_id: i64,
) -> Result<FundAsset> {
    let now = DbDateTime::now();
    
    let fund_asset = sqlx::query_as!(
        FundAsset,
        r#"
        INSERT INTO fund_assets (fund_id, asset_id, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
        fund_id,
        asset_id,
        now,
        now
    )
    .fetch_one(pool)
    .await
    .context("Failed to add fund asset")?;

    Ok(fund_asset)
}

pub async fn get_fund_assets(
    pool: &Pool<Sqlite>,
    fund_id: i64,
) -> Result<Vec<FundAsset>> {
    let assets = sqlx::query_as!(
        FundAsset,
        r#"
        SELECT * FROM fund_assets WHERE fund_id = ?
        "#,
        fund_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to get fund assets")?;

    Ok(assets)
}

pub async fn get_fund_by_id(pool: &Pool<Sqlite>, fund_id: i64) -> Result<Fund> {
    get_fund(pool, fund_id).await
}

pub async fn get_fund_messages(pool: &Pool<Sqlite>, fund_id: i64, limit: i64, before_id: Option<i64>) -> Result<Vec<Message>> {
    get_messages(pool, fund_id, limit, before_id).await
}

```

## File: /Users/saint/Desktop/windfall/apps/backend/src/main.rs

```rs
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use log::{info, error};

use backend::{
    api::routes,
    db::{create_pool, Pool, schema::initialize_database},
    config::ClientConfig,
    Client,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub client: Client,
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting application initialization...");

    // Initialize database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Initializing database connection...");
    let pool = create_pool(&database_url).await?;
    
    info!("Running database migrations...");
    if let Err(e) = initialize_database(&pool).await {
        error!("Failed to initialize database: {}", e);
        return Err(e);
    }
    info!("Database initialization completed successfully");

    // Initialize Aptos client
    info!("Initializing Aptos client...");
    let config = ClientConfig::default();
    let client = Client::new(config).await?;
    info!("Aptos client initialized successfully");

    // Create shared application state
    let state = AppState { 
        db: pool,
        client,
    };

    info!("Starting server at http://127.0.0.1:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/middleware/auth.rs

```rs
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::task::{Context, Poll};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for certain paths
        if should_skip_auth(&req) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Get the token from the Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))
                })
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))
                })
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid authorization scheme"))
            });
        }

        let token = &auth_str[7..];

        // Validate JWT token
        match validate_token(token) {
            Ok(claims) => {
                // Add claims to request extensions
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(_) => Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid token"))
            }),
        }
    }
}

fn should_skip_auth(req: &ServiceRequest) -> bool {
    let path = req.path();
    // List of paths that don't require authentication
    let public_paths = vec![
        "/api/v1/health",
        "/api/v1/auth/login",
        "/api/v1/auth/register",
    ];
    public_paths.iter().any(|&p| path.starts_with(p))
}

fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-256-bit-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/mod.rs

```rs
pub mod account;
pub mod routes;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    account::configure(cfg);
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/account.rs

```rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize};
use aptos_sdk::types::account_address::AccountAddress;
use crate::{
    client::Client,
    error::ClientError,
};

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
}

#[derive(Serialize)]
pub struct ModulesResponse {
    pub address: String,
    pub modules: Vec<String>,
}

#[derive(Serialize)]
pub struct ResourcesResponse {
    pub address: String,
    pub resources: Vec<serde_json::Value>,
}

pub async fn get_balance(
    client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    let address = match AccountAddress::from_hex_literal(&address) {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid address format"
        })),
    };

    match client.get_account_balance(address).await {
        Ok(balance) => HttpResponse::Ok().json(BalanceResponse {
            address: format!("0x{}", address),
            balance,
        }),
        Err(ClientError::ResourceNotFound(_)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Account not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_modules(
    client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    if address.as_str() == "0x1" {
        match client.get_core_account_modules().await {
            Ok(modules) => HttpResponse::Ok().json(ModulesResponse {
                address: "0x1".to_string(),
                modules,
            }),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        let _address = match AccountAddress::from_hex_literal(&address) {
            Ok(addr) => addr,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid address format"
            })),
        };

        // TODO: Implement get_account_modules for non-core addresses
        HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "Getting modules for non-core addresses is not yet implemented"
        }))
    }
}

pub async fn get_resources(
    _client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    let _address = match AccountAddress::from_hex_literal(&address) {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid address format"
        })),
    };

    // TODO: Implement get_account_resources
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Getting account resources is not yet implemented"
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/accounts")
            .route("/{address}/balance", web::get().to(get_balance))
            .route("/{address}/modules", web::get().to(get_modules))
            .route("/{address}/resources", web::get().to(get_resources))
    );
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/members.rs

```rs
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub member_address: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/members")
        .service(add_member)
}

#[post("")]
async fn add_member(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<AddMemberRequest>,
) -> impl Responder {
    let member = match req.member_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid member address"),
    };

    match operations::add_fund_member(&state.db, fund_id.into_inner(), member).await {
        Ok(member) => HttpResponse::Ok().json(member),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/mod.rs

```rs
pub mod funds;
pub mod members;
pub mod messages; 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/proposals.rs

```rs
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use chrono::{DateTime, Utc};
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateProposalRequest {
    title: String,
    description: String,
    proposer_address: String,
    end_time: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct VoteRequest {
    voter_address: String,
    vote_type: bool,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/proposals")
        .service(create_proposal)
        .service(get_proposal)
        .service(vote_on_proposal)
}

#[post("")]
async fn create_proposal(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<CreateProposalRequest>,
) -> impl Responder {
    let proposer = match req.proposer_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid proposer address"),
    };

    match operations::create_proposal(
        &state.db,
        fund_id.into_inner(),
        proposer,
        req.title.clone(),
        req.description.clone(),
        req.end_time,
    ).await {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/{proposal_id}")]
async fn get_proposal(
    state: web::Data<AppState>,
    proposal_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_proposal(&state.db, proposal_id.into_inner()).await {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[post("/{proposal_id}/votes")]
async fn vote_on_proposal(
    state: web::Data<AppState>,
    proposal_id: web::Path<i64>,
    req: web::Json<VoteRequest>,
) -> impl Responder {
    let voter = match req.voter_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voter address"),
    };

    match operations::vote_on_proposal(
        &state.db,
        proposal_id.into_inner(),
        voter,
        req.vote_type,
    ).await {
        Ok(vote) => HttpResponse::Ok().json(vote),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/assets.rs

```rs
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct AddFundAssetRequest {
    asset_id: i64,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/assets")
        .service(add_fund_asset)
        .service(get_fund_assets)
}

#[post("")]
async fn add_fund_asset(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<AddFundAssetRequest>,
) -> impl Responder {
    match operations::add_fund_asset(
        &state.db,
        fund_id.into_inner(),
        req.asset_id,
    ).await {
        Ok(asset) => HttpResponse::Ok().json(asset),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("")]
async fn get_fund_assets(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund_assets(&state.db, fund_id.into_inner()).await {
        Ok(assets) => HttpResponse::Ok().json(assets),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/funds.rs

```rs
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateFundRequest {
    pub name: String,
    pub description: String,
    pub executor_address: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds")
        .service(create_fund)
        .service(get_fund)
        .service(get_fund_members)
}

#[post("")]
async fn create_fund(
    state: web::Data<AppState>,
    req: web::Json<CreateFundRequest>,
) -> impl Responder {
    let executor = match req.executor_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid executor address"),
    };

    match operations::create_fund(&state.db, req.name.clone(), req.description.clone(), executor).await {
        Ok(fund) => HttpResponse::Ok().json(fund),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/{fund_id}")]
async fn get_fund(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund(&state.db, fund_id.into_inner()).await {
        Ok(fund) => HttpResponse::Ok().json(fund),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[get("/{fund_id}/members")]
async fn get_fund_members(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund_members(&state.db, fund_id.into_inner()).await {
        Ok(members) => HttpResponse::Ok().json(members),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/routes/messages.rs

```rs
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub sender_address: String,
}

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before_id: Option<i64>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/messages")
        .service(create_message)
        .service(get_messages)
}

#[post("")]
async fn create_message(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<CreateMessageRequest>,
) -> impl Responder {
    let sender = match req.sender_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid sender address"),
    };

    match operations::create_message(
        &state.db,
        fund_id.into_inner(),
        sender,
        req.content.clone(),
    ).await {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("")]
async fn get_messages(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    query: web::Query<GetMessagesQuery>,
) -> impl Responder {
    match operations::get_messages(
        &state.db,
        fund_id.into_inner(),
        query.limit.unwrap_or(50),
        query.before_id,
    ).await {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 
```

## File: /Users/saint/Desktop/windfall/apps/backend/src/api/handlers/accounts.rs

```rs
use actix_web::{web, HttpResponse, Result};
use aptos_sdk::types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use crate::api::ApiState;
use crate::error::ClientError;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    address: String,
    balance: u64,
}

#[derive(Debug, Serialize)]
pub struct ResourcesResponse {
    address: String,
    resources: Vec<ResourceData>,
}

#[derive(Debug, Serialize)]
pub struct ResourceData {
    type_: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ModulesResponse {
    address: String,
    modules: Vec<String>,
}

pub async fn get_balance(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let balance = state
        .client
        .get_account_balance(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(BalanceResponse {
        address,
        balance,
    }))
}

pub async fn get_resources(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let resources = state
        .client
        .get_account_resources(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(ResourcesResponse {
        address,
        resources: resources
            .into_iter()
            .map(|r| ResourceData {
                type_: r.type_,
                data: r.data,
            })
            .collect(),
    }))
}

pub async fn get_modules(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let modules = state
        .client
        .get_account_modules(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(ModulesResponse {
        address,
        modules,
    }))
} 
```



---

> 📸 Generated with [Jockey CLI](https://github.com/saint0x/jockey-cli)
