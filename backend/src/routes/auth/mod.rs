pub mod verify_pin;
pub mod logout;
pub mod pin_required;

pub use verify_pin::verify_pin;
pub use logout::logout;
pub use pin_required::{pin_required, auth_check};

use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use constant_time_eq::constant_time_eq;
use shared_backend::server::get_client_ip;
use std::net::SocketAddr;
use std::time::Duration;

pub const COOKIE_NAME: &str = "TRACE_PIN";

#[derive(serde::Deserialize)]
pub struct VerifyPinPayload {
    pub pin: Option<String>,
}

/// True if the request presents a valid PIN session (cookie or header).
///
/// Note: the cookie value is a random session ID minted by `verify_pin`
/// (not the raw PIN), so constant-time comparison is defense in depth
/// against timing leaks of the session-id table.
pub async fn is_authenticated(headers: &HeaderMap, state: &AppState) -> bool {
    let pin = match &state.config.pin {
        Some(p) => p,
        None => return true,
    };

    let cookie_pin = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c_str| {
            c_str
                .split(';')
                .find(|s| s.trim().starts_with(&format!("{}=", COOKIE_NAME)))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.trim().to_string())
        });

    let header_pin = headers.get("x-pin").and_then(|h| h.to_str().ok());

    match (cookie_pin, header_pin) {
        (Some(cookie), _) => state.active_sessions.read().await.contains(&cookie),
        (None, Some(hdr)) => constant_time_eq(hdr.as_bytes(), pin.as_bytes()),
        (None, None) => false,
    }
}

pub async fn require_pin(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !is_authenticated(req.headers(), &state).await {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

pub async fn origin_validation_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let origins_env = &state.config.allowed_origins;
    if origins_env == "*" {
        return Ok(next.run(req).await);
    }

    let referer = req.headers().get("referer").and_then(|v| v.to_str().ok());
    let host = req.headers().get("host").and_then(|v| v.to_str().ok());

    let origin = if let Some(ref_val) = referer {
        if let Ok(url) = reqwest::Url::parse(ref_val) {
            url.origin().ascii_serialization()
        } else {
            ref_val.to_string()
        }
    } else if let Some(host_val) = host {
        let proto = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http");
        format!("{}://{}", proto, host_val)
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let allowed_list: Vec<String> = origins_env
        .split(',')
        .map(|s| {
            let s_trim = s.trim();
            if let Ok(url) = reqwest::Url::parse(s_trim) {
                url.origin().ascii_serialization()
            } else {
                s_trim.to_string()
            }
        })
        .collect();

    let normalized_origin = if let Ok(url) = reqwest::Url::parse(&origin) {
        url.origin().ascii_serialization()
    } else {
        origin.clone()
    };

    if allowed_list.contains(&normalized_origin) {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("Blocked request from origin: {}", origin);
        Err(StatusCode::FORBIDDEN)
    }
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);

    let ip = get_client_ip(
        req.headers(),
        addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0))),
        state.config.trust_proxy,
        &state.config.trusted_proxies,
    );

    // 100 requests per 60 seconds per IP
    if !state
        .check_rate_limit(
            ip.parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
            100,
            Duration::from_secs(60),
        )
        .await
    {
        let body = serde_json::json!({
            "error": "Too many requests. Please slow down."
        });
        let mut response = axum::response::Json(body).into_response();
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        return Ok(response);
    }

    Ok(next.run(req).await)
}
