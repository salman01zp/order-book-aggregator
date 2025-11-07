use tokio::time::{Duration, Instant};

use crate::error::AggregatorError;

// RateLimiter struct to manage API request limits
pub struct RateLimiter {
    max_requests: u32,
    interval: Duration,
    requests_made: u32,
    last_reset: tokio::time::Instant,
}

impl RateLimiter {
    // Create a new RateLimiter
    pub fn new(max_requests: u32, interval_secs: u64) -> Self {
        RateLimiter {
            max_requests,
            interval: Duration::new(interval_secs, 0),
            requests_made: 0,
            last_reset: Instant::now(),
        }
    }
    // Check if a request can be made, otherwise return an error
    pub async fn check_if_rate_limited(&mut self) -> Result<(), AggregatorError> {
        let now = Instant::now();
        if now.duration_since(self.last_reset) >= self.interval {
            self.requests_made = 0;
            self.last_reset = now;
        }
        if self.requests_made < self.max_requests {
            self.requests_made += 1;
            Ok(())
        } else {
            Err(AggregatorError::RateLimitExceeded(
                "Rate limit exceeded".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_rate_limiter() {
        let mut rate_limiter = RateLimiter::new(1, 2); // 1 request per 2 seconds
        // Use up 1 token
        assert!(rate_limiter.check_if_rate_limited().await.is_ok());
        // Next request should be rate limited wiht RateLimitExceeded error
        let res = rate_limiter.check_if_rate_limited().await;
        assert!(matches!(res, Err(AggregatorError::RateLimitExceeded(_))));
        
    }
}
