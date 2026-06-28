use crate::config::AppConfig;
use crate::rate_limit::UpstreamRateLimiter;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Per-IP rate limiter. Used by the `rate_limit_middleware` in `auth.rs`
/// and (for outbound calls) by `asn.rs` / `ip.rs` via the embedded
/// [`UpstreamRateLimiter`].
///
/// PIN-attempt lockouts live in [`shared_backend::auth::attempts`] (global,
/// per-IP). This is intentionally separate: a per-IP request budget is
/// unrelated to PIN brute-force protection, and a key in this table expires
/// on a sliding window while PIN attempts stay locked for the configured
/// lockout duration.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: reqwest::Client,
    pub active_sessions: Arc<RwLock<std::collections::HashSet<String>>>,
    pub rate_limiter: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    pub upstream_limiter: Arc<UpstreamRateLimiter>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        client: reqwest::Client,
        upstream_limiter: Arc<UpstreamRateLimiter>,
    ) -> Self {
        Self {
            config,
            client,
            active_sessions: Arc::new(RwLock::new(std::collections::HashSet::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            upstream_limiter,
        }
    }

    /// Per-IP sliding-window rate limit: `max_requests` per `window`.
    /// Defaults to 100 req/60s (configured at the call site).
    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
        max_requests: usize,
        window: Duration,
    ) -> bool {
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

    /// Periodic cleanup of stale rate-limit entries (called from a
    /// tokio task spawned in `main`).
    pub async fn clean_old_rate_limits(&self, window: Duration) {
        let now = Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}
