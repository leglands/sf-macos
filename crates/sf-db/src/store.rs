//! CRUD operations on local SQLite

use crate::{DbConn, DbResult, DbError, models::*};
use rusqlite::{params, Row};

// ── Agents ──────────────────────────────────────────────────────────────────

fn row_to_agent(row: &Row) -> rusqlite::Result<Agent> {
    let skills_json: String = row.get(4)?;
    let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_default();
    Ok(Agent {
        id: row.get(0)?,
        name: row.get(1)?,
        role: row.get(2)?,
        persona: row.get(3)?,
        skills,
        model: row.get(5)?,
        provider: row.get(6)?,
    })
}

pub fn get_agent(conn: &DbConn, id: &str) -> DbResult<Option<Agent>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT id,name,role,persona,skills,model,provider FROM agents WHERE id=?1")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? { Ok(Some(row_to_agent(row)?)) } else { Ok(None) }
}

pub fn list_agents(conn: &DbConn) -> DbResult<Vec<Agent>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT id,name,role,persona,skills,model,provider FROM agents ORDER BY name")?;
    let rows = stmt.query_map([], row_to_agent)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn upsert_agent(conn: &DbConn, a: &Agent) -> DbResult<()> {
    let skills = serde_json::to_string(&a.skills)?;
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO agents (id,name,role,persona,skills,model,provider) VALUES (?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,role=excluded.role,persona=excluded.persona,
         skills=excluded.skills,model=excluded.model,provider=excluded.provider",
        params![a.id, a.name, a.role, a.persona, skills, a.model, a.provider],
    )?;
    Ok(())
}

// ── Messages ────────────────────────────────────────────────────────────────

fn role_str(r: &MessageRole) -> &'static str {
    match r { MessageRole::System => "system", MessageRole::User => "user",
              MessageRole::Assistant => "assistant", MessageRole::Tool => "tool" }
}

fn str_role(s: &str) -> MessageRole {
    match s { "user" => MessageRole::User, "assistant" => MessageRole::Assistant,
              "tool" => MessageRole::Tool, _ => MessageRole::System }
}

pub fn append_message(conn: &DbConn, msg: &Message) -> DbResult<i64> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO messages (session_id,role,content,tool_name,tool_call_id) VALUES (?1,?2,?3,?4,?5)",
        params![msg.session_id, role_str(&msg.role), msg.content, msg.tool_name, msg.tool_call_id],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn get_messages(conn: &DbConn, session_id: &str) -> DbResult<Vec<Message>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare(
        "SELECT id,session_id,role,content,tool_name,tool_call_id FROM messages WHERE session_id=?1 ORDER BY created_at"
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        let role_s: String = row.get(2)?;
        Ok(Message {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: str_role(&role_s),
            content: row.get(3)?,
            tool_name: row.get(4)?,
            tool_call_id: row.get(5)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ── Memory ──────────────────────────────────────────────────────────────────

pub fn memory_set(conn: &DbConn, entry: &MemoryEntry) -> DbResult<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO memory (agent_id,key,value,kind) VALUES (?1,?2,?3,?4)
         ON CONFLICT(agent_id,key) DO UPDATE SET value=excluded.value,kind=excluded.kind",
        params![entry.agent_id, entry.key, entry.value, entry.kind],
    )?;
    Ok(())
}

pub fn memory_get(conn: &DbConn, agent_id: &str, key: &str) -> DbResult<Option<String>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT value FROM memory WHERE agent_id=?1 AND key=?2")?;
    let mut rows = stmt.query(params![agent_id, key])?;
    if let Some(row) = rows.next()? { Ok(Some(row.get(0)?)) } else { Ok(None) }
}

pub fn memory_list(conn: &DbConn, agent_id: &str) -> DbResult<Vec<MemoryEntry>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT agent_id,key,value,kind FROM memory WHERE agent_id=?1 ORDER BY key")?;
    let rows = stmt.query_map(params![agent_id], |row| {
        Ok(MemoryEntry { agent_id: row.get(0)?, key: row.get(1)?, value: row.get(2)?, kind: row.get(3)? })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ── Settings ─────────────────────────────────────────────────────────────────

pub fn setting_get(conn: &DbConn, key: &str) -> DbResult<Option<String>> {
    let c = conn.lock().unwrap();
    let mut stmt = c.prepare("SELECT value FROM settings WHERE key=?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? { Ok(Some(row.get(0)?)) } else { Ok(None) }
}

pub fn setting_set(conn: &DbConn, key: &str, value: &str) -> DbResult<()> {
    let c = conn.lock().unwrap();
    c.execute(
        "INSERT INTO settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=datetime('now')",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{open_db, models::*};

    fn test_db() -> crate::DbConn {
        open_db(":memory:").expect("in-memory DB")
    }

    #[test]
    fn test_upsert_and_get_agent() {
        let db = test_db();
        let agent = Agent {
            id: "a1".into(), name: "Test Agent".into(), role: "dev".into(),
            persona: Some("helpful".into()), skills: vec!["s1".into(), "s2".into()],
            model: Some("llama3.2".into()), provider: Some("ollama".into()),
        };
        upsert_agent(&db, &agent).unwrap();
        let got = get_agent(&db, "a1").unwrap().expect("should exist");
        assert_eq!(got.name, "Test Agent");
        assert_eq!(got.skills, vec!["s1", "s2"]);
    }

    #[test]
    fn test_list_agents_empty() {
        let db = test_db();
        let agents = list_agents(&db).unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_append_and_get_messages() {
        let db = test_db();
        // Create session first
        db.lock().unwrap().execute(
            "INSERT INTO sessions (id,kind,title) VALUES ('s1','chat','Test')", [],
        ).unwrap();
        let msg = Message {
            id: None, session_id: "s1".into(), role: MessageRole::User,
            content: "Hello!".into(), tool_name: None, tool_call_id: None,
        };
        append_message(&db, &msg).unwrap();
        let msgs = get_messages(&db, "s1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "Hello!");
        assert_eq!(msgs[0].role, MessageRole::User);
    }

    #[test]
    fn test_memory_set_get() {
        let db = test_db();
        let entry = MemoryEntry { agent_id: "a1".into(), key: "last_task".into(), value: "done".into(), kind: "episodic".into() };
        memory_set(&db, &entry).unwrap();
        let val = memory_get(&db, "a1", "last_task").unwrap();
        assert_eq!(val, Some("done".into()));
    }

    #[test]
    fn test_memory_update() {
        let db = test_db();
        let entry1 = MemoryEntry { agent_id: "a1".into(), key: "k".into(), value: "v1".into(), kind: "episodic".into() };
        let entry2 = MemoryEntry { agent_id: "a1".into(), key: "k".into(), value: "v2".into(), kind: "episodic".into() };
        memory_set(&db, &entry1).unwrap();
        memory_set(&db, &entry2).unwrap();
        let val = memory_get(&db, "a1", "k").unwrap();
        assert_eq!(val, Some("v2".into()));
    }

    #[test]
    fn test_setting_get_set() {
        let db = test_db();
        setting_set(&db, "llm_provider", "ollama").unwrap();
        let v = setting_get(&db, "llm_provider").unwrap();
        assert_eq!(v, Some("ollama".into()));
    }

    #[test]
    fn test_memory_list() {
        let db = test_db();
        for k in ["a", "b", "c"] {
            let e = MemoryEntry { agent_id: "agent1".into(), key: k.into(), value: "val".into(), kind: "episodic".into() };
            memory_set(&db, &e).unwrap();
        }
        let list = memory_list(&db, "agent1").unwrap();
        assert_eq!(list.len(), 3);
    }
}
