use backend::utils::RateLimiter;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_rate_limiter_basic() {
    let limiter = RateLimiter::new(10, 1); // 10 requests per second, burst of 1
    
    // First request should succeed
    assert!(limiter.acquire_permit().await.is_ok());
    
    // Immediate second request should fail
    assert!(limiter.acquire_permit().await.is_err());
    
    // Wait for replenish
    sleep(Duration::from_millis(100)).await;
    
    // Should succeed again
    assert!(limiter.acquire_permit().await.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_burst() {
    let limiter = RateLimiter::new(10, 3); // 10 requests per second, burst of 3
    
    // First three requests should succeed
    assert!(limiter.acquire_permit().await.is_ok());
    assert!(limiter.acquire_permit().await.is_ok());
    assert!(limiter.acquire_permit().await.is_ok());
    
    // Fourth request should fail
    assert!(limiter.acquire_permit().await.is_err());
}

#[tokio::test]
async fn test_rate_limiter_replenish() {
    let limiter = RateLimiter::new(10, 1); // 10 requests per second, burst of 1
    
    // First request should succeed
    assert!(limiter.acquire_permit().await.is_ok());
    
    // Wait for full replenish
    sleep(Duration::from_millis(200)).await;
    
    // Should succeed again
    assert!(limiter.acquire_permit().await.is_ok());
} 