use serde::{Deserialize, Serialize};

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
