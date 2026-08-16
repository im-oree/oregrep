use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::engine::analysis::{build_graph, short_path};
use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct RelatedArgs {
    file: PathBuf,
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'n', long, default_value = "15")]
    top: usize,
}

pub fn run(args: RelatedArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let g = build_graph(&args.root, args.ext.as_deref(), None)?;
    let target = normalize_path(&args.file);

    let mut score: HashMap<PathBuf, i32> = HashMap::new();

    // Same folder siblings +2
    if let Some(parent) = args.file.parent() {
        if let Ok(rd) = std::fs::read_dir(parent) {
            for entry in rd.flatten() {
                let p = entry.path();
                if normalize_path(&p) == target { continue; }
                if !p.is_file() { continue; }
                *score.entry(normalize_path(&p)).or_insert(0) += 2;
            }
        }
    }
    // Files that import target +5
    if let Some(rev) = g.deps_reverse.get(&target) {
        for f in rev { *score.entry(f.clone()).or_insert(0) += 5; }
    }
    // Files target imports +3
    if let Some(deps) = g.deps.get(&target) {
        for f in deps { *score.entry(f.clone()).or_insert(0) += 3; }
    }
    // Git co-change +N (files that changed in same commits). git reports
    // repo-root-relative paths, so join against the repo root (cwd), not the
    // passed root arg — otherwise paths get double-prefixed.
    if ensure_repo().is_ok() {
        if let Ok(commits) = git(&["log", "--pretty=format:%H", "-n", "50", "--", &args.file.to_string_lossy()]) {
            let cwd = std::env::current_dir()?;
            let mut co: HashMap<PathBuf, i32> = HashMap::new();
            for sha in commits.lines() {
                if let Ok(files) = git(&["show", "--name-only", "--pretty=format:", sha]) {
                    for line in files.lines() {
                        if line.trim().is_empty() { continue; }
                        let p = normalize_path(&cwd.join(line.trim()));
                        if p == target { continue; }
                        *co.entry(p).or_insert(0) += 1;
                    }
                }
            }
            for (p, n) in co {
                *score.entry(p).or_insert(0) += n * 2;
            }
        }
    }

    let mut ranked: Vec<(PathBuf, i32)> = score.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{} {}", "Related to:".cyan().bold(), args.file.display().to_string().yellow());
    for (p, s) in ranked.iter().take(args.top) {
        println!("  {:>4}  {}", s.to_string().yellow(), short_path(&args.root, p).cyan());
    }
    Ok(())
}

fn normalize_path(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = abs.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
}
