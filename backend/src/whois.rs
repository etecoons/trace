use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};
use crate::dns::{IpAddresses, resolve_dns};

static RE_REFER: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)^\s*(refer|whois|whois\s+server|registrar\s+whois\s+server)\s*:\s*([a-z0-9\-\._]+)\s*$").unwrap()
});

static RE_IPV4: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b").unwrap()
});

static RE_IPV6: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(?:[0-9a-f]{1,4}:){1,7}(?:[0-9a-f]{1,4}|:)|(?:::[0-9a-f]{1,4})\b").unwrap()
});

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedWhoisData {
    pub domain_name: String,
    pub registrar: String,
    pub creation_date: String,
    pub expiration_date: String,
    pub last_updated: String,
    pub status: Vec<String>,
    pub nameservers: Vec<String>,
    pub ip_addresses: IpAddresses,
    pub raw: String,
}

pub async fn whois_lookup(query: &str) -> Result<String, String> {
    let mut server = if query.to_lowercase().ends_with(".eu") {
        "whois.eu".to_string()
    } else {
        "whois.iana.org".to_string()
    };
    let mut visited = HashSet::new();

    for _ in 0..4 {
        if visited.contains(&server) {
            break;
        }
        visited.insert(server.clone());

        tracing::info!("Querying WHOIS server {} for {}", server, query);
        let raw_data = query_whois_server(&server, query).await?;

        if let Some(next_server) = find_redirect_server(&raw_data) {
            server = next_server;
        } else {
            return Ok(raw_data);
        }
    }
    Err("Too many WHOIS redirects".to_string())
}

async fn query_whois_server(server: &str, query: &str) -> Result<String, String> {
    let addr = if server.contains(':') { server.to_string() } else { format!("{}:43", server) };
    let mut stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("Connection timeout to {}", server))?
        .map_err(|e| format!("Failed to connect to {}: {}", server, e))?;

    stream.write_all(format!("{}\r\n", query).as_bytes()).await
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

    tokio::time::timeout(Duration::from_secs(10), read_future).await
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

pub fn extract_ips_from_raw(raw_data: &str) -> IpAddresses {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    for line in raw_data.lines() {
        let lower = line.to_lowercase();
        if lower.contains("ip address") || lower.contains("a record") || lower.contains("aaaa record")
            || lower.contains("addresses") || lower.contains("host") || lower.contains("dns")
        {
            for cap in RE_IPV4.find_iter(line) {
                let ip = cap.as_str().to_string();
                if !v4.contains(&ip) { v4.push(ip); }
            }
            for cap in RE_IPV6.find_iter(line) {
                let ip = cap.as_str().to_string();
                if !v6.contains(&ip) { v6.push(ip); }
            }
        }
    }
    IpAddresses { v4, v6 }
}

pub async fn parse_whois_data(raw_data: &str, domain: &str) -> ParsedWhoisData {
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
        if !dns_ips.v4.contains(&ip) { dns_ips.v4.push(ip); }
    }
    for ip in raw_ips.v6 {
        if !dns_ips.v6.contains(&ip) { dns_ips.v6.push(ip); }
    }
    result.ip_addresses = dns_ips;

    if domain.to_lowercase().ends_with(".eu") {
        parse_eu_whois(raw_data, &mut result);
    } else {
        parse_generic_whois(raw_data, &mut result);
    }
    result
}

fn parse_eu_whois(raw_data: &str, result: &mut ParsedWhoisData) {
    let mut current_section = String::new();
    for line in raw_data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') { continue; }
        if let Some(stripped) = trimmed.strip_suffix(':') {
            current_section = stripped.to_lowercase();
            continue;
        }
        if line.starts_with("        ") {
            let parts: Vec<&str> = trimmed.splitn(2, ':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let (key, val) = (parts[0], parts[1]);
                match current_section.as_str() {
                    "registrar" if key == "Name" => result.registrar = val.to_string(),
                    "name servers" if !key.contains(':') && key != "Please visit www.eurid.eu for more info." => {
                        result.nameservers.push(key.to_string());
                    }
                    "technical" if key == "Organisation" && result.registrar.is_empty() => {
                        result.registrar = val.to_string();
                    }
                    _ => {}
                }
            } else if current_section == "name servers" && !trimmed.contains(':')
                && trimmed != "Please visit www.eurid.eu for more info."
            {
                result.nameservers.push(trimmed.to_string());
            }
        } else if trimmed.contains(':') {
            let parts: Vec<&str> = trimmed.splitn(2, ':').map(|s| s.trim()).collect();
            if parts.len() == 2 && parts[0] == "Domain" {
                result.domain_name = parts[1].to_string();
            }
        }
    }
    if result.status.is_empty() {
        result.status.push("registered".to_string());
    }
}

fn parse_generic_whois(raw_data: &str, result: &mut ParsedWhoisData) {
    for line in raw_data.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').map(|s| s.trim()).collect();
        if parts.len() < 2 { continue; }
        let (key, val) = (parts[0], parts[1]);
        if key.is_empty() || val.is_empty() { continue; }
        let key_lower = key.to_lowercase();
        if key_lower.contains("registrar") {
            result.registrar = val.to_string();
        } else if key_lower.contains("creation") || key_lower.contains("created") || key_lower.contains("registered") {
            if result.creation_date.is_empty() { result.creation_date = val.to_string(); }
        } else if key_lower.contains("expir") {
            if result.expiration_date.is_empty() { result.expiration_date = val.to_string(); }
        } else if key_lower.contains("updated") || key_lower.contains("modified") {
            if result.last_updated.is_empty() { result.last_updated = val.to_string(); }
        } else if key_lower.contains("status") {
            for s in val.split([',', ';']) {
                let ts = s.trim().to_string();
                if !ts.is_empty() && !result.status.contains(&ts) { result.status.push(ts); }
            }
        } else if key_lower.contains("name server") || key_lower.contains("nameserver") {
            let ns = val.split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .next().unwrap_or("").trim().to_string();
            if !ns.is_empty() && !result.nameservers.contains(&ns) { result.nameservers.push(ns); }
        }
    }
}
