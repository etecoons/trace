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
        flat_records.iter()
            .find(|r| r.key.as_ref().map_or(false, |k| k.to_lowercase() == key.to_lowercase()))
            .and_then(|r| r.value.clone()).unwrap_or_default()
    };

    let find_all_values = |key: &str| -> Vec<String> {
        flat_records.iter()
            .filter(|r| r.key.as_ref().map_or(false, |k| k.to_lowercase() == key.to_lowercase()))
            .filter_map(|r| r.value.clone()).collect()
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
