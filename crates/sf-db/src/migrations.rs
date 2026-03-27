//! SQLite schema migrations (idempotent — CREATE IF NOT EXISTS)

use rusqlite::Connection;
use crate::DbResult;

pub fn run_migrations(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(SCHEMA_V1)?;
    Ok(())
}

const SCHEMA_V1: &str = r#"
-- Agents
CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL,
    persona     TEXT,
    skills      TEXT,           -- JSON array of skill ids
    model       TEXT,
    provider    TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Skills (loaded from YAML at bootstrap)
CREATE TABLE IF NOT EXISTS skills (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    schema_json TEXT,           -- JSON tool schema (OpenAI format)
    category    TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions (chat or mission)
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('chat', 'mission')),
    title       TEXT,
    agent_id    TEXT,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    role        TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content     TEXT NOT NULL,
    tool_name   TEXT,
    tool_call_id TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);

-- Memory (key-value store per agent)
CREATE TABLE IF NOT EXISTS memory (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id    TEXT NOT NULL,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    kind        TEXT NOT NULL DEFAULT 'episodic',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(agent_id, key)
);
CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory(agent_id);

-- Remote instances config
CREATE TABLE IF NOT EXISTS instances (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('local', 'remote')),
    url         TEXT,
    auth_provider TEXT,         -- 'oauth2', 'token', 'none'
    access_token TEXT,          -- encrypted in Keychain; this stores keychain ref
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Patterns
CREATE TABLE IF NOT EXISTS patterns (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL,  -- 'sequential', 'parallel', 'loop', 'hierarchical'
    config_json TEXT            -- JSON config
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
