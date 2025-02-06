use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};
use tokio::sync::Mutex;
use crate::error::{AppError, Result};

pub struct RateLimiter {
    permits: Arc<Semaphore>,
    replenish_interval: Duration,
    last_replenish: Instant,
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32, burst_limit: u32) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(burst_limit as usize)),
            replenish_interval: Duration::from_secs_f64(1.0 / requests_per_second as f64),
            last_replenish: Instant::now(),
            last_request: Mutex::new(Instant::now()),
            min_interval: Duration::from_secs(1) / requests_per_second,
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
            Err(AppError::Internal("Rate limit exceeded".to_string()))
        }
    }

    pub async fn wait(&self) -> Result<()> {
        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }

        *last = Instant::now();
        Ok(())
    }
} 
