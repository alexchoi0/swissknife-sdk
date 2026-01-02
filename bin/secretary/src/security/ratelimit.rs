use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    limits: Mutex<HashMap<String, Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }

    pub fn check(&self, key: &str) -> Result<(), String> {
        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - self.window;

        let timestamps = limits.entry(key.to_string()).or_default();

        timestamps.retain(|t| *t > cutoff);

        if timestamps.len() >= self.max_requests {
            return Err(format!(
                "Rate limit exceeded: {} requests in {:?}",
                self.max_requests, self.window
            ));
        }

        timestamps.push(now);
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref DNS_LIMITER: RateLimiter = RateLimiter::new(10, Duration::from_secs(60));
    pub static ref FILE_LIMITER: RateLimiter = RateLimiter::new(100, Duration::from_secs(60));
}
