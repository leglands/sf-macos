//! Ollama local LLM provider (localhost:11434)
//! API: POST /api/chat (OpenAI-compatible via /v1/chat/completions)

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use crate::{SfError, SfResult};
use super::client::{ChatRequest, ChatResponse, LlmProvider, ToolCall};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self { client: Client::new(), base_url: base_url.into(), model: model.into() }
    }
    pub fn default() -> Self {
        Self::new("http://localhost:11434", "llama3.2")
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }

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
            "stream": false,
            "options": { "temperature": req.temperature, "num_predict": req.max_tokens }
        });

        // Ollama supports tools via /v1/chat/completions (OpenAI-compatible endpoint)
        if !req.tools.is_empty() {
            let tools: Vec<Value> = req.tools.iter().map(|t| json!({
                "type": "function",
                "function": { "name": t.function.name, "description": t.function.description, "parameters": t.function.parameters }
            })).collect();
            body["tools"] = json!(tools);
        }

        let url = format!("{}/v1/chat/completions", self.base_url);
        let resp: Value = self.client.post(&url).json(&body).send().await
            .map_err(|e| SfError::Llm(e.to_string()))?
            .json().await
            .map_err(|e| SfError::Llm(e.to_string()))?;

        parse_openai_response(resp)
    }
}

pub fn parse_openai_response(resp: Value) -> SfResult<ChatResponse> {
    let choice = &resp["choices"][0];
    let msg = &choice["message"];
    let content = msg["content"].as_str().unwrap_or("").to_string();
    let finish_reason = choice["finish_reason"].as_str().unwrap_or("stop").to_string();

    let mut tool_calls = Vec::new();
    if let Some(tcs) = msg["tool_calls"].as_array() {
        for tc in tcs {
            let args: serde_json::Value = serde_json::from_str(
                tc["function"]["arguments"].as_str().unwrap_or("{}")
            ).unwrap_or(json!({}));
            tool_calls.push(ToolCall {
                id: tc["id"].as_str().unwrap_or("").to_string(),
                name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                arguments: args,
            });
        }
    }

    Ok(ChatResponse {
        content,
        tool_calls,
        finish_reason,
        input_tokens: resp["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
        output_tokens: resp["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
    })
}
