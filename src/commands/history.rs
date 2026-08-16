use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::history::list_recent;
use crate::engine::index::open_index_if_exists;
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct HistoryArgs {
    #[arg(default_value = ".")]
    root: PathBuf,

    /// Filter to entries for a specific file
    #[arg(short = 'f', long)]
    file: Option<String>,

    /// Include undone entries
    #[arg(short = 'a', long)]
    all: bool,

    /// Max entries
    #[arg(short = 'n', long, default_value = "30")]
    top: i64,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: HistoryArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found. History requires the index. Run `ore index-build`."),
    };
    let rows = list_recent(&conn, args.top, args.file.as_deref(), args.all)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("{}", "(no history)".dimmed());
        return Ok(());
    }
    println!("{} {} recent operations", "History:".cyan().bold(), rows.len().to_string().yellow());
    for e in &rows {
        let ts = chrono::DateTime::from_timestamp(e.timestamp, 0)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| e.timestamp.to_string());
        let mark = if e.undone { "↶".yellow().to_string() } else { " ".to_string() };
        let file = e.file.as_deref().unwrap_or("").to_string();
        let bak = e.backup.as_deref().map(|s| format!(" ← {}", s)).unwrap_or_default();
        let det = e.details.as_deref().map(|s| format!(" ({})", s)).unwrap_or_default();
        println!("  {} [{}] {} {} {}{}{}",
            mark,
            e.id.to_string().dimmed(),
            ts.dimmed(),
            e.operation.magenta(),
            file.cyan(),
            bak.dimmed(),
            det.dimmed());
    }
    Ok(())
}
