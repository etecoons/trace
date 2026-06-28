use serde::{Deserialize, Serialize};

pub use shared_core::i18n::Language;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WhoisEvent {
    pub event_action: String,
    pub event_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WhoisNameserver {
    pub ldh_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhoisEntity {
    pub roles: Vec<String>,
    #[serde(rename = "vcardArray")]
    pub vcard_array: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct IpAddresses {
    pub v4: Vec<String>,
    pub v6: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WhoisData {
    pub ldh_name: String,
    pub handle: String,
    pub status: Vec<String>,
    pub ip_addresses: IpAddresses,
    pub events: Vec<WhoisEvent>,
    pub nameservers: Vec<WhoisNameserver>,
    pub entities: Vec<WhoisEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpData {
    pub ip: String,
    pub version: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub region_code: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub postal: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    pub source: String,
    pub network: Option<String>,
    pub continent_code: Option<String>,
    pub languages: Option<String>,
    pub currency: Option<String>,
    pub currency_name: Option<String>,
    pub country_calling_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RirAllocation {
    pub rir_name: String,
    pub date_allocated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum LookupResponse {
    Whois(WhoisData),
    Ip(IpData),
    Asn(AsnData),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub is_error: bool,
}

pub enum Msg {
    UpdateQuery(String),
    PerformLookup,
    LookupIP(String),
    LookupSuccess(Box<LookupResponse>),
    LookupFailure(String),
    LoadConfig(serde_json::Value),
    PinInputChanged(String),
    VerifyPin,
    VerifyPinSuccess,
    VerifyPinFailure(String),
    Logout,
    LogoutSuccess,
    ToggleTheme,
    SwitchLanguage(Language),
    ShowToast(String, bool),
    DismissToast(usize),
    PrintPage,
    Nothing,
}
