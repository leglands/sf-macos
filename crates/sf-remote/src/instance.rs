//! Remote SF instance — HTTP API wrapper

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::RemoteResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInstance {
    pub id: String,
    pub name: String,
    pub url: String,
    pub token: Option<String>,
}

impl RemoteInstance {
    pub fn new(id: impl Into<String>, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), url: url.into(), token: None }
    }

    fn client(&self) -> Client { Client::new() }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    pub async fn health(&self) -> RemoteResult<bool> {
        let resp = self.client().get(format!("{}/api/health", self.url)).send().await?;
        Ok(resp.status().is_success())
    }

    pub async fn list_agents(&self) -> RemoteResult<Value> {
        let mut req = self.client().get(format!("{}/api/agents", self.url));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        Ok(req.send().await?.json().await?)
    }

    pub async fn list_sessions(&self) -> RemoteResult<Value> {
        let mut req = self.client().get(format!("{}/api/sessions", self.url));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        Ok(req.send().await?.json().await?)
    }

    pub async fn send_message(&self, session_id: &str, content: &str) -> RemoteResult<Value> {
        let mut req = self.client()
            .post(format!("{}/api/sessions/{session_id}/message", self.url))
            .json(&serde_json::json!({"content": content}));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        Ok(req.send().await?.json().await?)
    }
}
