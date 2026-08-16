use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

use crate::engine::index::open_index;

static PROCESS_SPEND_USD: OnceLock<Mutex<f64>> = OnceLock::new();

fn spend_cell() -> &'static Mutex<f64> {
    PROCESS_SPEND_USD.get_or_init(|| Mutex::new(0.0))
}

pub fn add_process_cost(cost_usd: f64) {
    if let Ok(mut total) = spend_cell().lock() {
        *total += cost_usd.max(0.0);
    }
}

pub fn process_total_cost() -> f64 {
    spend_cell().lock().map(|v| *v).unwrap_or(0.0)
}

fn ensure_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS ai_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts INTEGER NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            tokens_in INTEGER NOT NULL,
            tokens_out INTEGER NOT NULL,
            cost_usd REAL NOT NULL,
            duration_ms INTEGER NOT NULL,
            task TEXT
        );
    "#)?;
    Ok(())
}

pub fn record(
    provider: &str,
    model: &str,
    tokens_in: u32,
    tokens_out: u32,
    cost_usd: f64,
    duration_ms: u128,
    task: Option<&str>,
) -> Result<()> {
    let root = crate::engine::ai::models::cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    conn.execute(
        "INSERT INTO ai_usage (ts, provider, model, tokens_in, tokens_out, cost_usd, duration_ms, task) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            chrono::Local::now().timestamp(),
            provider, model,
            tokens_in as i64, tokens_out as i64,
            cost_usd, duration_ms as i64,
            task
        ],
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageRow {
    pub provider: String,
    pub model: String,
    pub calls: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRow {
    pub id: i64,
    pub ts: i64,
    pub provider: String,
    pub model: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
    pub duration_ms: i64,
    pub task: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub process_spend_usd: f64,
    pub session_budget_usd: f64,
    pub call_budget_usd: f64,
    pub historical_total_usd: f64,
}

pub fn budget_status(cfg: &crate::engine::ai::config::AiConfig) -> Result<BudgetStatus> {
    Ok(BudgetStatus {
        process_spend_usd: process_total_cost(),
        session_budget_usd: cfg.session_budget_usd,
        call_budget_usd: cfg.call_budget_usd,
        historical_total_usd: total_cost(None)?,
    })
}

pub fn summary_by_model(days_back: Option<i64>) -> Result<Vec<UsageRow>> {
    let root = crate::engine::ai::models::cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    let cutoff: i64 = days_back
        .map(|d| chrono::Local::now().timestamp() - d * 86400)
        .unwrap_or(0);
    let mut stmt = conn.prepare(
        "SELECT provider, model, COUNT(*), SUM(tokens_in), SUM(tokens_out), SUM(cost_usd) \
         FROM ai_usage WHERE ts >= ?1 \
         GROUP BY provider, model ORDER BY SUM(cost_usd) DESC",
    )?;
    let rows: Vec<UsageRow> = stmt
        .query_map(params![cutoff], |r| {
            Ok(UsageRow {
                provider: r.get(0)?,
                model: r.get(1)?,
                calls: r.get(2)?,
                tokens_in: r.get::<_, i64>(3).unwrap_or(0),
                tokens_out: r.get::<_, i64>(4).unwrap_or(0),
                cost_usd: r.get::<_, f64>(5).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

pub fn total_cost(days_back: Option<i64>) -> Result<f64> {
    let root = crate::engine::ai::models::cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    let cutoff: i64 = days_back
        .map(|d| chrono::Local::now().timestamp() - d * 86400)
        .unwrap_or(0);
    Ok(conn
        .query_row(
            "SELECT COALESCE(SUM(cost_usd), 0.0) FROM ai_usage WHERE ts >= ?1",
            params![cutoff],
            |r| r.get(0),
        )
        .unwrap_or(0.0))
}

pub fn query_history(
    limit: i64,
    task_filter: Option<&str>,
    since_ts: Option<i64>,
) -> Result<Vec<HistoryRow>> {
    let root = crate::engine::ai::models::cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    let cutoff = since_ts.unwrap_or(0);
    let rows: Vec<HistoryRow>;
    match task_filter {
        Some(task) => {
            let mut stmt = conn.prepare(
                "SELECT id, ts, provider, model, tokens_in, tokens_out, cost_usd, duration_ms, task \
                 FROM ai_usage WHERE ts >= ?1 AND task = ?2 \
                 ORDER BY ts DESC LIMIT ?3",
            )?;
            let iter = stmt.query_map(params![cutoff, task, limit], map_row)?;
            rows = iter.filter_map(|r| r.ok()).collect();
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, ts, provider, model, tokens_in, tokens_out, cost_usd, duration_ms, task \
                 FROM ai_usage WHERE ts >= ?1 \
                 ORDER BY ts DESC LIMIT ?2",
            )?;
            let iter = stmt.query_map(params![cutoff, limit], map_row)?;
            rows = iter.filter_map(|r| r.ok()).collect();
        }
    }
    Ok(rows)
}

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryRow> {
    Ok(HistoryRow {
        id: r.get(0)?,
        ts: r.get(1)?,
        provider: r.get(2)?,
        model: r.get(3)?,
        tokens_in: r.get::<_, i64>(4).unwrap_or(0),
        tokens_out: r.get::<_, i64>(5).unwrap_or(0),
        cost_usd: r.get::<_, f64>(6).unwrap_or(0.0),
        duration_ms: r.get::<_, i64>(7).unwrap_or(0),
        task: r.get(8)?,
    })
}
