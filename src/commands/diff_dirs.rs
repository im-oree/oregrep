use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::commands::checksum::sha256_of;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct DiffDirsArgs {
    dir_a: PathBuf,
    dir_b: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    /// Compare by content hash (default is size + mtime; slower but exact)
    #[arg(short = 'C', long)]
    content: bool,

    /// Verbose (show unchanged too)
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: DiffDirsArgs) -> Result<()> {
    if !args.dir_a.is_dir() { anyhow::bail!("Not a directory: {}", args.dir_a.display()); }
    if !args.dir_b.is_dir() { anyhow::bail!("Not a directory: {}", args.dir_b.display()); }

    let ext = args.ext.as_deref().map(parse_extensions).unwrap_or_default();
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();

    let files_a = collect_files(&WalkConfig {
        root: args.dir_a.clone(),
        extensions: ext.clone(),
        excludes: exc.clone(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: true,
        skip_backups: true,
    })?;
    let files_b = collect_files(&WalkConfig {
        root: args.dir_b.clone(),
        extensions: ext,
        excludes: exc,
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: true,
        skip_backups: true,
    })?;

    let rel = |root: &Path, p: &Path| -> String {
        p.strip_prefix(root).map(|r| r.to_string_lossy().to_string()).unwrap_or(p.to_string_lossy().to_string())
    };

    let mut map_a: HashMap<String, PathBuf> = HashMap::new();
    let mut map_b: HashMap<String, PathBuf> = HashMap::new();
    for p in &files_a { map_a.insert(rel(&args.dir_a, p), p.clone()); }
    for p in &files_b { map_b.insert(rel(&args.dir_b, p), p.clone()); }

    let mut only_a: Vec<String> = Vec::new();
    let mut only_b: Vec<String> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    let mut unchanged: usize = 0;

    for (k, a) in &map_a {
        match map_b.get(k) {
            Some(b) => {
                let equal = if args.content {
                    sha256_of(a)? == sha256_of(b)?
                } else {
                    let ma = std::fs::metadata(a);
                    let mb = std::fs::metadata(b);
                    match (ma, mb) {
                        (Ok(ma), Ok(mb)) => ma.len() == mb.len() && ma.modified().ok() == mb.modified().ok(),
                        _ => false,
                    }
                };
                if equal { unchanged += 1; } else { changed.push(k.clone()); }
            }
            None => only_a.push(k.clone()),
        }
    }
    for (k, _) in &map_b {
        if !map_a.contains_key(k) { only_b.push(k.clone()); }
    }

    println!("{}", format!("Comparing {} vs {}", args.dir_a.display(), args.dir_b.display()).cyan().bold());
    if !only_a.is_empty() {
        println!("\n{}", format!("Only in A ({}):", only_a.len()).red().bold());
        for f in &only_a { println!("  {} {}", "-".red(), f); }
    }
    if !only_b.is_empty() {
        println!("\n{}", format!("Only in B ({}):", only_b.len()).green().bold());
        for f in &only_b { println!("  {} {}", "+".green(), f); }
    }
    if !changed.is_empty() {
        println!("\n{}", format!("Changed ({}):", changed.len()).yellow().bold());
        for f in &changed { println!("  {} {}", "~".yellow(), f); }
    }
    if args.verbose && unchanged > 0 {
        println!("\n{}", format!("Unchanged: {}", unchanged).green());
    }
    println!("\n{} A-only: {}, B-only: {}, changed: {}, unchanged: {}",
        "Summary:".bold(),
        only_a.len().to_string().red(),
        only_b.len().to_string().green(),
        changed.len().to_string().yellow(),
        unchanged.to_string().dimmed()
    );
    Ok(())
}
