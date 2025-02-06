use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use aptos_sdk::rest_client::Client as AptosRestClient;
use url::Url;
use crate::error::{AppError, Result};
use crate::config::NodeConfig;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(60);
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

#[derive(Debug)]
struct NodeHealth {
    client: AptosRestClient,
    last_check: Instant,
    healthy: bool,
    consecutive_failures: u32,
}

pub struct HealthChecker {
    node_health: Arc<RwLock<HashMap<String, NodeHealth>>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            node_health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_node(&self, config: &NodeConfig) -> Result<()> {
        let client = AptosRestClient::new(config.url.clone());
        
        // Perform initial health check
        let initial_health = self.check_node_health_internal(&client).await?;
        
        let node_health = NodeHealth {
            client,
            last_check: Instant::now(),
            healthy: initial_health,
            consecutive_failures: 0,
        };

        let mut health = self.node_health.write().await;
        health.insert(config.url.to_string(), node_health);
        Ok(())
    }

    pub async fn check_node(&self, node_url: &str) -> Result<()> {
        let now = Instant::now();
        let mut health = self.node_health.write().await;
        
        match health.get(node_url) {
            Some(node) if now.duration_since(node.last_check) < Duration::from_secs(60) && node.healthy => {
                Ok(())
            }
            _ => {
                // Perform health check
                health.insert(node_url.to_string(), NodeHealth {
                    client: AptosRestClient::new(Url::parse(node_url).map_err(|e| AppError::config_error(&e.to_string()))?),
                    healthy: true,
                    last_check: now,
                    consecutive_failures: 0,
                });
                Ok(())
            }
        }
    }

    pub async fn mark_unhealthy(&self, node_url: &str) {
        let mut health = self.node_health.write().await;
        if let Some(node) = health.get_mut(node_url) {
            node.healthy = false;
            node.consecutive_failures += 1;
        }
    }

    pub async fn get_healthy_client(&self) -> Result<AptosRestClient> {
        let now = Instant::now();
        let mut health = self.node_health.write().await;
        
        for (_url, node) in health.iter_mut() {
            // Check if we need to perform a health check
            if node.last_check.elapsed() >= HEALTH_CHECK_INTERVAL {
                node.healthy = self.check_node_health_internal(&node.client).await.unwrap_or(false);
                node.last_check = now;
                
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

        Err(AppError::no_healthy_nodes())
    }

    async fn check_node_health_internal(&self, client: &AptosRestClient) -> Result<bool> {
        match client.get_ledger_information().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 
