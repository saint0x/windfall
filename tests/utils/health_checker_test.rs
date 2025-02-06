use backend::{
    utils::HealthChecker,
    config::NodeConfig,
    error::ClientError,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_health_checker_basic() {
    let checker = HealthChecker::new();
    
    // Add a healthy node
    let config = NodeConfig {
        url: "https://fullnode.devnet.aptoslabs.com".to_string(),
    };
    
    assert!(checker.add_node(&config).await.is_ok());
    assert!(checker.get_healthy_client().await.is_ok());
}

#[tokio::test]
async fn test_health_checker_unhealthy_node() {
    let checker = HealthChecker::new();
    
    // Add an unhealthy node
    let config = NodeConfig {
        url: "https://invalid.node.url".to_string(),
    };
    
    // Adding should fail because initial health check fails
    assert!(checker.add_node(&config).await.is_err());
}

#[tokio::test]
async fn test_health_checker_node_failure() {
    let checker = HealthChecker::new();
    
    // Add a node that will fail after some time
    let config = NodeConfig {
        url: "https://fullnode.devnet.aptoslabs.com".to_string(),
    };
    
    assert!(checker.add_node(&config).await.is_ok());
    
    // First check should succeed
    assert!(checker.get_healthy_client().await.is_ok());
    
    // Wait for health check interval
    sleep(Duration::from_secs(61)).await;
    
    // Should still succeed because node is still healthy
    assert!(checker.get_healthy_client().await.is_ok());
} 