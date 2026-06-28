use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use crate::state::AppState;
use super::COOKIE_NAME;

pub async fn logout(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let cookie_val = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c_str| {
            c_str
                .split(';')
                .find(|s| s.trim().starts_with(&format!("{}=", COOKIE_NAME)))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.trim().to_string())
        });

    if let Some(session_id) = cookie_val {
        state.active_sessions.write().await.remove(&session_id);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        header::HeaderValue::from_static(
            "TRACE_PIN=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        ),
    );
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true })),
    )
        .into_response()
}
