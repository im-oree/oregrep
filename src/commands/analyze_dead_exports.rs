use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::engine::analysis::{build_graph, short_path};
use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_imports, Symbol};

#[derive(Args)]
pub struct AnalyzeDeadExportsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Additional entry-point patterns (never treated as dead). Repeat.
    #[arg(short = 'k', long = "keep")]
    keep: Vec<String>,
    #[arg(short = 'n', long, default_value = "50")]
    top: usize,
}

pub fn run(args: AnalyzeDeadExportsArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;

    // Normalize every symbol key the same way: canonicalize + strip \\?\ prefix.
    // (g.symbols is keyed by raw walker paths; imports resolve to canonical paths,
    //  so without this the two key spaces never match.)
    let mut norm_symbols: HashMap<PathBuf, (PathBuf, &Vec<Symbol>)> = HashMap::new();
    for (raw, syms) in &g.symbols {
        norm_symbols.insert(normalize_path(raw), (raw.clone(), syms));
    }

    // Collect ALL imported names per file
    let mut used_names: HashSet<(PathBuf, String)> = HashSet::new();
    for f in g.deps.keys() {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        for imp in extract_imports(&content, f) {
            if let Some(resolved) = crate::engine::symbols::resolve_ts_import(f, &imp.source) {
                let cleaned = normalize_path(&resolved);
                for n in &imp.named { used_names.insert((cleaned.clone(), n.clone())); }
                if let Some(d) = &imp.default { used_names.insert((cleaned.clone(), d.clone())); }
                if let Some(_ns) = &imp.namespace {
                    // Namespace imports mean any export could be used → mark all exports of this file used
                    if let Some((_, syms)) = norm_symbols.get(&cleaned) {
                        for s in *syms { used_names.insert((cleaned.clone(), s.name.clone())); }
                    }
                }
            }
        }
    }

    let mut dead: Vec<(PathBuf, String, usize)> = Vec::new();
    'files: for (norm, (raw, syms)) in &norm_symbols {
        let sp_lower = short_path(&args.path, raw).to_lowercase();
        for k in &args.keep {
            if sp_lower.contains(&k.to_lowercase()) { continue 'files; }
        }
        // Skip index/barrel/entry files by convention
        let name_lower = raw.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if matches!(name_lower.as_str(), "index" | "main" | "app" | "cli" | "mod") { continue; }

        for s in *syms {
            if !s.exported { continue; }
            let key = (norm.clone(), s.name.clone());
            if !used_names.contains(&key) {
                dead.push((raw.clone(), s.name.clone(), s.line));
            }
        }
    }
    dead.sort_by(|a, b| a.0.cmp(&b.0));

    println!("{} {} unused exports across the codebase",
        "Dead exports:".cyan().bold(),
        dead.len().to_string().yellow());
    for (f, name, line) in dead.iter().take(args.top) {
        println!("  {}:{}  {}", short_path(&args.path, f).cyan(), line.to_string().dimmed(), name.yellow());
    }
    if dead.len() > args.top {
        println!("  {}", format!("… and {} more", dead.len() - args.top).dimmed());
    }
    Ok(())
}

fn normalize_path(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = abs.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
}
