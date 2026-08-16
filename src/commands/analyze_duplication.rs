use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::analysis::short_path;
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct AnalyzeDuplicationArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Minimum lines in a duplicated block
    #[arg(short = 'm', long, default_value = "6")]
    min_lines: usize,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: AnalyzeDuplicationArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    // Sliding window hashes of `min_lines`
    let mut fingerprints: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<String> = content.lines().map(|l| l.trim().to_string()).collect();
        for i in 0..lines.len().saturating_sub(args.min_lines) {
            let window: Vec<&str> = lines[i..i + args.min_lines].iter().map(|s| s.as_str()).collect();
            // Skip mostly-empty windows
            if window.iter().filter(|l| !l.is_empty()).count() < args.min_lines / 2 { continue; }
            let joined = window.join("\n");
            if joined.len() < 40 { continue; }
            fingerprints.entry(joined).or_default().push((f.clone(), i + 1));
        }
    }

    let mut dupes: Vec<(String, Vec<(PathBuf, usize)>)> = fingerprints.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();
    dupes.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    println!("{} {} duplicated block groups (≥{} lines)",
        "Duplication:".cyan().bold(),
        dupes.len().to_string().yellow(),
        args.min_lines.to_string().dimmed());
    for (block, hits) in dupes.iter().take(args.top) {
        let first_line = block.lines().next().unwrap_or("").chars().take(60).collect::<String>();
        println!("\n{} {} copies — {} …", "──".magenta(), hits.len().to_string().yellow(), first_line.dimmed());
        for (p, line) in hits {
            println!("  {}:{}", short_path(&args.path, p).cyan(), line.to_string().dimmed());
        }
    }
    Ok(())
}
