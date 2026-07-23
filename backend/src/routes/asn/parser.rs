use super::helpers::{build_arin_address, extract_created, extract_last_modified};
use crate::asn_types::*;

pub fn parse_raw_responses(
    asn_val: u32,
    whois_data: RipeStatWhoisResponse,
    overview_data: RipeStatOverviewResponse,
    peering_db_data: PeeringDbResponse,
) -> Result<AsnLookupResponse, String> {
    let peering_db_net = peering_db_data.data.as_ref().and_then(|v| v.first());

    let mut flat_records = Vec::new();
    if let Some(records) = whois_data.data.as_ref().and_then(|d| d.records.as_ref()) {
        for record_list in records {
            flat_records.extend(record_list);
        }
    }

    let find_value = |key: &str| -> String {
        flat_records
            .iter()
            .find(|r| {
                r.key
                    .as_ref()
                    .map_or(false, |k| k.to_lowercase() == key.to_lowercase())
            })
            .and_then(|r| r.value.clone())
            .unwrap_or_default()
    };

    let find_all_values = |key: &str| -> Vec<String> {
        flat_records
            .iter()
            .filter(|r| {
                r.key
                    .as_ref()
                    .map_or(false, |k| k.to_lowercase() == key.to_lowercase())
            })
            .filter_map(|r| r.value.clone())
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
        if let Ok(re_cc) = regex::Regex::new(r",\s*([A-Z]{2})$") {
            if let Some(caps) = re_cc.captures(&holder) {
                if let Some(m) = caps.get(1) {
                    country_code = Some(m.as_str().to_string());
                }
            }
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
        owner_address = build_arin_address(&find_value);
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

    let created = extract_created(&find_value);
    let last_modified = extract_last_modified(&find_value);

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
