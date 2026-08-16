use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::index::{open_index_if_exists, search_symbols};
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct IndexSearchArgs {
    /// Symbol name substring
    pattern: String,

    #[arg(default_value = ".")]
    root: PathBuf,

    /// Filter by kind (fn, class, hook, comp, const, type, iface, enum, struct, trait, mod)
    #[arg(short = 'k', long)]
    kind: Option<String>,

    /// Only exported
    #[arg(short = 'E', long)]
    exported: bool,

    /// Max results
    #[arg(short = 'n', long, default_value = "50")]
    top: usize,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: IndexSearchArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found. Run `ore index-build` first."),
    };
    let rows = search_symbols(&conn, &args.pattern, args.kind.as_deref(), args.exported)?;

    if args.json {
        let out: Vec<_> = rows.iter().take(args.top).map(|r| serde_json::json!({
            "name": r.name, "kind": r.kind, "line": r.line, "col": r.col,
            "exported": r.exported, "file": r.file,
        })).collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    for r in rows.iter().take(args.top) {
        let star = if r.exported { "*".green().to_string() } else { " ".to_string() };
        println!("{} {:<8} {}:{}  {}",
            star,
            r.kind.magenta(),
            r.file.cyan(),
            r.line.to_string().dimmed(),
            r.name.yellow());
    }
    eprintln!("\n{} {} results (from index)", "Total:".bold(), rows.len().min(args.top).to_string().yellow());
    Ok(())
}
