use super::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[test]
fn rejects_ipv4_loopback() {
    assert!(is_private_ip("127.0.0.1".parse().unwrap()));
    assert!(is_private_ip("127.255.255.254".parse().unwrap()));
}

#[test]
fn rejects_ipv4_rfc1918() {
    assert!(is_private_ip("10.0.0.1".parse().unwrap()));
    assert!(is_private_ip("172.16.0.1".parse().unwrap()));
    assert!(is_private_ip("192.168.1.1".parse().unwrap()));
}

#[test]
fn rejects_ipv4_link_local() {
    assert!(is_private_ip("169.254.169.254".parse().unwrap()));
}

#[test]
fn rejects_ipv4_multicast_and_unspecified() {
    assert!(is_private_ip("224.0.0.1".parse().unwrap()));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::BROADCAST)));
}

#[test]
fn accepts_public_ipv4() {
    assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
    assert!(!is_private_ip("1.1.1.1".parse().unwrap()));
}

#[test]
fn rejects_ipv6_loopback_and_unspecified() {
    assert!(is_private_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(is_private_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
}

#[test]
fn rejects_ipv6_unique_local_fc00() {
    // fc00::/7
    assert!(is_private_ip("fc00::1".parse().unwrap()));
    assert!(is_private_ip("fd00::1".parse().unwrap()));
}

#[test]
fn rejects_ipv6_link_local_fe80() {
    // fe80::/10
    assert!(is_private_ip("fe80::1".parse().unwrap()));
}

#[test]
fn rejects_ipv6_multicast() {
    // ff00::/8
    assert!(is_private_ip("ff02::1".parse().unwrap()));
}

#[tokio::test]
async fn resolve_public_rejects_literal_private_ip() {
    let err = resolve_public_whois_addr("127.0.0.1:43").await.unwrap_err();
    assert!(err.contains("private"), "got: {err}");
}

#[tokio::test]
async fn resolve_public_rejects_literal_rfc1918() {
    let err = resolve_public_whois_addr("10.0.0.1:43").await.unwrap_err();
    assert!(err.contains("private"), "got: {err}");
}

#[tokio::test]
async fn resolve_public_accepts_public_hostname() {
    // whois.iana.org is a stable public WHOIS host.
    let addr = resolve_public_whois_addr("whois.iana.org").await.unwrap();
    assert_eq!(addr.port(), 43);
    assert!(!is_private_ip(addr.ip()));
}

#[tokio::test]
async fn resolve_public_accepts_hostname_with_port() {
    // host:port syntax should be accepted.
    let addr = resolve_public_whois_addr("whois.iana.org:43")
        .await
        .unwrap();
    assert_eq!(addr.port(), 43);
}

#[tokio::test]
async fn resolve_public_rejects_unresolvable_host() {
    // A name that won't resolve should error rather than silently
    // connecting.
    let result = resolve_public_whois_addr("nonexistent.invalid").await;
    assert!(result.is_err());
}
