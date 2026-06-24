use std::net::{IpAddr, SocketAddr};
use axum::http::HeaderMap;

pub fn safe_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ipv6) => {
            if let Some(ipv4) = ipv6.to_ipv4_mapped() {
                IpAddr::V4(ipv4)
            } else {
                IpAddr::V6(ipv6)
            }
        }
        IpAddr::V4(ipv4) => IpAddr::V4(ipv4),
    }
}

pub fn first_ip_from_x_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    let xff = headers.get("x-forwarded-for")?.to_str().ok()?;
    let first = xff.split(',').next()?.trim();
    first.parse::<IpAddr>().ok().map(normalize_ip)
}

pub fn is_trusted_proxy(remote_addr: IpAddr, trusted_list: &[ipnet::IpNet]) -> bool {
    let normalized = normalize_ip(remote_addr);
    for net in trusted_list {
        if net.contains(&normalized) {
            return true;
        }
    }
    false
}

pub fn get_client_ip(
    headers: &HeaderMap,
    connect_info: SocketAddr,
    trust_proxy: bool,
    trusted_proxies: &[ipnet::IpNet],
) -> IpAddr {
    let socket_ip = normalize_ip(connect_info.ip());

    if !trust_proxy {
        return socket_ip;
    }

    if !trusted_proxies.is_empty() {
        if is_trusted_proxy(socket_ip, trusted_proxies) {
            if let Some(xff_ip) = first_ip_from_x_forwarded_for(headers) {
                return xff_ip;
            }
        }
        return socket_ip;
    }

    if let Some(xff_ip) = first_ip_from_x_forwarded_for(headers) {
        xff_ip
    } else {
        socket_ip
    }
}
