use std::sync::LazyLock;

static RE_ASN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)^(AS)?\d+$")
        .unwrap_or_else(|_| regex::Regex::new("").unwrap_or_else(|_| unreachable!()))
});

static RE_IPV4: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?:\/\d{1,2})?$")
        .unwrap_or_else(|_| regex::Regex::new("").unwrap_or_else(|_| unreachable!()))
});

pub fn detect_query_type(query: &str) -> &'static str {
    let clean = query.replace(['[', ']'], "");

    if RE_ASN.is_match(&clean) {
        return "asn";
    }

    if RE_IPV4.is_match(&clean) {
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
