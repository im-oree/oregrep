use anyhow::Result;
use rusqlite::{named_params, Connection};
use serde::{Deserialize, Serialize};

use crate::engine::index::open_index;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: i64,
    pub operation: String,
    pub file: Option<String>,
    pub backup: Option<String>,
    pub details: Option<String>,
    pub undone: bool,
}

/// Record an operation. `root` is the workspace root (needed to locate the DB).
pub fn record(root: &Path, operation: &str, file: Option<&str>, backup: Option<&str>, details: Option<&str>) -> Result<i64> {
    let conn = open_index(root)?;
    let ts = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT INTO history (timestamp, operation, file, backup, details, undone) VALUES (?1,?2,?3,?4,?5,0)",
        rusqlite::params![ts, operation, file, backup, details])?;
    Ok(conn.last_insert_rowid())
}

/// Best-effort record: swallows errors so callers can always call it.
pub fn record_soft(root: &Path, operation: &str, file: Option<&str>, backup: Option<&str>, details: Option<&str>) {
    let _ = record(root, operation, file, backup, details);
}

pub fn list_recent(conn: &Connection, limit: i64, file: Option<&str>, include_undone: bool) -> Result<Vec<HistoryEntry>> {
    let mut sql = String::from(
        "SELECT id, timestamp, operation, file, backup, details, undone FROM history WHERE 1=1"
    );
    if file.is_some() { sql.push_str(" AND file = :file"); }
    if !include_undone { sql.push_str(" AND undone = 0"); }
    sql.push_str(" ORDER BY id DESC LIMIT :limit");

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<HistoryEntry> = if let Some(f) = file {
        stmt.query_map(named_params! { ":file": f, ":limit": limit }, |r| Ok(HistoryEntry {
            id: r.get(0)?, timestamp: r.get(1)?, operation: r.get(2)?,
            file: r.get(3)?, backup: r.get(4)?, details: r.get(5)?,
            undone: r.get::<_, i64>(6)? != 0,
        }))?.filter_map(|r| r.ok()).collect()
    } else {
        stmt.query_map(named_params! { ":limit": limit }, |r| Ok(HistoryEntry {
            id: r.get(0)?, timestamp: r.get(1)?, operation: r.get(2)?,
            file: r.get(3)?, backup: r.get(4)?, details: r.get(5)?,
            undone: r.get::<_, i64>(6)? != 0,
        }))?.filter_map(|r| r.ok()).collect()
    };
    Ok(rows)
}

pub fn mark_undone(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("UPDATE history SET undone = 1 WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn mark_redone(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("UPDATE history SET undone = 0 WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

pub fn get_entry(conn: &Connection, id: i64) -> Option<HistoryEntry> {
    conn.query_row(
        "SELECT id, timestamp, operation, file, backup, details, undone FROM history WHERE id = ?1",
        rusqlite::params![id],
        |r| Ok(HistoryEntry {
            id: r.get(0)?, timestamp: r.get(1)?, operation: r.get(2)?,
            file: r.get(3)?, backup: r.get(4)?, details: r.get(5)?,
            undone: r.get::<_, i64>(6)? != 0,
        })
    ).ok()
}

#[allow(dead_code)] // Staged: reserved for a future `redo --replay` mode.
pub fn most_recent_undone_ids(conn: &Connection, n: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT id FROM history WHERE undone = 1 ORDER BY id DESC LIMIT ?1")?;
    let ids: Vec<i64> = stmt.query_map(rusqlite::params![n], |r| r.get(0))?.filter_map(|r| r.ok()).collect();
    Ok(ids)
}
