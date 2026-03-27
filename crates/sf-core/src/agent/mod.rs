//! Agent executor — tool-calling loop
//!
//! Implements the agentic loop from the SF Python platform.
//! Reasoning model: arXiv:2603.01896
//! Each LLM turn is Premises (system+history) → Trace (tool calls) → Verdict (final text)

pub mod executor;
pub use executor::{AgentExecutor, ExecutorConfig, ExecutionResult};
