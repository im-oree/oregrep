use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::index::{file_count, get_meta, import_count, open_index_if_exists, resolve_db_path, stale_files, symbol_count};
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct IndexStatusArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
}

pub fn run(args: IndexStatusArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let db_path = resolve_db_path(&root_abs)?;
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => {
            println!("{}", "No index built yet.".yellow());
            println!("  Would live at: {}", db_path.display().to_string().dimmed());
            println!("  Build with: {}", "ore index-build".cyan());
            return Ok(());
        }
    };

    let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let fc = file_count(&conn)?;
    let sc = symbol_count(&conn)?;
    let ic = import_count(&conn)?;
    let stale = stale_files(&conn)?;

    println!("{} {}", "Index:".cyan().bold(), db_path.display().to_string().yellow());
    println!("  Size:    {}", format_size(size).green());
    println!("  Files:   {}", fc.to_string().yellow());
    println!("  Symbols: {}", sc.to_string().yellow());
    println!("  Imports: {}", ic.to_string().yellow());
    if let Some(built) = get_meta(&conn, "built_at") {
        if let Ok(ts) = built.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                println!("  Built:   {}", dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            }
        }
    }
    if let Some(upd) = get_meta(&conn, "updated_at") {
        if let Ok(ts) = upd.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                println!("  Updated: {}", dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            }
        }
    }

    if !stale.is_empty() {
        println!("\n{} {} files changed on disk since last index",
            "!".yellow().bold(),
            stale.len().to_string().yellow());
        for r in stale.iter().take(10) {
            println!("  {} {}", "~".yellow(), r.path.dimmed());
        }
        if stale.len() > 10 { println!("  {}", format!("… and {} more", stale.len() - 10).dimmed()); }
        println!("\nRun {} to refresh.", "ore index-update".cyan());
    } else {
        println!("\n{}", "Index is fresh.".green().bold());
    }
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
