use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct StatsArgs {
    /// Path to analyze
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    /// Top N largest files by size
    #[arg(short = 'n', long, default_value = "0")]
    top: usize,
}

pub fn run(args: StatsArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: true,
        skip_backups: false,
    };
    let files = collect_files(&cfg)?;

    let mut total_size: u64 = 0;
    let mut total_lines: usize = 0;
    let mut by_ext: HashMap<String, (usize, u64, usize)> = HashMap::new(); // (count, size, lines)
    let mut sizes: Vec<(PathBuf, u64)> = Vec::new();

    for f in &files {
        let meta = match std::fs::metadata(f) { Ok(m) => m, Err(_) => continue };
        let size = meta.len();
        total_size += size;

        let ext = f
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("(none)")
            .to_lowercase();

        // Count lines (skip binary)
        let lines = match crate::engine::encoding::is_binary(f) {
            Ok(true) => 0,
            _ => match crate::engine::encoding::read_file_smart(f) {
                Ok(c) => c.lines().count(),
                Err(_) => 0,
            },
        };
        total_lines += lines;

        let entry = by_ext.entry(ext).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += size;
        entry.2 += lines;

        sizes.push((f.clone(), size));
    }

    println!("{} {}", "Path:".dimmed(), args.path.display().to_string().cyan());
    println!("{} {}", "Files:".dimmed(), files.len().to_string().yellow());
    println!("{} {}", "Total size:".dimmed(), format_size(total_size).green());
    println!("{} {}", "Total lines:".dimmed(), total_lines.to_string().green());

    if !by_ext.is_empty() {
        println!("\n{}", "By extension:".bold());
        let mut entries: Vec<(&String, &(usize, u64, usize))> = by_ext.iter().collect();
        entries.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
        for (ext, (count, size, lines)) in entries {
            println!("  {:<10} {:>6} files  {:>10}  {:>10} lines",
                ext.cyan(),
                count.to_string().yellow(),
                format_size(*size).green(),
                lines.to_string().green()
            );
        }
    }

    if args.top > 0 {
        sizes.sort_by(|a, b| b.1.cmp(&a.1));
        println!("\n{} {}", "Top".bold(), args.top.to_string().yellow());
        for (path, sz) in sizes.iter().take(args.top) {
            println!("  {:>10}  {}", format_size(*sz).green(), path.display().to_string().cyan());
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
