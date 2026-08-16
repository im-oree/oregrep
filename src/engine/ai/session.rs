use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::engine::ai::models::cache_dir_root;
use crate::engine::index::open_index;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub ts: i64,
}

pub fn ensure_table() -> Result<()> {
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS ai_sessions (
            name TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            summary TEXT
        );
        CREATE TABLE IF NOT EXISTS ai_session_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            ts INTEGER NOT NULL,
            FOREIGN KEY (session) REFERENCES ai_sessions(name) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_ai_session_messages ON ai_session_messages(session, id);
    "#)?;
    Ok(())
}

pub fn create_if_missing(name: &str) -> Result<()> {
    ensure_table()?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let now = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT OR IGNORE INTO ai_sessions (name, created_at, updated_at, summary) VALUES (?1, ?2, ?2, NULL)",
        params![name, now],
    )?;
    Ok(())
}

pub fn append(name: &str, role: &str, content: &str) -> Result<()> {
    ensure_table()?;
    create_if_missing(name)?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let now = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT INTO ai_session_messages (session, role, content, ts) VALUES (?1, ?2, ?3, ?4)",
        params![name, role, content, now],
    )?;
    conn.execute("UPDATE ai_sessions SET updated_at = ?1 WHERE name = ?2", params![now, name])?;
    Ok(())
}

pub fn load(name: &str, limit: Option<i64>) -> Result<Vec<SessionMessage>> {
    ensure_table()?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let n = limit.unwrap_or(200);
    let mut stmt = conn.prepare(
        "SELECT role, content, ts FROM (
            SELECT role, content, ts, id FROM ai_session_messages WHERE session = ?1 ORDER BY id DESC LIMIT ?2
         ) ORDER BY id ASC"
    )?;
    let iter = stmt.query_map(params![name, n], |r| Ok(SessionMessage {
        role: r.get(0)?, content: r.get(1)?, ts: r.get(2)?,
    }))?;
    let out: Vec<SessionMessage> = iter.filter_map(|r| r.ok()).collect();
    Ok(out)
}

pub fn list() -> Result<Vec<(String, i64, i64, i64)>> {
    ensure_table()?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let mut stmt = conn.prepare(
        "SELECT s.name, s.created_at, s.updated_at, COALESCE(COUNT(m.id), 0)
         FROM ai_sessions s LEFT JOIN ai_session_messages m ON s.name = m.session
         GROUP BY s.name ORDER BY s.updated_at DESC"
    )?;
    let iter = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?)))?;
    Ok(iter.filter_map(|r| r.ok()).collect())
}

pub fn delete(name: &str) -> Result<bool> {
    ensure_table()?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let n = conn.execute("DELETE FROM ai_sessions WHERE name = ?1", params![name])?;
    Ok(n > 0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub session: String,
    pub role: String,
    pub content: String,
    pub ts: i64,
}

pub fn search_messages(query: &str, limit: i64, session_filter: Option<&str>) -> Result<Vec<SearchResult>> {
    ensure_table()?;
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    let pattern = format!("%{}%", query);
    let rows: Vec<SearchResult>;
    match session_filter {
        Some(sname) => {
            let mut stmt = conn.prepare(
                "SELECT session, role, content, ts FROM ai_session_messages \
                 WHERE session = ?1 AND content LIKE ?2 \
                 ORDER BY ts DESC LIMIT ?3",
            )?;
            let iter = stmt.query_map(params![sname, pattern, limit], |r| Ok(SearchResult {
                session: r.get(0)?,
                role: r.get(1)?,
                content: r.get(2)?,
                ts: r.get(3)?,
            }))?;
            rows = iter.filter_map(|r| r.ok()).collect();
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT session, role, content, ts FROM ai_session_messages \
                 WHERE content LIKE ?1 \
                 ORDER BY ts DESC LIMIT ?2",
            )?;
            let iter = stmt.query_map(params![pattern, limit], |r| Ok(SearchResult {
                session: r.get(0)?,
                role: r.get(1)?,
                content: r.get(2)?,
                ts: r.get(3)?,
            }))?;
            rows = iter.filter_map(|r| r.ok()).collect();
        }
    }
    Ok(rows)
}
