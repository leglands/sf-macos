use serde_json::Value;
use reqwest::Client;
use crate::SfResult;

pub async fn fetch(args: &Value) -> SfResult<String> {
    let url = args["url"].as_str().ok_or_else(|| crate::SfError::Tool { name: "http_fetch".into(), msg: "missing url".into() })?;
    let client = Client::builder().timeout(std::time::Duration::from_secs(30)).build()
        .map_err(|e| crate::SfError::Llm(e.to_string()))?;
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();
    let body = resp.text().await?;
    // Truncate to 8KB to avoid flooding context
    let truncated = if body.len() > 8192 { &body[..8192] } else { &body };
    Ok(format!("[HTTP {status}]\n{truncated}"))
}
