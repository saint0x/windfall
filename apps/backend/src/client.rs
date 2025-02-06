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
    error::{AppError, Result},
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
                            delay + self.config.retry_config.base_delay,
                            self.config.retry_config.max_delay
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::Internal("Retry failed with no error".to_string())))
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
                .map_err(|e| AppError::internal(format!("Failed to get account resource: {}", e)))?;

            let coin_store = account_resource
                .into_inner()
                .ok_or_else(|| AppError::internal("Coin store not found".to_string()))?;

            let balance = coin_store
                .data
                .get("coin")
                .and_then(|coin| coin.get("value"))
                .and_then(|value| value.as_str())
                .and_then(|value_str| value_str.parse::<u64>().ok())
                .ok_or_else(|| AppError::internal("Balance not found".to_string()))?;

            Ok(balance)
        }).await
    }

    pub async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let account = client
                .get_account(address)
                .await
                .map_err(|e| AppError::internal(format!("Failed to get sequence number: {}", e)))?;
            
            Ok(account.into_inner().sequence_number)
        }).await
    }

    pub async fn submit_transaction(&self, txn: SignedTransaction) -> Result<PendingTransaction> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let response = client
                .submit(&txn)
                .await
                .map_err(|e| AppError::transaction_error(&format!("Failed to submit transaction: {}", e)))?;
            Ok(response.into_inner())
        }).await
    }

    pub async fn get_transaction_status(&self, txn_hash: &str) -> Result<Transaction> {
        let hash_str = txn_hash.strip_prefix("0x").unwrap_or(txn_hash);
        if hash_str.len() != 64 {
            return Err(AppError::InvalidInput("Invalid transaction hash length".to_string()));
        }

        let hash_bytes = Vec::from_hex(hash_str)
            .map_err(|e| AppError::InvalidInput(e.to_string()))?;
            
        let hash = HashValue::from_slice(&hash_bytes)
            .map_err(|e| AppError::InvalidInput(e.to_string()))?;

        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let txn_resp = client
                .get_transaction_by_hash(hash)
                .await
                .map_err(|e| AppError::transaction_error(&e.to_string()))?;

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
                .map_err(|e| AppError::transaction_error(&e.to_string()))?;

            Ok(txn.into_inner())
        }).await
    }

    pub async fn get_core_account_modules(&self) -> Result<Vec<String>> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            
            let modules = client
                .get_account_modules(CORE_CODE_ADDRESS)
                .await
                .map_err(|e| AppError::internal(format!("Failed to get modules: {}", e)))?;

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
                Err(AppError::internal("No modules found in core account".to_string()))
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
                .map_err(|e| AppError::internal(format!("Failed to get events: {}", e)))?;

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
                .map_err(|e| AppError::internal(format!("Failed to get resource: {}", e)))?;

            let data = resource
                .into_inner()
                .ok_or_else(|| AppError::internal(format!("Resource {} not found", resource_type)))?;

            serde_json::from_value(data.data)
                .map_err(|e| AppError::deserialization_error(&format!("Failed to deserialize resource: {}", e)))
        }).await
    }

    pub async fn simulate_transaction(&self, txn: &SignedTransaction) -> Result<Vec<serde_json::Value>> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let response = client
                .simulate(txn)
                .await
                .map_err(|e| AppError::transaction_error(&format!("Failed to simulate transaction: {}", e)))?;

            Ok(response.into_inner().into_iter().map(|txn| serde_json::to_value(txn).unwrap()).collect())
        }).await
    }

    pub async fn get_chain_id(&self) -> Result<ChainId> {
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let info = client
                .get_ledger_information()
                .await
                .map_err(|e| AppError::network_error(&format!("Failed to get chain ID: {}", e)))?;

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
