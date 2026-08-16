use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::checksum::sha256_of;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct FindDupesArgs {
    /// Root paths (one or more)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    #[arg(short = 's', long, default_value = "1")]
    min_size: u64,
}

pub fn run(args: FindDupesArgs) -> Result<()> {
    let mut all_files: Vec<PathBuf> = Vec::new();
    for p in &args.paths {
        let cfg = WalkConfig {
            root: p.clone(),
            extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
            excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
            hidden: args.hidden,
            respect_gitignore: !args.no_ignore,
            include_binary: true,
            skip_backups: false,
        };
        all_files.extend(collect_files(&cfg)?);
    }
    println!("{} {} files scanned across {} paths",
        "Scanned:".cyan(),
        all_files.len().to_string().yellow(),
        args.paths.len().to_string().yellow()
    );

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for f in &all_files {
        let sz = match std::fs::metadata(f) { Ok(m) => m.len(), Err(_) => continue };
        if sz < args.min_size { continue; }
        by_size.entry(sz).or_default().push(f.clone());
    }

    let mut dupe_groups = 0usize;
    let mut wasted: u64 = 0;

    let mut size_groups: Vec<(u64, Vec<PathBuf>)> = by_size.into_iter().filter(|(_, v)| v.len() > 1).collect();
    size_groups.sort_by(|a, b| b.0.cmp(&a.0));

    for (sz, group) in size_groups {
        let mut by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for f in &group {
            let h = match sha256_of(f) { Ok(h) => h, Err(_) => continue };
            by_hash.entry(h).or_default().push(f.clone());
        }
        for (hash, files) in by_hash {
            if files.len() < 2 { continue; }
            dupe_groups += 1;
            wasted += sz * (files.len() as u64 - 1);
            println!("\n{} {}  {}  ({} copies, {} wasted)",
                "DUPE".red().bold(),
                short_hash(&hash).yellow(),
                format_size(sz).green(),
                files.len().to_string().yellow(),
                format_size(sz * (files.len() as u64 - 1)).red()
            );
            for f in &files {
                println!("  {}", f.display().to_string().cyan());
            }
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  Duplicate groups: {}", dupe_groups.to_string().yellow());
    println!("  Space wasted: {}", format_size(wasted).red());
    Ok(())
}

fn short_hash(h: &str) -> String { h.chars().take(12).collect() }

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
