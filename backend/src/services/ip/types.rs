use serde::{Deserialize, Serialize};

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
pub struct IpApiComResponse {
    pub query: String,
    pub city: Option<String>,
    #[serde(rename = "regionName")]
    pub region_name: Option<String>,
    pub region: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub timezone: Option<String>,
    pub org: Option<String>,
    pub isp: Option<String>,
    #[serde(rename = "as")]
    pub asn: Option<String>,
}

#[derive(Deserialize)]
pub struct IpWhoIsResponse {
    pub ip: String,
    #[serde(rename = "type")]
    pub ip_type: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub postal: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: IpWhoIsTimezone,
    pub connection: IpWhoIsConnection,
}

#[derive(Deserialize)]
pub struct IpWhoIsTimezone {
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct IpWhoIsConnection {
    pub asn: Option<serde_json::Value>,
    pub org: Option<String>,
}
