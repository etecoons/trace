use crate::rate_limit::UpstreamRateLimiter;
use crate::routes::whois::is_private_ip;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

pub mod types;
pub use types::{GeolocationResponse, IpApiComResponse, IpWhoIsResponse};

/// Validate that a user-supplied IP is something we want to forward to
/// third-party geolocation services. Rejects loopback, RFC1918, CGNAT,
/// link-local, multicast, unspecified, broadcast, and the IPv6 equivalents.
fn parse_public_ip(s: &str) -> Result<IpAddr, String> {
    let clean = s
        .replace(['[', ']'], "")
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let ip = IpAddr::from_str(&clean).map_err(|e| format!("invalid IP: {e}"))?;
    if is_private_ip(ip) {
        return Err(format!("refusing to look up private/internal IP: {ip}"));
    }
    Ok(ip)
}

pub async fn try_ip_lookup(
    client: &reqwest::Client,
    limiter: &Arc<UpstreamRateLimiter>,
    ip: &str,
) -> Result<GeolocationResponse, String> {
    // Validate before we burn API quota on a private IP and before we
    // potentially use the user's input as a path component in a URL.
    let parsed_ip = parse_public_ip(ip)?;
    let clean_ip = parsed_ip.to_string();

    // 1. Try ipapi.co
    limiter.acquire("ipapi");
    match client
        .get(format!("https://ipapi.co/{}/json/", clean_ip))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(mut data) = res.json::<GeolocationResponse>().await {
                if !data.ip.is_empty() {
                    data.source = "ipapi.co".to_string();
                    return Ok(data);
                }
            }
        }
        Err(e) => {
            tracing::warn!("ipapi.co failed: {}, trying next...", e);
        }
    }

    // 2. Try ip-api.com
    // NOTE: previously this was over HTTP, leaking the user's IP lookup
    // in cleartext. Use HTTPS to match the rest of the app (which insists
    // on rustls via reqwest's `rustls-tls` feature).
    limiter.acquire("ipapi_com");
    match client
        .get(format!("https://ip-api.com/json/{}", clean_ip))
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
    limiter.acquire("ipwhois");
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
