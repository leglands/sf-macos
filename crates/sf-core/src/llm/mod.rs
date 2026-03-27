//! LLM client — multi-provider with automatic fallback
//!
//! Providers: MiniMax (93% TC-15) → Mistral/devstral (87%) → Ollama (local) → Azure OpenAI
//! Quirks documented per-provider inline.

pub mod client;
pub mod ollama;
pub mod azure;
pub mod minimax;
pub mod mistral;

pub use client::{LlmClient, LlmProvider, ChatRequest, ChatResponse, ToolCall, ToolSchema};
