pub mod parser;

use crate::asn_types::*;
use crate::rate_limit::UpstreamRateLimiter;
use std::sync::Arc;

pub async fn fetch_asn_data(
    client: &reqwest::Client,
    limiter: &Arc<UpstreamRateLimiter>,
    asn_number: &str,
) -> Result<AsnLookupResponse, String> {
    let asn_clean = asn_number.to_uppercase().replace("AS", "");
    let asn_val: u32 = asn_clean
        .parse()
        .map_err(|_| "Invalid ASN format".to_string())?;

    let whois_url = format!(
        "https://stat.ripe.net/data/whois/data.json?resource=AS{}",
        asn_clean
    );
    let overview_url = format!(
        "https://stat.ripe.net/data/as-overview/data.json?resource=AS{}",
        asn_clean
    );
    let peering_db_url = format!("https://www.peeringdb.com/api/net?asn={}", asn_clean);

    // Per-upstream throttle. The three calls below all run in parallel
    // via `tokio::join!`, but each acquires its own limiter slot before
    // firing so two back-to-back ASN lookups don't trip RIPE's 1 req/s
    // limit.
    limiter.acquire("ripe_stat");
    limiter.acquire("ripe_overview");
    limiter.acquire("peeringdb");

    let whois_fut = client.get(&whois_url).send();
    let overview_fut = client.get(&overview_url).send();
    let peering_db_fut = client.get(&peering_db_url).send();

    let (whois_res, overview_res, peering_db_res) =
        tokio::join!(whois_fut, overview_fut, peering_db_fut);

    let whois_data: RipeStatWhoisResponse = match whois_res {
        Ok(res) => res
            .json()
            .await
            .unwrap_or(RipeStatWhoisResponse { data: None }),
        Err(_) => RipeStatWhoisResponse { data: None },
    };

    let overview_data: RipeStatOverviewResponse = match overview_res {
        Ok(res) => res
            .json()
            .await
            .unwrap_or(RipeStatOverviewResponse { data: None }),
        Err(_) => RipeStatOverviewResponse { data: None },
    };

    let peering_db_data: PeeringDbResponse = match peering_db_res {
        Ok(res) => res.json().await.unwrap_or(PeeringDbResponse { data: None }),
        Err(_) => PeeringDbResponse { data: None },
    };

    parser::parse_raw_responses(asn_val, whois_data, overview_data, peering_db_data)
}
