//! SSE consumer — parse Server-Sent Events from remote SF instances

use reqwest::Client;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::RemoteResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

/// Consume SSE stream from URL, send events to channel.
/// Stops when channel is closed or stream ends.
pub async fn consume(
    url: String,
    auth_token: Option<String>,
    tx: mpsc::Sender<SseEvent>,
) -> RemoteResult<()> {
    let mut builder = Client::new().get(&url);
    if let Some(token) = auth_token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    builder = builder.header("Accept", "text/event-stream");

    let resp = builder.send().await?;
    let mut body = resp.bytes_stream();

    use futures_util::StreamExt;
    let mut buf = String::new();

    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        // SSE events are separated by double newlines
        while let Some(pos) = buf.find("\n\n") {
            let event_str = buf[..pos].to_string();
            buf = buf[pos + 2..].to_string();

            let event = parse_sse_event(&event_str);
            if tx.send(event).await.is_err() {
                return Ok(());  // Channel closed, stop consuming
            }
        }
    }
    Ok(())
}

fn parse_sse_event(s: &str) -> SseEvent {
    let mut event = SseEvent { event: None, data: String::new(), id: None };
    for line in s.lines() {
        if let Some(v) = line.strip_prefix("event: ") { event.event = Some(v.to_string()); }
        else if let Some(v) = line.strip_prefix("data: ") { event.data = v.to_string(); }
        else if let Some(v) = line.strip_prefix("id: ") { event.id = Some(v.to_string()); }
    }
    event
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_event_data_only() {
        let e = parse_sse_event("data: hello world");
        assert_eq!(e.data, "hello world");
        assert_eq!(e.event, None);
    }

    #[test]
    fn test_parse_sse_event_with_event_type() {
        let e = parse_sse_event("event: token\ndata: foo");
        assert_eq!(e.event, Some("token".into()));
        assert_eq!(e.data, "foo");
    }

    #[test]
    fn test_parse_sse_event_with_id() {
        let e = parse_sse_event("id: 42\ndata: bar");
        assert_eq!(e.id, Some("42".into()));
        assert_eq!(e.data, "bar");
    }

    #[test]
    fn test_parse_sse_event_empty() {
        let e = parse_sse_event("");
        assert_eq!(e.data, "");
        assert_eq!(e.event, None);
    }
}
