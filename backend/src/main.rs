#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

use axum::{Router, middleware, routing::get};
use shared_backend::middleware::{
    HstsState, TitleState, cors_layer, hsts_layer, security_headers_layer, title_injection_layer,
};
use crate::config::AppConfig;
use shared_backend::tracing_init::{default_log_dir, init_tracing};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;

mod ip;
mod asn_types;
mod config;
mod cookie_auth;
mod routes;
pub mod services;
pub mod middleware;
mod session_id;
mod state;
pub mod utils;

use config::AppConfig;
use routes::{auth, lookup};
use services::rate_limit::UpstreamRateLimiter;
pub use services::{dns, ip, query, rate_limit};
use state::AppState;

/// Sliding-window per-IP request budget for the `rate_limit_middleware`.
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing(default_log_dir().as_deref());

    let config = AppConfig::load_from_env(4404);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let upstream_limiter = Arc::new(UpstreamRateLimiter::new());
    let state = AppState::new(config.clone(), client, upstream_limiter.clone());

    lookup::generate_pwa_manifest(&config.0.site_title);

    let state_clone = state.clone();
    let window = RATE_LIMIT_WINDOW;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_rate_limits(window).await;
        }
    });

    let server_config = Arc::new(ServerConfig::from_env("TRACE"));
    let cors = cors_layer(&crate::middleware::CorsState(server_config.clone()));

    let api_routes = Router::new()
        .route(
            "/lookup/{query}",
            get(lookup::handle_lookup).layer(middleware::from_fn_with_state(
                state.clone(),
                auth::require_pin,
            )),
        )
        .route("/verify-pin", axum::routing::post(auth::verify_pin))
        .route("/logout", axum::routing::post(auth::logout))
        .route(
            "/auth-check",
            axum::routing::get(auth::auth_check).layer(middleware::from_fn_with_state(
                state.clone(),
                auth::require_pin,
            )),
        )
        .route("/pin-required", axum::routing::get(auth::pin_required))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/config", get(lookup::serve_config))
        .route("/health", get(lookup::serve_health))
        .route("/", get(lookup::serve_index))
        .route("/index.html", get(lookup::serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(middleware::from_fn_with_state(
            crate::middleware::TitleState(server_config.clone()),
            title_injection_layer,
        ))
        .layer(middleware::from_fn_with_state(
            crate::middleware::crate::middleware::HstsState(server_config.clone()),
            hsts_layer,
        ))
        .layer(middleware::from_fn_with_state(crate::middleware::SecurityHeadersState(server_config.clone()), crate::middleware::security_headers_layer))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.0.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(graceful_shutdown())
    .await?;

    Ok(())
}

async fn graceful_shutdown() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = signal(SignalKind::interrupt()).ok();
    let mut sigterm = signal(SignalKind::terminate()).ok();

    tokio::select! {
        _ = async {
            if let Some(ref mut s) = sigint {
                s.recv().await;
            } else {
                std::future::pending::<()>().await;
            }
        } => tracing::info!("received SIGINT"),
        _ = async {
            if let Some(ref mut s) = sigterm {
                s.recv().await;
            } else {
                std::future::pending::<()>().await;
            }
        } => tracing::info!("received SIGTERM"),
    }

    tracing::info!("draining connections (5s)");
    tokio::time::sleep(Duration::from_secs(5)).await;
}
