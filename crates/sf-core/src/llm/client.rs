//! LlmClient trait and multi-provider dispatch

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::SfResult;

/// OpenAI-compatible message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Tool call returned by LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// OpenAI-style tool schema (for tools array in chat request)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub kind: String,  // always "function"
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Request to LLM
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSchema>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

/// Response from LLM
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Provider trait — implemented by Ollama, AzureOpenAI, MiniMax
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn supports_tools(&self) -> bool { true }
    async fn chat(&self, req: &ChatRequest) -> SfResult<ChatResponse>;
}

/// Multi-provider client — tries primary, falls back on error
pub struct LlmClient {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl LlmClient {
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self { providers }
    }

    /// Chat with automatic fallback chain
    pub async fn chat(&self, req: &ChatRequest) -> SfResult<ChatResponse> {
        let mut last_err = None;
        for provider in &self.providers {
            match provider.chat(req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    tracing::warn!("Provider {} failed: {e}", provider.name());
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| crate::SfError::Llm("No providers configured".into())))
    }
}

// async_trait is needed for dyn trait with async — add to Cargo.toml
