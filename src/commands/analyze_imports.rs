use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct AnalyzeImportsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Sort by: fanout (default), fanin, name
    #[arg(short = 's', long, default_value = "fanout")]
    sort: String,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AnalyzeImportsArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;
    let mut rows: Vec<(PathBuf, usize, usize)> = g.deps.iter().map(|(f, deps)| {
        let fanin = g.deps_reverse.get(f).map(|s| s.len()).unwrap_or(0);
        (f.clone(), deps.len(), fanin)
    }).collect();

    match args.sort.as_str() {
        "fanin" => rows.sort_by(|a, b| b.2.cmp(&a.2)),
        "name" => rows.sort_by(|a, b| a.0.cmp(&b.0)),
        _ => rows.sort_by(|a, b| b.1.cmp(&a.1)),
    }

    if args.json {
        let arr: Vec<_> = rows.iter().take(args.top).map(|(p, out, inc)| serde_json::json!({
            "file": short_path(&args.path, p),
            "fanout": out,
            "fanin": inc,
        })).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    println!("{} {} files scanned", "Imports:".cyan().bold(), g.deps.len().to_string().yellow());
    println!("{:>6} {:>6}  {}", "→out".dimmed(), "in←".dimmed(), "file".dimmed());
    for (p, out, inc) in rows.iter().take(args.top) {
        println!("{:>6} {:>6}  {}", out.to_string().yellow(), inc.to_string().magenta(), short_path(&args.path, p).cyan());
    }
    Ok(())
}
