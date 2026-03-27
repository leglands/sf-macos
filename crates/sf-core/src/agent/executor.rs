//! AgentExecutor — tool-calling loop
//!
//! Source: arXiv:2603.01896 — semi-formal agentic reasoning
//! P→T→V: Premises (system+history) → Trace (tool calls) → Verdict (final answer)
//!
//! Choices:
//! - max_rounds: 15 (SF default), configurable
//! - Tool errors are fed back as tool result messages (not fatal)
//! - If LLM returns no text and no tool calls: error (prevents infinite loop)
//! - Streaming: callback via tokio channel

use std::sync::Arc;
use tokio::sync::mpsc;
use serde_json::{json, Value};
use crate::{SfError, SfResult};
use crate::llm::client::{ChatMessage, ChatRequest, ChatResponse, LlmClient, ToolSchema};
use crate::tools::runner::ToolRunner;
use sf_db::{DbConn, models::MessageRole};

/// Configuration for agent execution
pub struct ExecutorConfig {
    pub max_rounds: u32,       // default: 15
    pub max_tokens: u32,       // default: 4096
    pub temperature: f32,      // default: 0.7
    pub system_prompt: String,
    pub tools: Vec<ToolSchema>,
    /// Anthropic harness pattern: reset context each phase (keep only system_prompt + last msg)
    /// Ref: https://anthropic.com/engineering/harness-design-long-running-apps
    pub context_reset: bool,
    /// Phase hint for auto-tuning (e.g. "ui", "design" → boost max_rounds)
    pub phase_hint: Option<String>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_rounds: 15,
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt: "You are a helpful software engineering agent.".into(),
            tools: vec![],
            context_reset: false,
            phase_hint: None,
        }
    }
}

/// Resolve max_rounds based on phase_hint (Anthropic harness: design/UI → 12 iterations)
fn resolve_max_rounds(config: &ExecutorConfig) -> u32 {
    if let Some(hint) = &config.phase_hint {
        let h = hint.to_lowercase();
        let ui_markers = ["ui", "design", "ux", "frontend", "ihm", "screen", "layout", "css"];
        if ui_markers.iter().any(|m| h.contains(m)) {
            return config.max_rounds.max(12);
        }
    }
    config.max_rounds
}

/// L0 adversarial guard — deterministic fast checks on agent output (0ms, no LLM cost).
/// Swiss Cheese model: catches slop, mocks, placeholders before they propagate.
/// Ref: platform/agents/adversarial.py L0 checks.
fn adversarial_l0(content: &str) -> Option<String> {
    let checks: &[(&str, &[&str])] = &[
        ("MOCK/STUB", &["NotImplementedError", "todo!()", "unimplemented!()", "pass  # TODO", "return {}", "# FIXME"]),
        ("TEST_SKIP", &["test.skip", "describe.skip", "@pytest.mark.skip", "#[ignore]", "xit(", "xdescribe("]),
        ("SLOP", &["```\nSUCCESS\n```", "echo \"BUILD SUCCESS\"", "BUILD_SUCCESS=true"]),
        ("PLACEHOLDER", &["lorem ipsum", "TODO: implement", "FIXME: stub", "placeholder text"]),
    ];
    let mut issues = Vec::new();
    for (category, patterns) in checks {
        for pat in *patterns {
            if content.contains(pat) {
                issues.push(format!("L0-{}: found '{}'", category, pat));
            }
        }
    }
    if issues.is_empty() { None } else { Some(issues.join("; ")) }
}

/// Result of an agent execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub content: String,
    pub rounds: u32,
    pub tool_calls_made: Vec<String>,
    pub total_tokens: u32,
}

/// Streaming event sent to the UI
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Token(String),
    ToolStart { name: String, args: Value },
    ToolResult { name: String, result: String },
    Done(ExecutionResult),
    Error(String),
}

/// The core agent executor
pub struct AgentExecutor {
    llm: Arc<LlmClient>,
    tool_runner: Arc<ToolRunner>,
}

impl AgentExecutor {
    pub fn new(llm: Arc<LlmClient>, tool_runner: Arc<ToolRunner>) -> Self {
        Self { llm, tool_runner }
    }

    /// Execute an agent with history, streaming events via channel.
    /// Premises = system_prompt + messages history
    /// Trace = tool calls made during execution
    /// Verdict = final text response
    pub async fn run(
        &self,
        config: &ExecutorConfig,
        messages: Vec<ChatMessage>,
        events_tx: mpsc::Sender<AgentEvent>,
    ) -> SfResult<ExecutionResult> {
        let mut history = messages;
        let mut rounds = 0u32;
        let mut tool_calls_made = Vec::new();
        let mut total_tokens = 0u32;
        let effective_max_rounds = resolve_max_rounds(config);

        // Context reset: keep only system_prompt + initial message (Anthropic harness pattern)
        if config.context_reset && history.len() > 1 {
            let last = history.last().cloned();
            history.clear();
            if let Some(msg) = last {
                history.push(msg);
            }
            tracing::info!("context_reset: cleared history, kept last message only");
        }

        loop {
            if rounds >= effective_max_rounds {
                return Err(SfError::Pattern(format!("Max rounds ({}) exceeded", effective_max_rounds)));
            }
            rounds += 1;

            // Build request (Premises phase)
            let req = ChatRequest {
                messages: history.clone(),
                tools: config.tools.clone(),
                system_prompt: Some(config.system_prompt.clone()),
                max_tokens: config.max_tokens,
                temperature: config.temperature,
            };

            // Call LLM (Trace phase starts)
            let resp = self.llm.chat(&req).await?;
            total_tokens += resp.input_tokens + resp.output_tokens;

            // Stream text tokens (simplified — actual streaming needs SSE provider support)
            if !resp.content.is_empty() {
                let _ = events_tx.send(AgentEvent::Token(resp.content.clone())).await;
            }

            // If no tool calls → Verdict reached, run adversarial L0 then done
            if resp.tool_calls.is_empty() {
                if resp.content.is_empty() {
                    return Err(SfError::Llm("LLM returned empty response with no tool calls".into()));
                }
                // Adversarial L0: deterministic quality check (Swiss Cheese layer 0)
                if let Some(issues) = adversarial_l0(&resp.content) {
                    tracing::warn!("ADVERSARIAL L0 REJECT: {issues}");
                    let _ = events_tx.send(AgentEvent::Error(format!("L0 VETO: {issues}"))).await;
                    return Err(SfError::Pattern(format!("Adversarial L0 rejected output: {issues}")));
                }
                let result = ExecutionResult {
                    content: resp.content.clone(),
                    rounds,
                    tool_calls_made: tool_calls_made.clone(),
                    total_tokens,
                };
                let _ = events_tx.send(AgentEvent::Done(result)).await;
                return Ok(ExecutionResult { content: resp.content, rounds, tool_calls_made, total_tokens });
            }

            // Add assistant turn to history
            history.push(ChatMessage {
                role: "assistant".into(),
                content: resp.content.clone(),
                tool_call_id: None,
                name: None,
            });

            // Execute tool calls (Trace phase continues)
            for tc in &resp.tool_calls {
                let _ = events_tx.send(AgentEvent::ToolStart {
                    name: tc.name.clone(),
                    args: tc.arguments.clone(),
                }).await;
                tool_calls_made.push(tc.name.clone());

                let result = match self.tool_runner.execute(&tc.name, &tc.arguments).await {
                    Ok(r) => r,
                    Err(e) => format!("ERROR: {e}"),
                };

                let _ = events_tx.send(AgentEvent::ToolResult {
                    name: tc.name.clone(),
                    result: result.clone(),
                }).await;

                // Feed tool result back into history
                history.push(ChatMessage {
                    role: "tool".into(),
                    content: result,
                    tool_call_id: Some(tc.id.clone()),
                    name: Some(tc.name.clone()),
                });
            }
        }
    }
}
