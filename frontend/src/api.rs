use crate::types::LookupResponse;
use gloo_net::http::Request;

pub async fn fetch_lookup(query: &str) -> Result<LookupResponse, String> {
    let url = format!("/api/lookup/{}", query);
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        if let Ok(err_json) = resp.json::<serde_json::Value>().await {
            if let Some(err_msg) = err_json.get("message").and_then(|v| v.as_str()) {
                return Err(err_msg.to_string());
            }
            if let Some(err_title) = err_json.get("error").and_then(|v| v.as_str()) {
                return Err(err_title.to_string());
            }
        }
        return Err(format!("Server returned status {}", resp.status()));
    }

    let lookup_data = resp
        .json::<LookupResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(lookup_data)
}
