pub mod helpers;
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

    // Perform rate limiting concurrently inside the async tasks to prevent latency accumulation
    let whois_fut = async {
        limiter.acquire("ripe_stat").await;
        client.get(&whois_url).send().await
    };
    let overview_fut = async {
        limiter.acquire("ripe_overview").await;
        client.get(&overview_url).send().await
    };
    let peering_db_fut = async {
        limiter.acquire("peeringdb").await;
        client.get(&peering_db_url).send().await
    };

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
