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

use super::{COOKIE_NAME, VerifyPinPayload};
use crate::state::AppState;

pub async fn verify_pin(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let pin_req = &state.config.0.pin;
    if pin_req.is_none() {
        return (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response();
    }

    // shared-assets normalizes the IP and applies the trust-proxy list,
    // closing the X-Forwarded-For bypass the previous local impl had.
    let ip = get_client_ip(
        &headers,
        addr,
        state.config.0.trust_proxy,
        &state.config.0.trusted_proxies,
    );
    let ip_str = ip.to_string();
    let lockout_dur = Duration::from_secs(state.config.0.lockout_time_minutes * 60);

    if attempts::is_locked_out(&ip_str, state.config.0.max_attempts, lockout_dur) {
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

    let expected_pin = match pin_req.as_ref() {
        Some(p) => p,
        None => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "success": true })),
            )
                .into_response();
        }
    };
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

        let session_id = shared_backend::session_id::generate_session_id();
        state
            .active_sessions
            .write()
            .await
            .insert(session_id.clone());

        let secure = shared_backend::cookie_auth::cookie_should_be_secure(
            &headers,
            &state.config.0.base_url,
        );

        let cookie = shared_backend::cookie_auth::build_cookie(
            COOKIE_NAME,
            &session_id,
            state.config.0.cookie_max_age_hours,
            secure,
        );
        let cookie_str = cookie.to_string();
        let mut headers = HeaderMap::new();
        if let Ok(hv) = header::HeaderValue::from_str(&cookie_str) {
            headers.insert(header::SET_COOKIE, hv);
        }
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        let attempt = attempts::record_attempt(&ip_str);
        let remaining = state.config.0.max_attempts.saturating_sub(attempt.count);
        tracing::warn!(
            target: "auth",
            "failed PIN attempt #{count} from {ip_str}",
            count = attempt.count
        );
        if attempt.count >= state.config.0.max_attempts {
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
