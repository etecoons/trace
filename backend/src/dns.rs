use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct IpAddresses {
    pub v4: Vec<String>,
    pub v6: Vec<String>,
}

pub async fn resolve_dns(domain: &str) -> IpAddresses {
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
