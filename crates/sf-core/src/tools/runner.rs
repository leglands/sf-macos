//! ToolRunner — dispatches tool calls to implementations

use std::sync::Arc;
use serde_json::Value;
use crate::SfResult;
use super::{file, shell, git, http, search};
use sf_db::DbConn;

pub struct ToolRunner {
    db: Arc<DbConn>,
}

impl ToolRunner {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }

    pub async fn execute(&self, name: &str, args: &Value) -> SfResult<String> {
        match name {
            "file_read"     => file::read(args).await,
            "file_write"    => file::write(args).await,
            "file_list"     => file::list(args).await,
            "shell_run"     => shell::run(args).await,
            "git_status"    => git::status(args).await,
            "git_diff"      => git::diff(args).await,
            "git_log"       => git::log(args).await,
            "http_fetch"    => http::fetch(args).await,
            "grep"          => search::grep(args).await,
            "memory_read"   => {
                let agent_id = args["agent_id"].as_str().unwrap_or("default");
                let key = args["key"].as_str().unwrap_or("");
                match sf_db::store::memory_get(&self.db, agent_id, key)? {
                    Some(v) => Ok(v),
                    None => Ok(format!("No memory entry for key '{key}'")),
                }
            },
            "memory_write"  => {
                let entry = sf_db::models::MemoryEntry {
                    agent_id: args["agent_id"].as_str().unwrap_or("default").into(),
                    key: args["key"].as_str().unwrap_or("").into(),
                    value: args["value"].as_str().unwrap_or("").into(),
                    kind: args["kind"].as_str().unwrap_or("episodic").into(),
                };
                sf_db::store::memory_set(&self.db, &entry)?;
                Ok("Memory stored.".into())
            },
            other => Err(crate::SfError::Tool { name: other.to_string(), msg: "Unknown tool".into() }),
        }
    }
}
