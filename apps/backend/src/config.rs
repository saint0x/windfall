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