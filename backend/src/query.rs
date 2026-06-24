pub fn detect_query_type(query: &str) -> &'static str {
    let clean = query.replace(['[', ']'], "");

    // ASN pattern
    let re_asn = regex::Regex::new(r"(?i)^(AS)?\d+$").unwrap();
    if re_asn.is_match(&clean) {
        return "asn";
    }

    // IPv4 pattern
    let re_ipv4 = regex::Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?:\/\d{1,2})?$").unwrap();
    if re_ipv4.is_match(&clean) {
        return "ip";
    }

    // IPv6 pattern
    if clean.contains(':') {
        if clean.parse::<std::net::IpAddr>().is_ok() {
            return "ip";
        }
        let clean_no_cidr = clean.split('/').next().unwrap_or("");
        if clean_no_cidr.parse::<std::net::IpAddr>().is_ok() {
            return "ip";
        }
    }

    // Domain pattern
    if clean.contains('.') {
        return "whois";
    }

    "unknown"
}
