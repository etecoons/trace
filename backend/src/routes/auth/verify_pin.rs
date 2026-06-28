use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use constant_time_eq::constant_time_eq;
use shared_backend::auth::attempts;
use shared_backend::server::get_client_ip;
use std::net::SocketAddr;
use std::time::Duration;

use crate::state::AppState;
use super::{COOKIE_NAME, VerifyPinPayload};

pub fn generate_session_id() -> String {
    use std::fs::File;
    use std::io::Read;
    let file = File::open("/dev/urandom").ok();
    let mut bytes = [0u8; 16];
    if let Some(mut f) = file {
        if f.read_exact(&mut bytes).is_ok() {
            return bytes.iter().map(|b| format!("{:02x}", b)).collect();
        }
    }
    let random_val = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(random_val.to_string().as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn verify_pin(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let pin_req = &state.config.pin;
    if pin_req.is_none() {
        return (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response();
    }

    // shared-assets normalizes the IP and applies the trust-proxy list,
    // closing the X-Forwarded-For bypass the previous local impl had.
    let ip = get_client_ip(
        &headers,
        addr,
        state.config.trust_proxy,
        &state.config.trusted_proxies,
    );
    let ip_str = ip.to_string();
    let lockout_dur = Duration::from_secs(state.config.lockout_time_minutes * 60);

    if attempts::is_locked_out(&ip_str, state.config.max_attempts as u32, lockout_dur) {
        let remaining = attempts::lockout_remaining_secs(&ip_str, lockout_dur);
        let time_left_min = (remaining as f64 / 60.0).ceil() as u64;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Too many attempts. Please try again in {} minute(s).", time_left_min)
            })),
        )
            .into_response();
    }

    let expected_pin = pin_req.as_ref().unwrap();
    let pin_str = payload.pin.as_deref().unwrap_or("").trim();

    if pin_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "PIN is required." })),
        )
            .into_response();
    }

    if constant_time_eq(pin_str.as_bytes(), expected_pin.as_bytes()) {
        attempts::reset_attempts(&ip_str);

        let session_id = generate_session_id();
        state
            .active_sessions
            .write()
            .await
            .insert(session_id.clone());

        let cookie_max_age = Duration::from_secs((state.config.cookie_max_age_hours * 3600) as u64);
        let secure = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or_else(|| state.config.base_url.starts_with("https"));

        let cookie_val = format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
            COOKIE_NAME,
            session_id,
            cookie_max_age.as_secs(),
            if secure { "; Secure" } else { "" }
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_str(&cookie_val).unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        let attempt = attempts::record_attempt(&ip_str);
        let remaining = state
            .config
            .max_attempts
            .saturating_sub(attempt.count as usize);
        tracing::warn!(
            target: "auth",
            "failed PIN attempt #{count} from {ip_str}",
            count = attempt.count
        );
        if attempt.count as usize >= state.config.max_attempts {
            tracing::warn!(target: "auth", "IP {ip_str} locked out");
        }

        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid PIN",
                "attemptsLeft": remaining
            })),
        )
            .into_response()
    }
}
