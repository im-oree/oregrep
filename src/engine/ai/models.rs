use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::engine::index::open_index;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub provider: String,
    pub id: String,
    pub context_window: Option<u32>,
    pub input_cost_per_1m: Option<f64>,
    pub output_cost_per_1m: Option<f64>,
    pub capabilities: Vec<String>, // e.g. ["chat","tools","vision"]
    pub cached_at: i64,
}

fn ensure_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS ai_models (
            provider TEXT NOT NULL,
            id TEXT NOT NULL,
            context_window INTEGER,
            input_cost_per_1m REAL,
            output_cost_per_1m REAL,
            capabilities TEXT,
            cached_at INTEGER NOT NULL,
            PRIMARY KEY(provider, id)
        );
    "#)?;
    Ok(())
}

pub fn cache_dir_root() -> std::path::PathBuf {
    // Reuse index workspace-root logic; fall back to cwd
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

pub fn save_models(models: &[ModelInfo]) -> Result<()> {
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    let now = chrono::Local::now().timestamp();
    let tx = conn.unchecked_transaction()?;
    for m in models {
        let caps = m.capabilities.join(",");
        tx.execute(
            "INSERT OR REPLACE INTO ai_models (provider, id, context_window, input_cost_per_1m, output_cost_per_1m, capabilities, cached_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![m.provider, m.id, m.context_window, m.input_cost_per_1m, m.output_cost_per_1m, caps, now],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_cached(provider: &str, ttl_secs: u64) -> Result<Vec<ModelInfo>> {
    let root = cache_dir_root();
    let conn = open_index(&root)?;
    ensure_table(&conn)?;
    let now = chrono::Local::now().timestamp();
    let cutoff = now - ttl_secs as i64;
    let mut stmt = conn.prepare("SELECT provider, id, context_window, input_cost_per_1m, output_cost_per_1m, capabilities, cached_at FROM ai_models WHERE provider = ?1 AND cached_at >= ?2 ORDER BY id")?;
    let rows: Vec<ModelInfo> = stmt.query_map(params![provider, cutoff], |r| {
        Ok(ModelInfo {
            provider: r.get(0)?,
            id: r.get(1)?,
            context_window: r.get(2)?,
            input_cost_per_1m: r.get(3)?,
            output_cost_per_1m: r.get(4)?,
            capabilities: r.get::<_, Option<String>>(5)?.unwrap_or_default().split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
            cached_at: r.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(rows)
}

/// A curated cost table used to augment provider `/models` responses which
/// often omit pricing. Extend as models change.
pub fn augment_cost(provider: &str, id: &str) -> (Option<f64>, Option<f64>, Option<u32>, Vec<String>) {
    let key = format!("{}:{}", provider, id);
    let (in_c, out_c, ctx, caps): (Option<f64>, Option<f64>, Option<u32>, &[&str]) = match key.as_str() {
        // OpenAI
        "openai:gpt-4o" => (Some(2.5), Some(10.0), Some(128_000), &["chat","tools","vision"]),
        "openai:gpt-4o-mini" => (Some(0.15), Some(0.6), Some(128_000), &["chat","tools","vision"]),
        "openai:gpt-4-turbo" => (Some(10.0), Some(30.0), Some(128_000), &["chat","tools","vision"]),
        "openai:o1" => (Some(15.0), Some(60.0), Some(200_000), &["chat","reasoning"]),
        "openai:o1-mini" => (Some(3.0), Some(12.0), Some(128_000), &["chat","reasoning"]),
        // Anthropic
        "anthropic:claude-sonnet-4-5" => (Some(3.0), Some(15.0), Some(200_000), &["chat","tools","vision"]),
        "anthropic:claude-opus-4-5" => (Some(15.0), Some(75.0), Some(200_000), &["chat","tools","vision"]),
        "anthropic:claude-haiku-4-5" => (Some(1.0), Some(5.0), Some(200_000), &["chat","tools","vision"]),
        // Groq (all extremely cheap, some free)
        "groq:llama-3.3-70b-versatile" => (Some(0.59), Some(0.79), Some(128_000), &["chat","tools"]),
        "groq:llama-3.1-8b-instant" => (Some(0.05), Some(0.08), Some(128_000), &["chat","tools"]),
        "groq:mixtral-8x7b-32768" => (Some(0.24), Some(0.24), Some(32_768), &["chat"]),
        // Google
        "google:gemini-2.5-pro" => (Some(1.25), Some(5.0), Some(1_000_000), &["chat","tools","vision"]),
        "google:gemini-2.5-flash" => (Some(0.075), Some(0.3), Some(1_000_000), &["chat","tools","vision"]),
        // Mistral
        "mistral:mistral-large-latest" => (Some(2.0), Some(6.0), Some(128_000), &["chat","tools"]),
        "mistral:mistral-small-latest" => (Some(0.2), Some(0.6), Some(128_000), &["chat","tools"]),
        // DeepSeek
        "deepseek:deepseek-chat" => (Some(0.14), Some(0.28), Some(128_000), &["chat","tools"]),
        "deepseek:deepseek-reasoner" => (Some(0.55), Some(2.19), Some(64_000), &["chat","reasoning"]),
        // Local
        _ if provider == "ollama" || provider == "lmstudio" => (Some(0.0), Some(0.0), None, &["chat"]),
        _ => (None, None, None, &[]),
    };
    (in_c, out_c, ctx, caps.iter().map(|s| s.to_string()).collect())
}
