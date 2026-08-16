use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::engine::paths::{canonicalize_clean, workspace_root};
use crate::engine::state::state_dir;

/// Location strategy for the index database.
#[derive(Copy, Clone, Debug)]
pub enum IndexLocation {
    /// `.ore-index/index.db` inside the workspace root
    Workspace,
    /// `%APPDATA%\ore\indexes\<workspace-hash>\index.db`
    AppData,
}

#[allow(dead_code)] // Staged: used by the --from-index default flip.
pub fn default_location() -> IndexLocation {
    IndexLocation::Workspace
}

/// Resolve the index database path for a given root directory.
pub fn db_path_for(root: &Path, location: IndexLocation) -> Result<PathBuf> {
    match location {
        IndexLocation::Workspace => {
            let ws = workspace_root(root).unwrap_or_else(|| canonicalize_clean(root));
            let dir = ws.join(".ore-index");
            crate::engine::paths::ensure_dir(&dir)?;
            Ok(dir.join("index.db"))
        }
        IndexLocation::AppData => {
            let ws = workspace_root(root).unwrap_or_else(|| canonicalize_clean(root));
            let hash = hash_path(&ws);
            let dir = state_dir()?.join("indexes").join(hash);
            crate::engine::paths::ensure_dir(&dir)?;
            Ok(dir.join("index.db"))
        }
    }
}

/// Try workspace first; fall back to appdata if workspace path is not writable.
pub fn resolve_db_path(root: &Path) -> Result<PathBuf> {
    // Try workspace unless disabled
    match db_path_for(root, IndexLocation::Workspace) {
        Ok(p) => Ok(p),
        Err(_) => db_path_for(root, IndexLocation::AppData),
    }
}

fn hash_path(p: &Path) -> String {
    let mut h = Sha256::new();
    h.update(p.to_string_lossy().as_bytes());
    format!("{:x}", h.finalize())[..16].to_string()
}

/// Ensure the workspace's .ore-index/ dir is in .gitignore.
pub fn ensure_gitignore_entry(root: &Path) -> Result<()> {
    let gi = root.join(".gitignore");
    let line = ".ore-index/";
    if gi.exists() {
        let content = std::fs::read_to_string(&gi)?;
        if content.lines().any(|l| l.trim() == line) { return Ok(()); }
        let mut new = content;
        if !new.ends_with('\n') { new.push('\n'); }
        new.push_str(line);
        new.push('\n');
        std::fs::write(&gi, new)?;
    } else {
        std::fs::write(&gi, format!("{}\n", line))?;
    }
    Ok(())
}

/// Open (creating if missing) the index database and ensure schema.
pub fn open_index(root: &Path) -> Result<Connection> {
    let path = resolve_db_path(root)?;
    let conn = Connection::open(&path).with_context(|| format!("Opening index at {}", path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL").ok();
    conn.pragma_update(None, "synchronous", "NORMAL").ok();
    conn.pragma_update(None, "foreign_keys", "ON").ok();
    init_schema(&conn)?;
    Ok(conn)
}

/// Open only if the index already exists. Returns Ok(None) if not built yet.
pub fn open_index_if_exists(root: &Path) -> Result<Option<Connection>> {
    let path = resolve_db_path(root)?;
    if !path.exists() { return Ok(None); }
    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL").ok();
    conn.pragma_update(None, "synchronous", "NORMAL").ok();
    conn.pragma_update(None, "foreign_keys", "ON").ok();
    Ok(Some(conn))
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            size INTEGER NOT NULL,
            mtime INTEGER NOT NULL,
            hash TEXT NOT NULL,
            lang TEXT,
            indexed_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_files_hash ON files(hash);
        CREATE INDEX IF NOT EXISTS idx_files_lang ON files(lang);

        CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            line INTEGER NOT NULL,
            col INTEGER NOT NULL,
            exported INTEGER NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
        CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);
        CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
        CREATE INDEX IF NOT EXISTS idx_symbols_exported ON symbols(exported);

        CREATE TABLE IF NOT EXISTS imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            source TEXT NOT NULL,
            resolved_file_id INTEGER,
            named TEXT,
            line INTEGER NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (resolved_file_id) REFERENCES files(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_imports_file ON imports(file_id);
        CREATE INDEX IF NOT EXISTS idx_imports_resolved ON imports(resolved_file_id);

        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            operation TEXT NOT NULL,
            file TEXT,
            backup TEXT,
            details TEXT,
            undone INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_history_file ON history(file);
        CREATE INDEX IF NOT EXISTS idx_history_ts ON history(timestamp);
    "#)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileRow {
    pub id: i64,
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub hash: String,
    pub lang: Option<String>,
    pub indexed_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SymbolRow {
    pub name: String,
    pub kind: String,
    pub line: i64,
    pub col: i64,
    pub exported: bool,
    pub file: String,
}

/// Upsert a file row and return the row id + whether it was new/changed.
pub fn upsert_file(conn: &Connection, path: &str, size: i64, mtime: i64, hash: &str, lang: Option<&str>) -> Result<(i64, bool)> {
    let now = chrono::Local::now().timestamp();
    // Check existing
    let existing: Option<(i64, String)> = conn.query_row(
        "SELECT id, hash FROM files WHERE path = ?1",
        params![path],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).ok();
    match existing {
        Some((id, old_hash)) if old_hash == hash => {
            // No change; still touch indexed_at
            conn.execute("UPDATE files SET indexed_at = ?1 WHERE id = ?2", params![now, id])?;
            Ok((id, false))
        }
        Some((id, _)) => {
            conn.execute(
                "UPDATE files SET size=?1, mtime=?2, hash=?3, lang=?4, indexed_at=?5 WHERE id=?6",
                params![size, mtime, hash, lang, now, id])?;
            // Wipe stale symbols/imports
            conn.execute("DELETE FROM symbols WHERE file_id = ?1", params![id])?;
            conn.execute("DELETE FROM imports WHERE file_id = ?1", params![id])?;
            Ok((id, true))
        }
        None => {
            conn.execute(
                "INSERT INTO files (path, size, mtime, hash, lang, indexed_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![path, size, mtime, hash, lang, now])?;
            Ok((conn.last_insert_rowid(), true))
        }
    }
}

pub fn insert_symbol(conn: &Connection, file_id: i64, name: &str, kind: &str, line: i64, col: i64, exported: bool) -> Result<()> {
    conn.execute(
        "INSERT INTO symbols (file_id, name, kind, line, col, exported) VALUES (?1,?2,?3,?4,?5,?6)",
        params![file_id, name, kind, line, col, exported as i64])?;
    Ok(())
}

pub fn insert_import(conn: &Connection, file_id: i64, source: &str, resolved_file_id: Option<i64>, named_csv: &str, line: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO imports (file_id, source, resolved_file_id, named, line) VALUES (?1,?2,?3,?4,?5)",
        params![file_id, source, resolved_file_id, named_csv, line])?;
    Ok(())
}

pub fn file_id_for(conn: &Connection, path: &str) -> Option<i64> {
    conn.query_row("SELECT id FROM files WHERE path = ?1", params![path], |r| r.get(0)).ok()
}

/// Remove file rows that no longer exist on disk.
pub fn gc_missing(conn: &Connection) -> Result<usize> {
    let mut stmt = conn.prepare("SELECT id, path FROM files")?;
    let rows: Vec<(i64, String)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    let mut removed = 0usize;
    for (id, path) in rows {
        if !PathBuf::from(&path).exists() {
            conn.execute("DELETE FROM files WHERE id = ?1", params![id])?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Compute SHA-256 of a file.
pub fn hash_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}

/// Get the total file count in the index.
pub fn file_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?)
}

pub fn symbol_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?)
}

pub fn import_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM imports", [], |r| r.get(0))?)
}

/// Get all files where hash doesn't match current disk content.
pub fn stale_files(conn: &Connection) -> Result<Vec<FileRow>> {
    let mut stmt = conn.prepare("SELECT id, path, size, mtime, hash, lang, indexed_at FROM files")?;
    let rows: Vec<FileRow> = stmt.query_map([], |r| Ok(FileRow {
        id: r.get(0)?,
        path: r.get(1)?,
        size: r.get(2)?,
        mtime: r.get(3)?,
        hash: r.get(4)?,
        lang: r.get(5)?,
        indexed_at: r.get(6)?,
    }))?.filter_map(|r| r.ok()).collect();

    let mut stale = Vec::new();
    for row in rows {
        let p = PathBuf::from(&row.path);
        if !p.exists() { continue; }
        if let Ok(meta) = std::fs::metadata(&p) {
            let cur_mtime = meta.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64).unwrap_or(0);
            let cur_size = meta.len() as i64;
            if cur_mtime != row.mtime || cur_size != row.size {
                stale.push(row);
            }
        }
    }
    Ok(stale)
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute("INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)", params![key, value])?;
    Ok(())
}

pub fn get_meta(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0)).ok()
}

pub fn all_files(conn: &Connection) -> Result<Vec<FileRow>> {
    let mut stmt = conn.prepare("SELECT id, path, size, mtime, hash, lang, indexed_at FROM files")?;
    let rows: Vec<FileRow> = stmt.query_map([], |r| Ok(FileRow {
        id: r.get(0)?,
        path: r.get(1)?,
        size: r.get(2)?,
        mtime: r.get(3)?,
        hash: r.get(4)?,
        lang: r.get(5)?,
        indexed_at: r.get(6)?,
    }))?.filter_map(|r| r.ok()).collect();
    Ok(rows)
}

pub fn search_symbols(conn: &Connection, name_pattern: &str, kind_filter: Option<&str>, exported_only: bool) -> Result<Vec<SymbolRow>> {
    let mut sql = String::from(
        "SELECT s.name, s.kind, s.line, s.col, s.exported, f.path
         FROM symbols s JOIN files f ON s.file_id = f.id
         WHERE s.name LIKE ?1"
    );
    if kind_filter.is_some() { sql.push_str(" AND s.kind = ?2"); }
    if exported_only { sql.push_str(" AND s.exported = 1"); }
    sql.push_str(" ORDER BY s.name, f.path");

    let mut stmt = conn.prepare(&sql)?;
    let pattern = format!("%{}%", name_pattern);
    let rows: Vec<SymbolRow> = if let Some(k) = kind_filter {
        stmt.query_map(params![pattern, k], |r| Ok(SymbolRow {
            name: r.get(0)?, kind: r.get(1)?, line: r.get(2)?, col: r.get(3)?,
            exported: r.get::<_, i64>(4)? != 0, file: r.get(5)?,
        }))?.filter_map(|r| r.ok()).collect()
    } else {
        stmt.query_map(params![pattern], |r| Ok(SymbolRow {
            name: r.get(0)?, kind: r.get(1)?, line: r.get(2)?, col: r.get(3)?,
            exported: r.get::<_, i64>(4)? != 0, file: r.get(5)?,
        }))?.filter_map(|r| r.ok()).collect()
    };
    Ok(rows)
}
