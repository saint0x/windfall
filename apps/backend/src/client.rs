use aptos_sdk::{
    rest_client::{Client as AptosRestClient, PendingTransaction, Transaction},
    types::{
        account_address::AccountAddress,
        account_config::CORE_CODE_ADDRESS,
        transaction::SignedTransaction,
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
use tokio::time::sleep;

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

    pub async fn submit_transaction(
        &self,
        signed_txn: SignedTransaction,
    ) -> Result<PendingTransaction> {
        let signed_txn = signed_txn.clone();
        self.execute_with_retry(|| async {
            let client = self.get_client().await?;
            let pending_txn = client
                .submit(&signed_txn)
                .await
                .map_err(|e| ClientError::TransactionError(e.to_string()))?;

            Ok(pending_txn.into_inner())
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
                .map_err(|e| ClientError::ResourceNotFound(e.to_string()))?;

            Ok(modules
                .into_inner()
                .into_iter()
                .map(|module| module.abi.unwrap().name.to_string())
                .collect())
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;

    fn get_test_config() -> ClientConfig {
        ClientConfig {
            primary_node: NodeConfig {
                url: "https://fullnode.devnet.aptoslabs.com".to_string(),
                health_check_interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
            },
            fallback_nodes: vec![],
            retry_config: crate::config::RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(5),
            },
            rate_limit: crate::config::RateLimitConfig {
                requests_per_second: 50,
                burst_limit: 100,
            },
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = get_test_config();
        let client = Client::new(config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_get_core_modules() {
        let config = get_test_config();
        let client = Client::new(config).await.unwrap();
        let modules = client.get_core_account_modules().await;
        assert!(modules.is_ok());
        assert!(!modules.unwrap().is_empty());
    }
} 
