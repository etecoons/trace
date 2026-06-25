#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

use axum::{Router, middleware, routing::get};
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod asn;
mod asn_types;
mod auth;
mod config;
mod dns;
mod handlers;
mod ip;
mod query;
mod state;
mod utils;
mod whois;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("LOG_DIR").ok();
    let file_layer = if let Some(ref dir) = log_dir {
        let _ = std::fs::create_dir_all(dir);
        let log_file_path = std::path::Path::new(dir).join("error.log");
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(log_file_path)
            .ok()
            .map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
            })
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
        .init();

    let config = AppConfig::load();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build reqwest client");

    let state = AppState::new(config.clone(), client);

    // Pre-generate PWA files if directory is already compiled
    handlers::generate_pwa_manifest(&config.site_title);

    // Lockout cleanup thread
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_lockouts().await;
        }
    });

    let cors = if config.allowed_origins == "*" {
        tower_http::cors::CorsLayer::permissive()
    } else {
        let mut cors = tower_http::cors::CorsLayer::new()
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::COOKIE]);
        for origin in config.allowed_origins.split(',') {
            if let Ok(parsed) = origin.trim().parse::<axum::http::HeaderValue>() {
                cors = cors.allow_origin(parsed);
            }
        }
        cors.allow_credentials(true)
    };

    let api_routes = Router::new()
        .route(
            "/lookup/:query",
            get(handlers::handle_lookup).layer(middleware::from_fn_with_state(
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
            auth::origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/config", get(handlers::serve_config))
        .route("/health", get(handlers::serve_health))
        .route("/", get(handlers::serve_index))
        .route("/index.html", get(handlers::serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(middleware::from_fn(auth::security_headers_middleware))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
