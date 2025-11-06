use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::error::AggregatorError;

// RateLimiter struct to manage API request limits
pub struct RateLimiter {
    max_requests: u32,
    interval: Duration,
    requests_made: u32,
    last_reset: Arc<Mutex<Instant>>, 
}

impl RateLimiter {
    // Create a new RateLimiter
    pub fn new(max_requests: u32, interval_secs: u64) -> Self {
        RateLimiter {
            max_requests,
            interval: Duration::new(interval_secs, 0),
            requests_made: 0,
            last_reset: Arc::new(Mutex::new(Instant::now())),
        }
    }
    // Check if a request can be made, otherwise return an error
    pub async fn check_if_rate_limited(&mut self)-> Result<(), AggregatorError> {
        let mut last_reset = self.last_reset.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last_reset) >= self.interval {
            self.requests_made = 0;
            *last_reset = now;
        }

        if self.requests_made >= self.max_requests {
            return Err(AggregatorError::RateLimitExceeded("Rate limit exceeded, retry.".to_string()));
        }

        self.requests_made += 1;
        Ok(())
    }
}



