use serde::{Deserialize, Serialize};
use std::time::Duration;

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

pub async fn try_ip_lookup(
    client: &reqwest::Client,
    ip: &str,
) -> Result<GeolocationResponse, String> {
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
