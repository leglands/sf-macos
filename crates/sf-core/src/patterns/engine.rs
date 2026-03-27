//! Pattern engine — sequential, parallel, loop
//!
//! Source: arXiv:2603.01896
//! Pattern selection: Premises (task description) → Trace (pattern graph) → Verdict (result)

use std::sync::Arc;
use tokio::sync::mpsc;
use serde_json::Value;
use crate::{SfResult, SfError};
use crate::agent::executor::{AgentExecutor, ExecutorConfig, AgentEvent};
use crate::llm::client::ChatMessage;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    Sequential,
    Parallel,
    Loop { max_iter: u32 },
    /// Adversarial pair: writer + reviewer iterate until approval or max_iter.
    /// Anthropic harness: "tuning a standalone evaluator to be skeptical is more tractable
    /// than making a generator critical of its own work."
    AdversarialPair { max_iter: u32 },
    /// Debate: N agents argue in rounds, last agent synthesizes.
    /// Ref: Anthropic Team of Rivals (arXiv:2601.14351)
    Debate { max_rounds: u32 },
}

pub struct PatternConfig {
    pub kind: PatternKind,
    pub agents: Vec<String>,          // agent IDs
    pub initial_message: String,
    pub loop_condition: Option<String>, // for Loop pattern: LLM prompt to check if done
}

pub struct PatternEngine {
    executor: Arc<AgentExecutor>,
}

impl PatternEngine {
    pub fn new(executor: Arc<AgentExecutor>) -> Self { Self { executor } }

    /// Run a pattern, streaming events to channel
    pub async fn run(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
    ) -> SfResult<String> {
        match &config.kind {
            PatternKind::Sequential => self.run_sequential(config, agent_configs, events_tx).await,
            PatternKind::Parallel   => self.run_parallel(config, agent_configs, events_tx).await,
            PatternKind::Loop { max_iter } => self.run_loop(config, agent_configs, events_tx, *max_iter).await,
            PatternKind::AdversarialPair { max_iter } => self.run_adversarial_pair(config, agent_configs, events_tx, *max_iter).await,
            PatternKind::Debate { max_rounds } => self.run_debate(config, agent_configs, events_tx, *max_rounds).await,
        }
    }

    /// Sequential: output of agent N becomes input of agent N+1
    async fn run_sequential(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
    ) -> SfResult<String> {
        let mut current_message = config.initial_message.clone();
        for (i, agent_cfg) in agent_configs.into_iter().enumerate() {
            let (tx, mut rx) = mpsc::channel(64);
            let messages = vec![ChatMessage { role: "user".into(), content: current_message.clone(), tool_call_id: None, name: None }];
            let exec = Arc::clone(&self.executor);
            let handle = tokio::spawn(async move { exec.run(&agent_cfg, messages, tx).await });
            while let Some(event) = rx.recv().await {
                let _ = events_tx.send(event).await;
            }
            let result = handle.await.map_err(|e| SfError::Pattern(e.to_string()))??;
            current_message = result.content;
        }
        Ok(current_message)
    }

    /// Parallel: all agents run simultaneously on the same input, results concatenated
    async fn run_parallel(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
    ) -> SfResult<String> {
        let initial = config.initial_message.clone();
        let mut handles = Vec::new();
        for agent_cfg in agent_configs {
            let (tx, mut rx) = mpsc::channel(64);
            let messages = vec![ChatMessage { role: "user".into(), content: initial.clone(), tool_call_id: None, name: None }];
            let exec = Arc::clone(&self.executor);
            let etx = events_tx.clone();
            handles.push(tokio::spawn(async move {
                let fwd = tokio::spawn(async move { while let Some(e) = rx.recv().await { let _ = etx.send(e).await; } });
                let r = exec.run(&agent_cfg, messages, tx).await;
                fwd.await.ok();
                r
            }));
        }
        let mut results = Vec::new();
        for h in handles {
            let r = h.await.map_err(|e| SfError::Pattern(e.to_string()))??;
            results.push(r.content);
        }
        Ok(results.join("\n\n---\n\n"))
    }

    /// Loop: run first agent repeatedly until condition met or max_iter reached
    async fn run_loop(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
        max_iter: u32,
    ) -> SfResult<String> {
        let agent_cfg = agent_configs.into_iter().next()
            .ok_or_else(|| SfError::Pattern("Loop pattern requires at least 1 agent".into()))?;
        let mut current_message = config.initial_message.clone();
        for iter in 0..max_iter {
            let (tx, mut rx) = mpsc::channel(64);
            let messages = vec![ChatMessage { role: "user".into(), content: current_message.clone(), tool_call_id: None, name: None }];
            let exec = Arc::clone(&self.executor);
            let cfg = ExecutorConfig { system_prompt: agent_cfg.system_prompt.clone(), ..Default::default() };
            let handle = tokio::spawn(async move { exec.run(&cfg, messages, tx).await });
            while let Some(event) = rx.recv().await {
                let _ = events_tx.send(event).await;
            }
            let result = handle.await.map_err(|e| SfError::Pattern(e.to_string()))??;
            current_message = result.content.clone();
            // Simple termination check: if response contains [DONE] or iter is last
            if current_message.contains("[DONE]") || iter + 1 == max_iter {
                break;
            }
        }
        Ok(current_message)
    }

    /// Adversarial pair: agent[0]=writer, agent[1]=reviewer. Loop until [APPROVE] or max_iter.
    /// Ref: Anthropic harness "Sprint Contract Negotiation" pattern.
    /// "Code writers cannot declare their own success."
    async fn run_adversarial_pair(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
        max_iter: u32,
    ) -> SfResult<String> {
        if agent_configs.len() < 2 {
            return Err(SfError::Pattern("AdversarialPair requires exactly 2 agents (writer + reviewer)".into()));
        }
        let writer_cfg = &agent_configs[0];
        let reviewer_cfg = &agent_configs[1];
        let mut current = config.initial_message.clone();

        for iter in 0..max_iter {
            // Writer produces
            let (tx, mut rx) = mpsc::channel(64);
            let messages = vec![ChatMessage {
                role: "user".into(),
                content: if iter == 0 { current.clone() } else { format!("Reviewer feedback:\n{}\n\nRevise your work accordingly.", current) },
                tool_call_id: None, name: None,
            }];
            let exec = Arc::clone(&self.executor);
            let cfg = ExecutorConfig { system_prompt: writer_cfg.system_prompt.clone(), ..Default::default() };
            let handle = tokio::spawn(async move { exec.run(&cfg, messages, tx).await });
            while let Some(event) = rx.recv().await { let _ = events_tx.send(event).await; }
            let writer_result = handle.await.map_err(|e| SfError::Pattern(e.to_string()))??;

            // Reviewer evaluates
            let (tx2, mut rx2) = mpsc::channel(64);
            let review_msg = vec![ChatMessage {
                role: "user".into(),
                content: format!(
                    "Review this output critically. If acceptable, respond with [APPROVE]. If not, provide specific feedback.\n\n---\n{}",
                    writer_result.content
                ),
                tool_call_id: None, name: None,
            }];
            let exec2 = Arc::clone(&self.executor);
            let cfg2 = ExecutorConfig { system_prompt: reviewer_cfg.system_prompt.clone(), ..Default::default() };
            let handle2 = tokio::spawn(async move { exec2.run(&cfg2, review_msg, tx2).await });
            while let Some(event) = rx2.recv().await { let _ = events_tx.send(event).await; }
            let reviewer_result = handle2.await.map_err(|e| SfError::Pattern(e.to_string()))??;

            if reviewer_result.content.contains("[APPROVE]") {
                return Ok(writer_result.content);
            }
            current = reviewer_result.content;
        }
        Ok(current) // Return last writer output even if not approved
    }

    /// Debate: N agents argue in rounds, building on each other's responses.
    /// Ref: Team of Rivals (arXiv:2601.14351) — multi-vendor adversarial collaboration.
    async fn run_debate(
        &self,
        config: &PatternConfig,
        agent_configs: Vec<ExecutorConfig>,
        events_tx: mpsc::Sender<AgentEvent>,
        max_rounds: u32,
    ) -> SfResult<String> {
        if agent_configs.is_empty() {
            return Err(SfError::Pattern("Debate requires at least 1 agent".into()));
        }
        let mut discussion = config.initial_message.clone();

        for round in 0..max_rounds {
            let mut round_outputs = Vec::new();
            for (i, agent_cfg) in agent_configs.iter().enumerate() {
                let (tx, mut rx) = mpsc::channel(64);
                let prompt = if round == 0 && i == 0 {
                    discussion.clone()
                } else {
                    format!("Round {} — previous discussion:\n{}\n\nProvide your analysis.", round + 1, discussion)
                };
                let messages = vec![ChatMessage { role: "user".into(), content: prompt, tool_call_id: None, name: None }];
                let exec = Arc::clone(&self.executor);
                let cfg = ExecutorConfig { system_prompt: agent_cfg.system_prompt.clone(), ..Default::default() };
                let handle = tokio::spawn(async move { exec.run(&cfg, messages, tx).await });
                while let Some(event) = rx.recv().await { let _ = events_tx.send(event).await; }
                let result = handle.await.map_err(|e| SfError::Pattern(e.to_string()))??;
                round_outputs.push(result.content);
            }
            discussion = round_outputs.join("\n\n---\n\n");
        }
        Ok(discussion)
    }
}
