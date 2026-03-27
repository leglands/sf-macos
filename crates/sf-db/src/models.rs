//! Domain models — plain structs, serde-serializable

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub persona: Option<String>,
    pub skills: Vec<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub schema_json: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind { Chat, Mission }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub kind: SessionKind,
    pub title: Option<String>,
    pub agent_id: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole { System, User, Assistant, Tool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub agent_id: String,
    pub key: String,
    pub value: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceKind { Local, Remote }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub kind: InstanceKind,
    pub url: Option<String>,
    pub auth_provider: Option<String>,
}
