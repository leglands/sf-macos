//! Pattern engine — v1: sequential + parallel + loop
//! Source: arXiv:2603.01896 — pattern selection uses P→T→V reasoning
//! v2 will add: hierarchical, network, debate

pub mod engine;
pub use engine::{PatternEngine, PatternConfig, PatternKind};
