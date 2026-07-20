pub mod parser;
#[cfg(test)]
mod tests;

use crate::dns::IpAddresses;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub use parser::parse_whois_data;

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

static RE_REFER: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)^\s*(refer|whois|whois\s+server|registrar\s+whois\s+server)\s*:\s*([a-z0-9\-\._]+)\s*$").unwrap()
});

/// Returns `true` if the IP is in any range that should never be the target
/// of an outbound WHOIS connection from this server.
pub fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_unspecified()
                || v4.is_broadcast()
                // CGNAT / shared address space (RFC 6598) — comments elsewhere
                // claimed this was blocked; is_private() alone does not cover it.
                || matches!(v4.octets(), [100, 64..=127, ..])
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                || (v6.segments()[0] & 0xff00) == 0xff00
        }
    }
}

/// Resolve a WHOIS host to a concrete SocketAddr after validating that
/// every resolved IP is public.
pub async fn resolve_public_whois_addr(target: &str) -> Result<std::net::SocketAddr, String> {
    // Strip optional :43 suffix.
    let (host, port) = match target.rsplit_once(':') {
        Some((h, p)) if p.parse::<u16>().is_ok() => (h, p.parse::<u16>().unwrap()),
        _ => (target, 43u16),
    };

    // If `host` is a literal IP, validate it directly.
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(ip) {
            return Err(format!(
                "refusing WHOIS connection: {ip} is private/internal"
            ));
        }
        return Ok(std::net::SocketAddr::new(ip, port));
    }

    // Otherwise resolve via DNS and reject if any resolved IP is private.
    let addrs = tokio::net::lookup_host(format!("{host}:{port}"))
        .await
        .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(format!("DNS resolution returned no addresses for {host}"));
    }
    for addr in &addrs {
        if is_private_ip(addr.ip()) {
            return Err(format!(
                "refusing WHOIS connection to {host}: resolved to private/internal IP {}",
                addr.ip()
            ));
        }
    }
    Ok(addrs[0])
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

        // SSRF defense: resolve to a concrete public IP before opening
        // any socket.
        let _resolved = resolve_public_whois_addr(&server).await?;

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
    let addr = resolve_public_whois_addr(server).await?;
    let mut stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr))
        .await
        .map_err(|_| format!("Connection timeout to {}", server))?
        .map_err(|e| format!("Failed to connect to {}: {}", server, e))?;

    let peer_ip = stream.peer_addr().ok().map(|a| a.ip());
    if let Some(ip) = peer_ip {
        if is_private_ip(ip) {
            return Err(format!(
                "refusing WHOIS connection: peer {ip} is private/internal"
            ));
        }
    }

    // WHOIS is a line-oriented protocol. Reject CR/LF (and other controls) so a
    // malicious query cannot inject extra WHOIS commands on the wire.
    if query.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err("refusing WHOIS query: control characters not allowed".into());
    }
    if query.len() > 253 {
        return Err("refusing WHOIS query: too long".into());
    }

    stream
        .write_all(format!("{}\r\n", query).as_bytes())
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
