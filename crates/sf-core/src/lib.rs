//! sf-core — Agent executor, LLM client, tool runner, pattern engine
//!
//! Source: arXiv:2603.01896 (semi-formal reasoning) for pattern selection logic.
//! Each LLM response is treated as: Premises → Trace → Verdict (P→T→V).

pub mod llm;
pub mod agent;
pub mod tools;
pub mod patterns;

pub use thiserror::Error;
pub use anyhow::Result;

#[derive(Error, Debug)]
pub enum SfError {
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Tool error: {name} — {msg}")]
    Tool { name: String, msg: String },
    #[error("Pattern error: {0}")]
    Pattern(String),
    #[error("DB error: {0}")]
    Db(#[from] sf_db::DbError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type SfResult<T> = Result<T, SfError>;
