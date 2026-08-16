use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct AnalyzeExportsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'n', long, default_value = "30")]
    top: usize,
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AnalyzeExportsArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;
    let mut rows: Vec<(PathBuf, usize)> = g.symbols.iter()
        .map(|(f, syms)| (f.clone(), syms.iter().filter(|s| s.exported).count()))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1));

    let total_exports: usize = rows.iter().map(|(_, n)| *n).sum();
    let files_with = rows.iter().filter(|(_, n)| *n > 0).count();

    if args.json {
        let arr: Vec<_> = rows.iter().take(args.top).map(|(p, n)| serde_json::json!({
            "file": short_path(&args.path, p),
            "exports": n,
        })).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    println!("{} {} exports across {} files", "Exports:".cyan().bold(), total_exports.to_string().yellow(), files_with.to_string().yellow());
    for (p, n) in rows.iter().take(args.top).filter(|(_, n)| *n > 0) {
        println!("  {:>4}  {}", n.to_string().yellow(), short_path(&args.path, p).cyan());
    }
    Ok(())
}
