use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const MAX_REQUESTS: usize = 100;
pub const WINDOW_SECS: u64 = 60;

#[derive(Clone)]
pub struct RateLimiter {
    windows: Arc<Mutex<HashMap<IpAddr, VecDeque<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, ip: IpAddr) -> (bool, usize, u64) {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let window = windows.entry(ip).or_default();
        let duration = Duration::from_secs(WINDOW_SECS);

        while window.front().map_or(false, |t| now.duration_since(*t) >= duration) {
            window.pop_front();
        }

        let count = window.len();
        if count >= MAX_REQUESTS {
            let oldest = *window.front().unwrap();
            let elapsed = now.duration_since(oldest).as_secs().min(WINDOW_SECS);
            return (false, 0, WINDOW_SECS - elapsed);
        }

        window.push_back(now);
        (true, MAX_REQUESTS - count - 1, WINDOW_SECS)
    }
}
