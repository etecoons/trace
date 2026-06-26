use crate::config::AppConfig;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct LoginAttempts {
    pub count: usize,
    pub last_attempt: Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: reqwest::Client,
    pub login_attempts: Arc<RwLock<HashMap<IpAddr, LoginAttempts>>>,
    pub active_sessions: Arc<RwLock<std::collections::HashSet<String>>>,
    pub rate_limiter: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
}

impl AppState {
    pub fn new(config: AppConfig, client: reqwest::Client) -> Self {
        Self {
            config,
            client,
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(std::collections::HashSet::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn is_locked_out(&self, ip: IpAddr) -> bool {
        let map = self.login_attempts.read().await;
        if let Some(attempts) = map.get(&ip) {
            if attempts.count >= self.config.max_attempts {
                let elapsed = attempts.last_attempt.elapsed();
                let lockout_dur = Duration::from_secs(self.config.lockout_time_minutes * 60);
                if elapsed < lockout_dur {
                    return true;
                }
            }
        }
        false
    }

    pub async fn record_login_attempt(&self, ip: IpAddr) {
        let mut map = self.login_attempts.write().await;
        let attempts = map.entry(ip).or_insert(LoginAttempts {
            count: 0,
            last_attempt: Instant::now(),
        });
        attempts.count += 1;
        attempts.last_attempt = Instant::now();
    }

    pub async fn reset_login_attempts(&self, ip: IpAddr) {
        let mut map = self.login_attempts.write().await;
        map.remove(&ip);
    }

    pub async fn clean_old_lockouts(&self) {
        let lockout_dur = Duration::from_secs(self.config.lockout_time_minutes * 60);
        let mut map = self.login_attempts.write().await;
        map.retain(|_, attempts| attempts.last_attempt.elapsed() < lockout_dur);
    }

    pub async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let max_requests = 100; // 100 requests
        let window = Duration::from_secs(60); // per 60 seconds
        let now = Instant::now();

        let mut map = self.rate_limiter.write().await;
        let timestamps = map.entry(ip).or_insert_with(Vec::new);

        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub async fn clean_old_rate_limits(&self) {
        let window = Duration::from_secs(60);
        let now = Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}
