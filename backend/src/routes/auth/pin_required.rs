use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use shared_backend::auth::attempts;
use shared_backend::server::get_client_ip;
use std::net::SocketAddr;
use std::time::Duration;

use crate::state::AppState;
use super::is_authenticated;

pub async fn auth_check(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    if !is_authenticated(&headers, &state).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    StatusCode::OK.into_response()
}

pub async fn pin_required(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ip = get_client_ip(
        &headers,
        addr,
        state.config.trust_proxy,
        &state.config.trusted_proxies,
    );
    let ip_str = ip.to_string();
    let lockout_dur = Duration::from_secs(state.config.lockout_time_minutes * 60);
    Json(serde_json::json!({
        "required": state.config.pin.is_some(),
        "length": state.config.pin.as_ref().map(|p| p.len()).unwrap_or(0),
        "locked": attempts::is_locked_out(&ip_str, state.config.max_attempts as u32, lockout_dur),
        "enable_translation": state.config.enable_translation,
        "enable_themes": state.config.enable_themes,
        "enable_print": state.config.enable_print,
        "show_version": state.config.show_version,
        "show_github": state.config.show_github,
    }))
}
