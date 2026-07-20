//! Per-upstream rate limiting for outbound HTTP calls.
//!
//! Different third-party services have different rate limits:
//!   - RIPE stat: documented at 1 req/s
//!   - PeeringDB: documented at 1 req / 2s
//!   - ipapi.co / ip-api.com / ipwho.is: undocumented but real
//!
//! Without throttling, a single user requesting `/api/lookup/AS15169` fires
//! 3 parallel RIPE calls and a single `/api/lookup/{ip}` can fire all 3
//! fallback providers sequentially. Without per-upstream limits, this
//! easily exhausts the upstream quotas.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Token-bucket-ish limiter keyed by upstream name. Each upstream has a
/// minimum interval; `acquire` blocks until the next call is allowed.
pub struct UpstreamRateLimiter {
    min_interval: HashMap<&'static str, Duration>,
    last_request: Mutex<HashMap<&'static str, Instant>>,
}

impl UpstreamRateLimiter {
    /// Intervals chosen with a small safety margin over the documented
    /// rate limits. See module docs.
    pub fn new() -> Self {
        let mut min_interval: HashMap<&'static str, Duration> = HashMap::new();
        min_interval.insert("ripe_stat", Duration::from_millis(1100));
        min_interval.insert("ripe_overview", Duration::from_millis(1100));
        min_interval.insert("peeringdb", Duration::from_millis(2100));
        min_interval.insert("ipapi", Duration::from_millis(500));
        min_interval.insert("ipapi_com", Duration::from_millis(500));
        min_interval.insert("ipwhois", Duration::from_millis(500));
        Self {
            min_interval,
            last_request: Mutex::new(HashMap::new()),
        }
    }

    /// Wait until it's safe to call `upstream`, then record the call.
    pub async fn acquire(&self, upstream: &'static str) {
        let Some(min) = self.min_interval.get(upstream).copied() else {
            return;
        };
        let now = std::time::Instant::now();
        let earliest = {
            let mut last = self
                .last_request
                .lock()
                .expect("rate limiter mutex poisoned");
            let next_allowed = last.get(upstream)
                .copied()
                .map(|t| t + min)
                .unwrap_or(now);
            let actual_earliest = std::cmp::max(now, next_allowed);
            last.insert(upstream, actual_earliest);
            actual_earliest
        };
        if now < earliest {
            tokio::time::sleep(earliest - now).await;
        }
    }
}

impl Default for UpstreamRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_upstream_returns_immediately() {
        let l = UpstreamRateLimiter::new();
        l.acquire("not_in_table").await; // should not panic / block
    }

    #[tokio::test]
    async fn ripe_stat_interval_is_at_least_1s() {
        let l = UpstreamRateLimiter::new();
        assert!(
            l.min_interval.get("ripe_stat").copied().unwrap() >= Duration::from_secs(1),
            "RIPE stat must be throttled to at least 1 req/s"
        );
    }

    #[tokio::test]
    async fn peeringdb_interval_is_at_least_2s() {
        let l = UpstreamRateLimiter::new();
        assert!(
            l.min_interval.get("peeringdb").copied().unwrap() >= Duration::from_secs(2),
            "PeeringDB must be throttled to at least 1 req / 2s"
        );
    }

    #[tokio::test]
    async fn second_call_is_delayed() {
        let l = UpstreamRateLimiter::new();
        let start = Instant::now();
        l.acquire("peeringdb").await; // first call: instant
        l.acquire("peeringdb").await; // second call: must wait ~2s
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(2000),
            "expected >= 2s wait between calls, got {elapsed:?}"
        );
    }
}
