use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{complexity_of, function_bodies, short_path};
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct AnalyzeComplexityArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
    /// Complexity threshold (default 10)
    #[arg(short = 't', long, default_value = "10")]
    threshold: usize,
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AnalyzeComplexityArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut rows: Vec<(PathBuf, String, usize, usize)> = Vec::new();
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        for (name, body, line) in function_bodies(&content) {
            let cx = complexity_of(&body);
            if cx >= args.threshold {
                rows.push((f.clone(), name, line, cx));
            }
        }
    }
    rows.sort_by(|a, b| b.3.cmp(&a.3));

    if args.json {
        let arr: Vec<_> = rows.iter().take(args.top).map(|(f, n, l, c)| serde_json::json!({
            "file": short_path(&args.path, f), "fn": n, "line": l, "complexity": c
        })).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }
    println!("{} threshold {}", "Complex functions:".cyan().bold(), args.threshold.to_string().yellow());
    for (f, name, line, cx) in rows.iter().take(args.top) {
        let color = if *cx >= 30 { "red" } else if *cx >= 20 { "yellow" } else { "green" };
        println!("  {:>4}  {}:{}  {}",
            cx.to_string().color(color).bold(),
            short_path(&args.path, f).cyan(),
            line.to_string().dimmed(),
            name.yellow());
    }
    eprintln!("\n{} {} functions above threshold", "Total:".bold(), rows.len().to_string().yellow());
    Ok(())
}
