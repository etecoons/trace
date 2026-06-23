use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path as StdPath;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Global startup time for uptime calculation
static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

// WHOIS redirect regex
static RE_REFER: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)^\s*(refer|whois|whois\s+server|registrar\s+whois\s+server)\s*:\s*([a-z0-9\-\._]+)\s*$",
    )
    .unwrap()
});

// Regexes for IP parsing
static RE_IPV4: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
    )
    .unwrap()
});

static RE_IPV6: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(?:[0-9a-f]{1,4}:){1,7}(?:[0-9a-f]{1,4}|:)|(?:::[0-9a-f]{1,4})\b")
        .unwrap()
});

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub site_title: String,
    pub apprise_url: Option<String>,
    pub apprise_message: String,
    pub pin: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4404);
        let site_title = std::env::var("SITE_TITLE").unwrap_or_else(|_| "RustWho".to_string());
        let apprise_url = std::env::var("APPRISE_URL").ok().filter(|s| !s.is_empty());
        let apprise_message = std::env::var("APPRISE_MESSAGE")
            .unwrap_or_else(|_| "WHOIS Lookup for {query} ({query_type})".to_string());
        let pin = std::env::var("RUSTWHO_PIN")
            .or_else(|_| std::env::var("PIN"))
            .ok()
            .filter(|p| {
                !p.is_empty()
                    && p.chars().all(|c| c.is_ascii_digit())
                    && p.len() >= 4
                    && p.len() <= 10
            });
        Self {
            port,
            site_title,
            apprise_url,
            apprise_message,
            pin,
        }
    }
}

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build reqwest client");

    let state = AppState {
        config: config.clone(),
        client,
    };

    // Pre-generate PWA files if directory is already compiled
    generate_pwa_manifest(&config.site_title);

    let cors = get_cors_layer();

    let api_routes = Router::new()
        .route("/lookup/:query", get(handle_lookup))
        .route("/verify-pin", axum::routing::post(verify_pin))
        .route("/logout", axum::routing::post(logout))
        .route("/auth-check", axum::routing::get(auth_check))
        .layer(middleware::from_fn(origin_validation_middleware));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/config", get(serve_config))
        .route("/health", get(serve_health))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        // Serve frontend distribution files
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Config Handler
async fn serve_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "siteTitle": state.config.site_title,
        "pinRequired": state.config.pin.is_some(),
        "pinLength": state.config.pin.as_ref().map(|p| p.len()).unwrap_or(0),
    }))
}

// Health Handler
async fn serve_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
        "uptime": START_TIME.elapsed().as_secs()
    }))
}

// Serve index.html and perform dynamic replacement of title
async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = StdPath::new("frontend/dist/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let rendered = content.replace("{{SITE_TITLE}}", &state.config.site_title);
            Html(rendered).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// Dynamic PWA Manifest Generator
fn generate_pwa_manifest(site_title: &str) {
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

    // Save asset-manifest.json
    let asset_path = dist_dir.join("asset-manifest.json");
    if let Ok(json) = serde_json::to_string_pretty(&assets) {
        let _ = fs::write(asset_path, json);
    }

    // Save manifest.json
    let pwa_manifest = serde_json::json!({
        "name": site_title,
        "short_name": site_title,
        "description": "A simple WHOIS lookup web application using free APIs",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#ffffff",
        "theme_color": "#000000",
        "icons": [
            {
                "src": "/assets/logo.png",
                "type": "image/png",
                "sizes": "192x192"
            },
            {
                "src": "/assets/logo.png",
                "type": "image/png",
                "sizes": "512x512"
            }
        ],
        "orientation": "any"
    });

    let manifest_path = dist_dir.join("manifest.json");
    if let Ok(json) = serde_json::to_string_pretty(&pwa_manifest) {
        let _ = fs::write(manifest_path, json);
    }
}

// CORS Helper
fn get_cors_layer() -> CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::Any;

    let origins_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    if origins_env == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let mut origins = Vec::new();
        for origin in origins_env.split(',') {
            let o = origin.trim();
            if !o.is_empty()
                && let Ok(val) = HeaderValue::from_str(o)
            {
                origins.push(val);
            }
        }
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    }
}

// Origin Validation Middleware
async fn origin_validation_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let origins_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
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

// --- Lookup Domain Logic ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IpAddresses {
    v4: Vec<String>,
    v6: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedWhoisData {
    domain_name: String,
    registrar: String,
    creation_date: String,
    expiration_date: String,
    last_updated: String,
    status: Vec<String>,
    nameservers: Vec<String>,
    ip_addresses: IpAddresses,
    raw: String,
}

fn detect_query_type(query: &str) -> &'static str {
    let clean = query.replace(['[', ']'], "");

    // ASN pattern
    let re_asn = regex::Regex::new(r"(?i)^(AS)?\d+$").unwrap();
    if re_asn.is_match(&clean) {
        return "asn";
    }

    // IPv4 pattern
    let re_ipv4 = regex::Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?:\/\d{1,2})?$").unwrap();
    if re_ipv4.is_match(&clean) {
        return "ip";
    }

    // IPv6 pattern
    if clean.contains(':') {
        if clean.parse::<std::net::IpAddr>().is_ok() {
            return "ip";
        }
        let clean_no_cidr = clean.split('/').next().unwrap_or("");
        if clean_no_cidr.parse::<std::net::IpAddr>().is_ok() {
            return "ip";
        }
    }

    // Domain pattern
    if clean.contains('.') {
        return "whois";
    }

    "unknown"
}

async fn query_whois_server(server: &str, query: &str) -> Result<String, String> {
    let addr = if server.contains(':') {
        server.to_string()
    } else {
        format!("{}:43", server)
    };

    let mut stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("Connection timeout to {}", server))?
        .map_err(|e| format!("Failed to connect to {}: {}", server, e))?;

    let query_str = format!("{}\r\n", query);
    stream
        .write_all(query_str.as_bytes())
        .await
        .map_err(|e| format!("Failed to write to socket: {}", e))?;

    let mut response = Vec::new();
    let mut buffer = [0u8; 4096];

    let read_future = async {
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buffer[..n]),
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }
        Ok(())
    };

    tokio::time::timeout(Duration::from_secs(10), read_future)
        .await
        .map_err(|_| "Timeout reading from WHOIS server".to_string())??;

    Ok(String::from_utf8_lossy(&response).into_owned())
}

fn find_redirect_server(raw_data: &str) -> Option<String> {
    for line in raw_data.lines() {
        if let Some(caps) = RE_REFER.captures(line) {
            let s = caps.get(2).unwrap().as_str().trim().to_string();
            if !s.is_empty() && s != "whois.iana.org" {
                return Some(s);
            }
        }
    }
    None
}

async fn whois_lookup(query: &str) -> Result<String, String> {
    let mut server = if query.to_lowercase().ends_with(".eu") {
        "whois.eu".to_string()
    } else {
        "whois.iana.org".to_string()
    };

    let current_query = query.to_string();
    let mut visited = HashSet::new();

    for _ in 0..4 {
        if visited.contains(&server) {
            break;
        }
        visited.insert(server.clone());

        tracing::info!("Querying WHOIS server {} for {}", server, current_query);
        let raw_data = query_whois_server(&server, &current_query).await?;

        if let Some(next_server) = find_redirect_server(&raw_data) {
            server = next_server;
        } else {
            return Ok(raw_data);
        }
    }

    Err("Too many WHOIS redirects".to_string())
}

fn extract_ips_from_raw(raw_data: &str) -> IpAddresses {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();

    for line in raw_data.lines() {
        let lower = line.to_lowercase();
        if lower.contains("ip address")
            || lower.contains("a record")
            || lower.contains("aaaa record")
            || lower.contains("addresses")
            || lower.contains("host")
            || lower.contains("dns")
        {
            for cap in RE_IPV4.find_iter(line) {
                let ip = cap.as_str().to_string();
                if !v4.contains(&ip) {
                    v4.push(ip);
                }
            }
            for cap in RE_IPV6.find_iter(line) {
                let ip = cap.as_str().to_string();
                if !v6.contains(&ip) {
                    v6.push(ip);
                }
            }
        }
    }
    IpAddresses { v4, v6 }
}

async fn resolve_dns(domain: &str) -> IpAddresses {
    let mut ips = IpAddresses::default();
    if let Ok(addrs) = tokio::net::lookup_host(format!("{}:80", domain)).await {
        for addr in addrs {
            match addr.ip() {
                std::net::IpAddr::V4(v4) => {
                    let s = v4.to_string();
                    if !ips.v4.contains(&s) {
                        ips.v4.push(s);
                    }
                }
                std::net::IpAddr::V6(v6) => {
                    let s = v6.to_string();
                    if !ips.v6.contains(&s) {
                        ips.v6.push(s);
                    }
                }
            }
        }
    }
    ips
}

async fn parse_whois_data(raw_data: &str, domain: &str) -> ParsedWhoisData {
    let mut result = ParsedWhoisData {
        domain_name: domain.to_string(),
        registrar: String::new(),
        creation_date: String::new(),
        expiration_date: String::new(),
        last_updated: String::new(),
        status: Vec::new(),
        nameservers: Vec::new(),
        ip_addresses: IpAddresses::default(),
        raw: raw_data.to_string(),
    };

    let mut dns_ips = resolve_dns(domain).await;
    let raw_ips = extract_ips_from_raw(raw_data);

    for ip in raw_ips.v4 {
        if !dns_ips.v4.contains(&ip) {
            dns_ips.v4.push(ip);
        }
    }
    for ip in raw_ips.v6 {
        if !dns_ips.v6.contains(&ip) {
            dns_ips.v6.push(ip);
        }
    }
    result.ip_addresses = dns_ips;

    if domain.to_lowercase().ends_with(".eu") {
        let mut current_section = String::new();
        for line in raw_data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('%') {
                continue;
            }

            if let Some(stripped) = trimmed.strip_suffix(':') {
                current_section = stripped.to_lowercase();
                continue;
            }

            if line.starts_with("        ") {
                let parts: Vec<&str> = trimmed.splitn(2, ':').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let val = parts[1];
                    match current_section.as_str() {
                        "registrar" => {
                            if key == "Name" {
                                result.registrar = val.to_string();
                            }
                        }
                        "name servers" => {
                            if !key.contains(':')
                                && key != "Please visit www.eurid.eu for more info."
                            {
                                result.nameservers.push(key.to_string());
                            }
                        }
                        "technical" if key == "Organisation" && result.registrar.is_empty() => {
                            result.registrar = val.to_string();
                        }
                        _ => {}
                    }
                } else if current_section == "name servers"
                    && !trimmed.contains(':')
                    && trimmed != "Please visit www.eurid.eu for more info."
                {
                    result.nameservers.push(trimmed.to_string());
                }
            } else if trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.splitn(2, ':').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let val = parts[1];
                    if key == "Domain" {
                        result.domain_name = val.to_string();
                    }
                }
            }
        }
        if result.status.is_empty() {
            result.status.push("registered".to_string());
        }
    } else {
        for line in raw_data.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                continue;
            }
            let key = parts[0];
            let val = parts[1];
            if key.is_empty() || val.is_empty() {
                continue;
            }

            let key_lower = key.to_lowercase();
            if key_lower.contains("registrar") {
                result.registrar = val.to_string();
            } else if key_lower.contains("creation")
                || key_lower.contains("created")
                || key_lower.contains("registered")
            {
                if result.creation_date.is_empty() {
                    result.creation_date = val.to_string();
                }
            } else if key_lower.contains("expir") {
                if result.expiration_date.is_empty() {
                    result.expiration_date = val.to_string();
                }
            } else if key_lower.contains("updated") || key_lower.contains("modified") {
                if result.last_updated.is_empty() {
                    result.last_updated = val.to_string();
                }
            } else if key_lower.contains("status") {
                for s in val.split([',', ';']) {
                    let ts = s.trim().to_string();
                    if !ts.is_empty() && !result.status.contains(&ts) {
                        result.status.push(ts);
                    }
                }
            } else if key_lower.contains("name server") || key_lower.contains("nameserver") {
                let ns = val
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !ns.is_empty() && !result.nameservers.contains(&ns) {
                    result.nameservers.push(ns);
                }
            }
        }
    }

    result
}

// --- Geolocation IP Logic ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeolocationResponse {
    pub ip: String,
    pub version: String,
    pub city: Option<String>,
    pub region: Option<String>,
    #[serde(rename = "region_code")]
    pub region_code: Option<String>,
    #[serde(rename = "country_code")]
    pub country_code: Option<String>,
    #[serde(rename = "country_name")]
    pub country_name: Option<String>,
    pub postal: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    pub source: String,

    #[serde(default)]
    pub network: Option<String>,
    #[serde(default, rename = "continent_code")]
    pub continent_code: Option<String>,
    #[serde(default)]
    pub languages: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default, rename = "currency_name")]
    pub currency_name: Option<String>,
    #[serde(default, rename = "country_calling_code")]
    pub country_calling_code: Option<String>,
}

#[derive(Deserialize)]
struct IpApiComResponse {
    query: String,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    region: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    country: Option<String>,
    zip: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
    org: Option<String>,
    isp: Option<String>,
    #[serde(rename = "as")]
    asn: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoIsResponse {
    ip: String,
    #[serde(rename = "type")]
    ip_type: String,
    city: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    country_code: Option<String>,
    country: Option<String>,
    postal: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: IpWhoIsTimezone,
    connection: IpWhoIsConnection,
}

#[derive(Deserialize)]
struct IpWhoIsTimezone {
    id: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoIsConnection {
    asn: Option<serde_json::Value>,
    org: Option<String>,
}

async fn try_ip_lookup(client: &reqwest::Client, ip: &str) -> Result<GeolocationResponse, String> {
    let clean_ip = ip
        .replace(['[', ']'], "")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string();

    // 1. Try ipapi.co
    match client
        .get(format!("https://ipapi.co/{}/json/", clean_ip))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(mut data) = res.json::<GeolocationResponse>().await
                && !data.ip.is_empty()
            {
                data.source = "ipapi.co".to_string();
                return Ok(data);
            }
        }
        Err(e) => {
            tracing::warn!("ipapi.co failed: {}, trying next...", e);
        }
    }

    // 2. Try ip-api.com
    match client
        .get(format!("http://ip-api.com/json/{}", clean_ip))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(data) = res.json::<IpApiComResponse>().await {
                let version = if data.query.contains(':') {
                    "IPv6"
                } else {
                    "IPv4"
                };
                return Ok(GeolocationResponse {
                    ip: data.query,
                    version: version.to_string(),
                    city: data.city,
                    region: data.region_name,
                    region_code: data.region,
                    country_code: data.country_code,
                    country_name: data.country,
                    postal: data.zip,
                    latitude: data.lat,
                    longitude: data.lon,
                    timezone: data.timezone,
                    org: data.org.or(data.isp),
                    asn: data.asn,
                    source: "ip-api.com".to_string(),
                    network: None,
                    continent_code: None,
                    languages: None,
                    currency: None,
                    currency_name: None,
                    country_calling_code: None,
                });
            }
        }
        Err(e) => {
            tracing::warn!("ip-api.com failed: {}, trying next...", e);
        }
    }

    // 3. Try ipwho.is
    match client
        .get(format!("https://ipwho.is/{}", clean_ip))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(data) = res.json::<IpWhoIsResponse>().await {
                let asn_str = data.connection.asn.map(|val| match val {
                    serde_json::Value::Number(num) => format!("AS{}", num),
                    serde_json::Value::String(s) => s,
                    _ => "".to_string(),
                });
                return Ok(GeolocationResponse {
                    ip: data.ip,
                    version: data.ip_type,
                    city: data.city,
                    region: data.region,
                    region_code: data.region_code,
                    country_code: data.country_code,
                    country_name: data.country,
                    postal: data.postal,
                    latitude: data.latitude,
                    longitude: data.longitude,
                    timezone: data.timezone.id,
                    org: data.connection.org,
                    asn: asn_str,
                    source: "ipwho.is".to_string(),
                    network: None,
                    continent_code: None,
                    languages: None,
                    currency: None,
                    currency_name: None,
                    country_calling_code: None,
                });
            }
        }
        Err(e) => {
            tracing::error!("ipwho.is failed: {}", e);
        }
    }

    Err("All IP lookup services failed".to_string())
}

// --- ASN Lookup Logic ---

#[derive(Deserialize, Debug)]
pub struct RipeStatWhoisRecord {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RipeStatWhoisData {
    pub records: Option<Vec<Vec<RipeStatWhoisRecord>>>,
    pub authorities: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct RipeStatWhoisResponse {
    pub data: Option<RipeStatWhoisData>,
}

#[derive(Deserialize, Debug)]
pub struct RipeStatOverviewData {
    pub holder: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RipeStatOverviewResponse {
    pub data: Option<RipeStatOverviewData>,
}

#[derive(Deserialize, Debug)]
pub struct PeeringDbNet {
    pub website: Option<String>,
    pub info_ratio: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PeeringDbResponse {
    pub data: Option<Vec<PeeringDbNet>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RirAllocation {
    pub rir_name: String,
    pub date_allocated: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AsnData {
    pub asn: u32,
    pub name: String,
    pub description_short: String,
    pub country_code: Option<String>,
    pub website: String,
    pub email_contacts: Vec<String>,
    pub abuse_contacts: Vec<String>,
    pub owner_address: Vec<String>,
    pub rir_allocation: RirAllocation,
    pub traffic_ratio: Option<String>,
    pub date_updated: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AsnLookupResponse {
    pub data: AsnData,
}

async fn fetch_asn_data(
    client: &reqwest::Client,
    asn_number: &str,
) -> Result<AsnLookupResponse, String> {
    let asn_clean = asn_number.to_uppercase().replace("AS", "");
    let asn_val: u32 = asn_clean
        .parse()
        .map_err(|_| "Invalid ASN format".to_string())?;

    let whois_url = format!(
        "https://stat.ripe.net/data/whois/data.json?resource=AS{}",
        asn_clean
    );
    let overview_url = format!(
        "https://stat.ripe.net/data/as-overview/data.json?resource=AS{}",
        asn_clean
    );
    let peering_db_url = format!("https://www.peeringdb.com/api/net?asn={}", asn_clean);

    let whois_fut = client.get(&whois_url).send();
    let overview_fut = client.get(&overview_url).send();
    let peering_db_fut = client.get(&peering_db_url).send();

    let (whois_res, overview_res, peering_db_res) =
        tokio::join!(whois_fut, overview_fut, peering_db_fut);

    let whois_data: RipeStatWhoisResponse = match whois_res {
        Ok(res) => res
            .json()
            .await
            .unwrap_or(RipeStatWhoisResponse { data: None }),
        Err(_) => RipeStatWhoisResponse { data: None },
    };

    let overview_data: RipeStatOverviewResponse = match overview_res {
        Ok(res) => res
            .json()
            .await
            .unwrap_or(RipeStatOverviewResponse { data: None }),
        Err(_) => RipeStatOverviewResponse { data: None },
    };

    let peering_db_data: PeeringDbResponse = match peering_db_res {
        Ok(res) => res.json().await.unwrap_or(PeeringDbResponse { data: None }),
        Err(_) => PeeringDbResponse { data: None },
    };

    let peering_db_net = peering_db_data.data.as_ref().and_then(|v| v.first());

    let mut flat_records = Vec::new();
    if let Some(data) = whois_data.data.as_ref()
        && let Some(records) = data.records.as_ref()
    {
        for record_list in records {
            for r in record_list {
                flat_records.push(r);
            }
        }
    }

    let find_value = |key: &str| -> String {
        flat_records
            .iter()
            .find(|r| {
                r.key
                    .as_ref()
                    .map(|k| k.to_lowercase() == key.to_lowercase())
                    .unwrap_or(false)
            })
            .and_then(|r| r.value.clone())
            .unwrap_or_default()
    };

    let find_all_values = |key: &str| -> Vec<String> {
        flat_records
            .iter()
            .filter_map(|r| {
                let matches = r
                    .key
                    .as_ref()
                    .map(|k| k.to_lowercase() == key.to_lowercase())
                    .unwrap_or(false);
                if matches { r.value.clone() } else { None }
            })
            .collect()
    };

    let source = find_value("source");
    let auth = whois_data
        .data
        .as_ref()
        .and_then(|d| d.authorities.as_ref())
        .and_then(|a| a.first().cloned())
        .unwrap_or_default();

    let source_upper = if !source.is_empty() {
        source.to_uppercase()
    } else {
        auth.to_uppercase()
    };
    let is_arin = source_upper == "ARIN";

    let holder = overview_data
        .data
        .as_ref()
        .and_then(|d| d.holder.clone())
        .unwrap_or_default();
    let holder_parts: Vec<&str> = holder.split(" - ").collect();

    let name = if !holder_parts.is_empty() && !holder_parts[0].is_empty() {
        holder_parts[0].to_string()
    } else {
        let as_name = find_value("as-name");
        if !as_name.is_empty() {
            as_name
        } else {
            let as_name_caps = find_value("ASName");
            if !as_name_caps.is_empty() {
                as_name_caps
            } else {
                find_value("aut-num")
            }
        }
    };

    let mut country_code = {
        let cc = find_value("country");
        if cc.is_empty() { None } else { Some(cc) }
    };
    if country_code.is_none() && !holder.is_empty() {
        let re_cc = regex::Regex::new(r",\s*([A-Z]{2})$").unwrap();
        if let Some(caps) = re_cc.captures(&holder) {
            country_code = Some(caps.get(1).unwrap().as_str().to_string());
        }
    }

    let mut descriptions = find_all_values("descr");
    let org_name = find_value("OrgName");
    let description = if !descriptions.is_empty() && !descriptions[0].is_empty() {
        descriptions.remove(0)
    } else if !org_name.is_empty() {
        org_name
    } else if holder_parts.len() > 1 {
        holder_parts[1..].join(" - ")
    } else {
        String::new()
    };

    let mut email_contacts = find_all_values("e-mail");
    if email_contacts.is_empty() {
        let tech_email = find_all_values("OrgTechEmail");
        let noc_email = find_all_values("OrgNOCEmail");
        email_contacts = tech_email;
        for email in noc_email {
            if !email_contacts.contains(&email) {
                email_contacts.push(email);
            }
        }
    }
    if email_contacts.is_empty() {
        let tech_c = find_all_values("tech-c");
        let admin_c = find_all_values("admin-c");
        email_contacts = tech_c;
        for c in admin_c {
            if !email_contacts.contains(&c) {
                email_contacts.push(c);
            }
        }
    }

    let mut abuse_contacts = find_all_values("abuse-mailbox");
    if abuse_contacts.is_empty() {
        abuse_contacts = find_all_values("OrgAbuseEmail");
    }
    if abuse_contacts.is_empty() {
        abuse_contacts = find_all_values("abuse-c");
    }

    let remarks = find_all_values("remarks");
    let mut owner_address = find_all_values("address");
    if owner_address.is_empty() && is_arin {
        let street = find_value("Address");
        let city = find_value("City");
        let state = find_value("StateProv");
        let postal = find_value("PostalCode");
        let country = find_value("Country");

        let mut addr_parts = Vec::new();
        if !street.is_empty() {
            addr_parts.push(street);
        }
        let mut city_state_zip = Vec::new();
        if !city.is_empty() {
            city_state_zip.push(city);
        }
        if !state.is_empty() {
            city_state_zip.push(state);
        }
        if !postal.is_empty() {
            city_state_zip.push(postal);
        }
        if !city_state_zip.is_empty() {
            addr_parts.push(city_state_zip.join(", "));
        }
        if !country.is_empty() {
            addr_parts.push(country);
        }
        owner_address = addr_parts;
    }
    if owner_address.is_empty() {
        owner_address = remarks
            .iter()
            .filter(|r| !r.contains("http"))
            .cloned()
            .collect();
    }

    let rir_name = match source_upper.as_str() {
        "RIPE" => "RIPE NCC".to_string(),
        "ARIN" => "ARIN".to_string(),
        "APNIC" => "APNIC".to_string(),
        "LACNIC" => "LACNIC".to_string(),
        "AFRINIC" => "AFRINIC".to_string(),
        "RADB" => "RADB".to_string(),
        _ => {
            if !source_upper.is_empty() {
                source_upper
            } else {
                "Unknown".to_string()
            }
        }
    };

    let created = {
        let c = find_value("created");
        if !c.is_empty() {
            Some(c)
        } else {
            let r = find_value("RegDate");
            if !r.is_empty() {
                Some(r)
            } else {
                let rd = find_value("reg-date");
                if !rd.is_empty() { Some(rd) } else { None }
            }
        }
    };

    let last_modified = {
        let lm = find_value("last-modified");
        if !lm.is_empty() {
            Some(lm)
        } else {
            let u = find_value("Updated");
            if !u.is_empty() {
                Some(u)
            } else {
                let ch = find_value("changed");
                if !ch.is_empty() { Some(ch) } else { None }
            }
        }
    };

    let website = peering_db_net
        .and_then(|net| net.website.clone())
        .unwrap_or_else(|| {
            remarks
                .iter()
                .find(|r| r.contains("http"))
                .cloned()
                .unwrap_or_default()
        });

    let traffic_ratio = peering_db_net.and_then(|net| net.info_ratio.clone());

    Ok(AsnLookupResponse {
        data: AsnData {
            asn: asn_val,
            name,
            description_short: description,
            country_code,
            website,
            abuse_contacts: if !abuse_contacts.is_empty() {
                abuse_contacts
            } else {
                email_contacts.clone()
            },
            email_contacts,
            owner_address,
            rir_allocation: RirAllocation {
                rir_name,
                date_allocated: created,
            },
            traffic_ratio,
            date_updated: last_modified,
        },
    })
}

async fn send_notification(
    query: &str,
    query_type: &str,
    config: &AppConfig,
    client: &reqwest::Client,
) {
    let url = match &config.apprise_url {
        Some(u) => u,
        None => return,
    };

    let message = config
        .apprise_message
        .replace("{query}", query)
        .replace("{query_type}", query_type);

    let body = serde_json::json!({
        "urls": url,
        "body": message,
        "title": format!("{} Notification", config.site_title),
    });

    tracing::info!("Sending notification via Apprise to URL: {}", url);
    match client
        .post("https://api.apprise.io/notify")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                tracing::info!("Notification sent successfully.");
            } else {
                tracing::error!("Apprise API returned error status: {:?}", resp.status());
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to Apprise API: {}", e);
        }
    }
}

// --- API Router Handler ---

async fn handle_lookup(
    Path(query): Path<String>,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Check PIN if configured
    if let Some(ref pin) = state.config.pin {
        let cookie_pin = headers
            .get(axum::http::header::COOKIE)
            .and_then(|c| c.to_str().ok())
            .and_then(|c_str| {
                c_str
                    .split(';')
                    .find(|s| s.trim().starts_with("RUSTWHO_PIN="))
                    .and_then(|s| s.split('=').nth(1))
                    .map(|s| s.trim().to_string())
            });
        let header_pin = headers
            .get("x-pin")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let provided_pin = cookie_pin.or(header_pin);

        let authenticated = match provided_pin {
            Some(prov) => safe_compare(&prov, pin),
            None => false,
        };

        if !authenticated {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid or missing PIN"
                })),
            )
                .into_response();
        }
    }

    let query_type = detect_query_type(&query);

    match query_type {
        "whois" => {
            match whois_lookup(&query).await {
                Ok(raw_data) => {
                    let parsed = parse_whois_data(&raw_data, &query).await;

                    // Match the JSON structure from Node.js:
                    let ldh_name = parsed.domain_name.clone();
                    let response_data = serde_json::json!({
                        "ldhName": ldh_name,
                        "handle": query,
                        "status": parsed.status,
                        "ipAddresses": parsed.ip_addresses,
                        "events": [
                            {
                                "eventAction": "registration",
                                "eventDate": parsed.creation_date
                            },
                            {
                                "eventAction": "expiration",
                                "eventDate": parsed.expiration_date
                            },
                            {
                                "eventAction": "lastChanged",
                                "eventDate": parsed.last_updated
                            }
                        ],
                        "nameservers": parsed.nameservers.into_iter().map(|ns| serde_json::json!({ "ldhName": ns })).collect::<Vec<_>>(),
                        "entities": [{
                            "roles": ["registrar"],
                            "vcardArray": [
                                "vcard",
                                [
                                    ["version", {}, "text", "4.0"],
                                    ["fn", {}, "text", parsed.registrar],
                                    ["email", {}, "text", ""]
                                ]
                            ]
                        }]
                    });

                    let config_clone = state.config.clone();
                    let client_clone = state.client.clone();
                    let query_clone = query.clone();
                    tokio::spawn(async move {
                        send_notification(&query_clone, "whois", &config_clone, &client_clone)
                            .await;
                    });

                    Json(serde_json::json!({
                        "type": "whois",
                        "data": response_data
                    }))
                    .into_response()
                }
                Err(e) => {
                    tracing::error!("WHOIS lookup error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Error fetching WHOIS data",
                            "message": e
                        })),
                    )
                        .into_response()
                }
            }
        }
        "ip" => match try_ip_lookup(&state.client, &query).await {
            Ok(ip_data) => {
                let config_clone = state.config.clone();
                let client_clone = state.client.clone();
                let query_clone = query.clone();
                tokio::spawn(async move {
                    send_notification(&query_clone, "ip", &config_clone, &client_clone).await;
                });

                Json(serde_json::json!({
                    "type": "ip",
                    "data": ip_data
                }))
                .into_response()
            }
            Err(e) => {
                tracing::error!("IP lookup error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Error fetching IP data",
                        "message": e
                    })),
                )
                    .into_response()
            }
        },
        "asn" => {
            let asn_number = query.to_uppercase().replace("AS", "");
            match fetch_asn_data(&state.client, &asn_number).await {
                Ok(asn_data) => {
                    let config_clone = state.config.clone();
                    let client_clone = state.client.clone();
                    let query_clone = query.clone();
                    tokio::spawn(async move {
                        send_notification(&query_clone, "asn", &config_clone, &client_clone).await;
                    });

                    Json(serde_json::json!({
                        "type": "asn",
                        "data": asn_data.data
                    }))
                    .into_response()
                }
                Err(e) => {
                    tracing::error!("ASN lookup error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Error fetching ASN data",
                            "message": e
                        })),
                    )
                        .into_response()
                }
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid input",
                "message": "Please enter a valid domain name, IP address, or ASN number"
            })),
        )
            .into_response(),
    }
}

fn safe_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[derive(Deserialize)]
struct VerifyPinPayload {
    pin: Option<String>,
}

async fn verify_pin(
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let Some(ref config_pin) = state.config.pin else {
        let mut headers = axum::http::header::HeaderMap::new();
        headers.insert(
            axum::http::header::SET_COOKIE,
            axum::http::header::HeaderValue::from_static(
                "RUSTWHO_PIN=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
            ),
        );
        return (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response();
    };

    let pin_str = payload.pin.as_deref().unwrap_or("").trim();
    if pin_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "PIN is required." })),
        )
            .into_response();
    }

    if safe_compare(pin_str, config_pin) {
        let mut headers = axum::http::header::HeaderMap::new();
        headers.insert(
            axum::http::header::SET_COOKIE,
            axum::http::header::HeaderValue::from_str(&format!(
                "RUSTWHO_PIN={}; Path=/; HttpOnly; SameSite=Lax",
                pin_str
            ))
            .unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "error": "Invalid PIN" })),
        )
            .into_response()
    }
}

async fn logout() -> impl IntoResponse {
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        axum::http::header::HeaderValue::from_static(
            "RUSTWHO_PIN=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        ),
    );
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true })),
    )
        .into_response()
}

async fn auth_check(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(ref pin) = state.config.pin {
        let cookie_pin = headers
            .get(axum::http::header::COOKIE)
            .and_then(|c| c.to_str().ok())
            .and_then(|c_str| {
                c_str
                    .split(';')
                    .find(|s| s.trim().starts_with("RUSTWHO_PIN="))
                    .and_then(|s| s.split('=').nth(1))
                    .map(|s| s.trim().to_string())
            });
        let header_pin = headers
            .get("x-pin")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let provided_pin = cookie_pin.or(header_pin);

        let authenticated = match provided_pin {
            Some(prov) => safe_compare(&prov, pin),
            None => false,
        };

        if !authenticated {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    StatusCode::OK.into_response()
}
