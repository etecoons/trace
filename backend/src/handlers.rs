use crate::asn::fetch_asn_data;
use crate::ip::try_ip_lookup;
use crate::query::detect_query_type;
use crate::state::AppState;
use crate::whois::{parse_whois_data, whois_lookup};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use std::fs;
use std::path::Path as StdPath;
use std::sync::LazyLock;
use std::time::Instant;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

pub async fn serve_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "siteTitle": state.config.site_title,
        "pinRequired": state.config.pin.is_some(),
        "pinLength": state.config.pin.as_ref().map_or(0, |p| p.len()),
        "enableTranslation": state.config.enable_translation,
        "enable_translation": state.config.enable_translation,
        "enableThemes": state.config.enable_themes,
        "enable_themes": state.config.enable_themes,
        "enablePrint": state.config.enable_print,
        "enable_print": state.config.enable_print,
    }))
}

pub async fn serve_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
        "uptime": START_TIME.elapsed().as_secs()
    }))
}

pub async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = StdPath::new("frontend/dist/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let rendered = content.replace("{{SITE_TITLE}}", &state.config.site_title);
            Html(rendered).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn generate_pwa_manifest(site_title: &str) {
    let dist_dir = StdPath::new("frontend/dist");
    if !dist_dir.exists() {
        return;
    }

    let mut assets = Vec::new();
    fn walk_dir(dir: &StdPath, base: &str, assets: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let full = entry.path();
                let rel = if base.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", base, name)
                };
                if full.is_dir() {
                    walk_dir(&full, &rel, assets);
                } else {
                    assets.push(format!("/{}", rel));
                }
            }
        }
    }
    walk_dir(dist_dir, "", &mut assets);

    let _ = fs::write(
        dist_dir.join("asset-manifest.json"),
        serde_json::to_string_pretty(&assets).unwrap_or_default(),
    );

    let pwa_manifest = serde_json::json!({
        "name": site_title,
        "short_name": site_title,
        "description": "A simple WHOIS lookup web application using free APIs",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#ffffff",
        "theme_color": "#000000",
        "icons": [
            { "src": "favicon.svg", "type": "image/svg+xml", "sizes": "any" },
            { "src": "favicon.png", "type": "image/png", "sizes": "192x192" },
            { "src": "favicon.png", "type": "image/png", "sizes": "512x512" }
        ],
        "orientation": "any"
    });

    let _ = fs::write(
        dist_dir.join("manifest.json"),
        serde_json::to_string_pretty(&pwa_manifest).unwrap_or_default(),
    );
}

pub async fn handle_lookup(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let query_type = detect_query_type(&query);
    match query_type {
        "whois" => match whois_lookup(&query).await {
            Ok(raw_data) => {
                let parsed = parse_whois_data(&raw_data, &query).await;
                let ldh_name = parsed.domain_name.clone();
                let response_data = serde_json::json!({
                    "ldhName": ldh_name,
                    "handle": query,
                    "status": parsed.status,
                    "ipAddresses": parsed.ip_addresses,
                    "events": [
                        { "eventAction": "registration", "eventDate": parsed.creation_date },
                        { "eventAction": "expiration", "eventDate": parsed.expiration_date },
                        { "eventAction": "lastChanged", "eventDate": parsed.last_updated }
                    ],
                    "nameservers": parsed.nameservers.into_iter().map(|ns| serde_json::json!({ "ldhName": ns })).collect::<Vec<_>>(),
                    "entities": [{
                        "roles": ["registrar"],
                        "vcardArray": ["vcard", [["version", {}, "text", "4.0"], ["fn", {}, "text", parsed.registrar], ["email", {}, "text", ""]]]
                    }]
                });
                Json(serde_json::json!({ "type": "whois", "data": response_data })).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Error fetching WHOIS data", "message": e }))).into_response(),
        },
        "ip" => match try_ip_lookup(&state.client, &state.upstream_limiter, &query).await {
            Ok(ip_data) => Json(serde_json::json!({ "type": "ip", "data": ip_data })).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Error fetching IP data", "message": e }))).into_response(),
        },
        "asn" => {
            let asn_number = query.to_uppercase().replace("AS", "");
            match fetch_asn_data(&state.client, &state.upstream_limiter, &asn_number).await {
                Ok(asn_data) => Json(serde_json::json!({ "type": "asn", "data": asn_data.data })).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Error fetching ASN data", "message": e }))).into_response(),
            }
        }
        _ => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid input", "message": "Please enter a valid domain name, IP address, or ASN number" }))).into_response(),
    }
}
