//! MiniMax provider (api.minimax.io)
//! Quirks:
//! - Returns <think>...</think> blocks → stripped
//! - Wraps JSON in ```json...``` → stripped
//! - Doesn't support `temperature` → omitted
//! - Returns json_object not json_schema → use json_object mode

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::{SfError, SfResult};
use super::client::{ChatRequest, ChatResponse, LlmProvider};
use super::ollama::parse_openai_response;

pub struct MiniMaxProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl MiniMaxProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self { client: Client::new(), api_key: api_key.into(), model: model.into() }
    }
}

/// Strip <think>...</think> and ```json...``` wrappers from MiniMax responses
fn strip_thinking(s: &str) -> String {
    let re_think = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
    let s = re_think.replace_all(s, "");
    let re_fence = regex::Regex::new(r"(?s)^```(?:json)?\s*(.*?)\s*```$").unwrap();
    re_fence.replace_all(s.trim(), "$1").to_string()
}

#[async_trait]
impl LlmProvider for MiniMaxProvider {
    fn name(&self) -> &str { "minimax" }

    async fn chat(&self, req: &ChatRequest) -> SfResult<ChatResponse> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(json!({"role": "system", "content": sys}));
        }
        for m in &req.messages {
            messages.push(json!({"role": m.role, "content": m.content}));
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": req.max_tokens,
            // temperature omitted — MiniMax ignores it and sometimes errors
        });

        if !req.tools.is_empty() {
            let tools: Vec<serde_json::Value> = req.tools.iter().map(|t| json!({
                "type": "function",
                "function": { "name": t.function.name, "description": t.function.description, "parameters": t.function.parameters }
            })).collect();
            body["tools"] = json!(tools);
        }

        let resp: serde_json::Value = self.client
            .post("https://api.minimax.io/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send().await.map_err(|e| SfError::Llm(e.to_string()))?
            .json().await.map_err(|e| SfError::Llm(e.to_string()))?;

        // Strip thinking from content before parsing
        let mut resp = resp;
        if let Some(content) = resp["choices"][0]["message"]["content"].as_str() {
            let cleaned = strip_thinking(content);
            resp["choices"][0]["message"]["content"] = json!(cleaned);
        }

        parse_openai_response(resp)
    }
}
