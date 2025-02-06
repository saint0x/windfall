use crate::error::Result;
use tokio::time::{sleep, Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    tokens: Mutex<u32>,
    last_replenish: Mutex<Instant>,
    max_tokens: u32,
    replenish_interval: Duration,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32, burst_limit: u32) -> Self {
        Self {
            tokens: Mutex::new(burst_limit),
            last_replenish: Mutex::new(Instant::now()),
            max_tokens: burst_limit,
            replenish_interval: Duration::from_secs_f64(1.0 / requests_per_second as f64),
            min_interval: Duration::from_secs(1) / requests_per_second,
        }
    }

    pub async fn acquire_permit(&self) -> Result<()> {
        loop {
            let now = Instant::now();
            let mut last = self.last_replenish.lock().await;
            let elapsed = now.duration_since(*last);

            if elapsed >= self.replenish_interval {
                *last = now;
                let mut tokens = self.tokens.lock().await;
                *tokens = (*tokens + 1).min(self.max_tokens);
            }

            let mut tokens = self.tokens.lock().await;
            if *tokens > 0 {
                *tokens -= 1;
                return Ok(());
            }

            drop(tokens);
            drop(last);
            sleep(self.min_interval).await;
        }
    }
} 
