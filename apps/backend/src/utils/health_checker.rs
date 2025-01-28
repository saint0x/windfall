use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;
use aptos_sdk::rest_client::Client as AptosRestClient;
use url::Url;
use crate::error::{ClientError, Result};
use crate::config::NodeConfig;

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
        let node_health = NodeHealth {
            client,
            last_check: Instant::now(),
            healthy: true,
            consecutive_failures: 0,
        };

        let mut nodes = self.nodes.write().await;
        nodes.insert(config.url.clone(), node_health);
        Ok(())
    }

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
        let nodes = self.nodes.read().await;
        for (_, node) in nodes.iter() {
            if node.healthy {
                return Ok(node.client.clone());
            }
        }
        Err(ClientError::NoHealthyNodes)
    }
} 
