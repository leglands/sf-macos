//! Azure OpenAI provider
//! Quirk: uses max_completion_tokens (NOT max_tokens)

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::{SfError, SfResult};
use super::client::{ChatRequest, ChatResponse, LlmProvider};
use super::ollama::parse_openai_response;

pub struct AzureOpenAIProvider {
    client: Client,
    endpoint: String,
    api_key: String,
    deployment: String,
    api_version: String,
}

impl AzureOpenAIProvider {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, deployment: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            deployment: deployment.into(),
            api_version: "2024-10-21".into(),
        }
    }
}

#[async_trait]
impl LlmProvider for AzureOpenAIProvider {
    fn name(&self) -> &str { "azure-openai" }

    async fn chat(&self, req: &ChatRequest) -> SfResult<ChatResponse> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(json!({"role": "system", "content": sys}));
        }
        for m in &req.messages {
            messages.push(json!({"role": m.role, "content": m.content}));
        }

        let mut body = json!({
            "messages": messages,
            "max_completion_tokens": req.max_tokens,  // Azure uses this, NOT max_tokens
            "temperature": req.temperature,
        });

        if !req.tools.is_empty() {
            let tools: Vec<serde_json::Value> = req.tools.iter().map(|t| json!({
                "type": "function",
                "function": { "name": t.function.name, "description": t.function.description, "parameters": t.function.parameters }
            })).collect();
            body["tools"] = json!(tools);
        }

        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        );

        let resp: serde_json::Value = self.client.post(&url)
            .header("api-key", &self.api_key)
            .json(&body)
            .send().await.map_err(|e| SfError::Llm(e.to_string()))?
            .json().await.map_err(|e| SfError::Llm(e.to_string()))?;

        if let Some(err) = resp["error"]["message"].as_str() {
            return Err(SfError::Llm(err.to_string()));
        }

        parse_openai_response(resp)
    }
}
