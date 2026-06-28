#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

use axum::{Router, middleware, routing::get};
use shared_backend::middleware::{
    HstsState, TitleState, cors_layer, hsts_layer, security_headers_layer, title_injection_layer,
};
use shared_backend::server::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod asn_types;
mod config;
mod routes;
pub mod services;
mod state;
pub mod utils;

use config::AppConfig;
pub use services::{dns, ip, query, rate_limit};
use services::rate_limit::UpstreamRateLimiter;
use routes::{auth, lookup};
use state::AppState;

/// Sliding-window per-IP request budget for the `rate_limit_middleware`.
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("LOG_DIR").ok().or_else(|| {
        let data_dir = std::path::Path::new("/app/data");
        if data_dir.is_dir() {
            Some("/app/data/log".to_string())
        } else {
            Some("/app/log".to_string())
        }
    });

    let (file_layer_error, file_layer_app) = if let Some(ref dir) = log_dir {
        if dir == "off" || dir == "none" || dir == "false" {
            (None, None)
        } else {
            let _ = std::fs::create_dir_all(dir);
            let error_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("app.log"))
                .ok();

            let error_layer = error_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
            });

            let app_layer = app_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
            });

            (error_layer, app_layer)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer_error)
        .with(file_layer_app)
        .init();

    let config = AppConfig::load();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build reqwest client");

    let upstream_limiter = Arc::new(UpstreamRateLimiter::new());
    let state = AppState::new(config.clone(), client, upstream_limiter.clone());

    lookup::generate_pwa_manifest(&config.site_title);

    // Background cleanup. PIN-attempt lockouts are now global via
    // shared-backend and clean themselves up; we only need to clean the
    // per-IP rate-limiter table.
    let state_clone = state.clone();
    let window = RATE_LIMIT_WINDOW;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_rate_limits(window).await;
        }
    });

    // shared-backend drives the security middleware from a single
    // ServerConfig. The TRACE prefix makes `TRACE_PIN`, `TRACE_PORT`,
    // `TRACE_SITE_TITLE`, etc. take precedence over generic env vars.
    let server_config = Arc::new(ServerConfig::from_env("TRACE"));
    let cors = cors_layer(&server_config);

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

    // Per-upstream throttle for outbound calls is held in AppState
    // (`state.upstream_limiter`) so `asn::fetch_asn_data` and
    // `ip::try_ip_lookup` can `acquire(...)` before each request.

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/config", get(lookup::serve_config))
        .route("/health", get(lookup::serve_health))
        .route("/", get(lookup::serve_index))
        .route("/index.html", get(lookup::serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        // shared-backend layers: title injection sees the raw HTML,
        // security headers add CSP/X-Frame-Options/etc., HSTS is HTTPS-only,
        // CORS is outermost so preflight requests aren't gated by auth.
        .layer(middleware::from_fn_with_state(
            TitleState(server_config.clone()),
            title_injection_layer,
        ))
        .layer(middleware::from_fn_with_state(
            HstsState(server_config.clone()),
            hsts_layer,
        ))
        .layer(middleware::from_fn(security_headers_layer))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(graceful_shutdown())
    .await
    .expect("server error");
}

async fn graceful_shutdown() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");

    tokio::select! {
        _ = sigint.recv() => tracing::info!("received SIGINT"),
        _ = sigterm.recv() => tracing::info!("received SIGTERM"),
    }

    tracing::info!("draining connections (5s)");
    tokio::time::sleep(Duration::from_secs(5)).await;
}
