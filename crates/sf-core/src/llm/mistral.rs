//! Mistral AI provider (api.mistral.ai)
//!
//! Quirks:
//! - tool_call_id must be exactly 9 chars, a-z A-Z 0-9 only (no underscores)
//! - OpenAI-compatible /v1/chat/completions
//! - Models: devstral-latest (93% TC-15), mistral-small-latest, mistral-medium-latest
//! - Free experiment tier available

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use crate::{SfError, SfResult};
use super::client::{ChatMessage, ChatRequest, ChatResponse, LlmProvider};
use super::ollama::parse_openai_response;

pub struct MistralProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl MistralProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self { client: Client::new(), api_key: api_key.into(), model: model.into() }
    }

    pub fn default_model(api_key: impl Into<String>) -> Self {
        Self::new(api_key, "devstral-latest")
    }
}

/// Sanitize tool_call_id for Mistral: must be exactly 9 alphanumeric chars.
/// Deterministic hash so paired assistant+tool messages stay consistent.
fn sanitize_tool_call_id(id: &str) -> String {
    let hash = Sha256::digest(id.as_bytes());
    let hex = format!("{:x}", hash);
    // Take first 9 chars, guaranteed alphanumeric from hex
    hex[..9].to_string()
}

#[async_trait]
impl LlmProvider for MistralProvider {
    fn name(&self) -> &str { "mistral" }

    async fn chat(&self, req: &ChatRequest) -> SfResult<ChatResponse> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(json!({"role": "system", "content": sys}));
        }
        for m in &req.messages {
            let mut msg = json!({"role": m.role, "content": m.content});
            // Sanitize tool_call_id for Mistral compatibility
            if let Some(ref tc_id) = m.tool_call_id {
                msg["tool_call_id"] = json!(sanitize_tool_call_id(tc_id));
            }
            if let Some(ref name) = m.name {
                msg["name"] = json!(name);
            }
            messages.push(msg);
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": req.max_tokens,
            "temperature": req.temperature,
        });

        if !req.tools.is_empty() {
            let tools: Vec<serde_json::Value> = req.tools.iter().map(|t| json!({
                "type": "function",
                "function": {
                    "name": t.function.name,
                    "description": t.function.description,
                    "parameters": t.function.parameters,
                }
            })).collect();
            body["tools"] = json!(tools);
            body["tool_choice"] = json!("auto");
        }

        let resp: serde_json::Value = self.client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send().await.map_err(|e| SfError::Llm(e.to_string()))?
            .json().await.map_err(|e| SfError::Llm(e.to_string()))?;

        // Check for API errors
        if let Some(err) = resp.get("error") {
            return Err(SfError::Llm(format!("Mistral API error: {}", err)));
        }

        parse_openai_response(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_tool_call_id() {
        let id = "call_function_cmtqx2ghji6m_1";
        let sanitized = sanitize_tool_call_id(id);
        assert_eq!(sanitized.len(), 9);
        assert!(sanitized.chars().all(|c| c.is_ascii_alphanumeric()));
        // Deterministic
        assert_eq!(sanitize_tool_call_id(id), sanitized);
    }
}
