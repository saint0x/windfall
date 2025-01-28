use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;
use crate::error::{ClientError, Result};

pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    last_update: Arc<Mutex<Instant>>,
    tokens_per_second: f64,
    max_tokens: f64,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32, burst_limit: u32) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(burst_limit as f64)),
            last_update: Arc::new(Mutex::new(Instant::now())),
            tokens_per_second: requests_per_second as f64,
            max_tokens: burst_limit as f64,
        }
    }

    pub async fn acquire_permit(&self) -> Result<()> {
        let mut tokens = self.tokens.lock().await;
        let mut last_update = self.last_update.lock().await;

        let now = Instant::now();
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        *last_update = now;

        // Replenish tokens based on elapsed time
        *tokens = (*tokens + elapsed * self.tokens_per_second).min(self.max_tokens);

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            Err(ClientError::RateLimitExceeded)
        }
    }

    pub async fn available_tokens(&self) -> f64 {
        let tokens = self.tokens.lock().await;
        *tokens
    }
} 
