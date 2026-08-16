use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::engine::analysis::{build_graph, short_path};
use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::symbols::{extract_imports, Symbol};

#[derive(Args)]
pub struct TrimDeadArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Patterns of files to preserve (e.g. "index" "main")
    #[arg(short = 'k', long = "keep")]
    keep: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(long)]
    no_backup: bool,
}

pub fn run(args: TrimDeadArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;

    // Normalize symbol keys the same way imports resolve (canonicalize +
    // strip \\?\ prefix) so the two key spaces actually match.
    let mut norm_symbols: HashMap<PathBuf, Vec<Symbol>> = HashMap::new();
    for (raw, syms) in &g.symbols {
        norm_symbols.insert(normalize_path(raw), syms.clone());
    }

    let mut used: HashSet<(PathBuf, String)> = HashSet::new();
    for f in g.deps.keys() {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        for imp in extract_imports(&content, f) {
            if let Some(resolved) = crate::engine::symbols::resolve_ts_import(f, &imp.source) {
                let cleaned = normalize_path(&resolved);
                for n in &imp.named { used.insert((cleaned.clone(), n.clone())); }
                if let Some(d) = &imp.default { used.insert((cleaned.clone(), d.clone())); }
                if let Some(_ns) = &imp.namespace {
                    if let Some(syms) = norm_symbols.get(&cleaned) {
                        for s in syms { used.insert((cleaned.clone(), s.name.clone())); }
                    }
                }
            }
        }
    }

    // Build edits: for each dead export, strip the `export` keyword (safe: leaves impl but drops from public API).
    let mut edits: Vec<(PathBuf, Vec<String>)> = Vec::new();
    'files: for (norm, syms) in &norm_symbols {
        let raw = raw_for(norm);
        let sp_lower = short_path(&args.path, raw).to_lowercase();
        for k in &args.keep { if sp_lower.contains(&k.to_lowercase()) { continue 'files; } }
        let stem = raw.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if matches!(stem.as_str(), "index" | "main" | "app" | "cli" | "mod") { continue; }

        let mut dead_names = Vec::new();
        for s in syms {
            if !s.exported { continue; }
            if !used.contains(&(norm.clone(), s.name.clone())) {
                dead_names.push(s.name.clone());
            }
        }
        if !dead_names.is_empty() { edits.push((norm.clone(), dead_names)); }
    }

    if edits.is_empty() {
        println!("{}", "No dead exports found.".green());
        return Ok(());
    }

    let total_dead: usize = edits.iter().map(|(_, v)| v.len()).sum();
    println!("{} {} dead exports across {} files:",
        "Dead exports:".yellow().bold(),
        total_dead.to_string().yellow(),
        edits.len().to_string().yellow());
    for (f, names) in &edits {
        println!("\n  {}", short_path(&args.path, raw_for(f)).cyan().bold());
        for n in names { println!("    {} {}", "-".red(), n.yellow()); }
    }
    println!("\n{}", "Action: strip `export` keyword from each (keeps implementation, drops public API).".dimmed());
    if args.dry_run {
        println!("\n{}", "[DRY RUN — no changes]".yellow().bold());
        return Ok(());
    }
    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Trim {} dead exports in {} files?", total_dead, edits.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }
    // Unique per invocation (microseconds + PID) so back-to-back runs don't
    // overwrite each other's backups.
    let label = format!("{}_{}", chrono::Local::now().format("%Y%m%d_%H%M%S%f"), std::process::id());
    let mut trimmed = 0usize;
    for (f, names) in &edits {
        let content = read_file_smart(f)?;
        let mut new_content = content.clone();
        for n in names {
            // Remove `export ` prefix from a matching declaration line
            let patterns = [
                (format!(r"(?m)^(\s*)export\s+(function\s+{}\b)", regex::escape(n)), "$1$2"),
                (format!(r"(?m)^(\s*)export\s+(default\s+)?(function\s+{}\b)", regex::escape(n)), "$1$2$3"),
                (format!(r"(?m)^(\s*)export\s+((?:const|let|var)\s+{}\b)", regex::escape(n)), "$1$2"),
                (format!(r"(?m)^(\s*)export\s+((?:abstract\s+)?class\s+{}\b)", regex::escape(n)), "$1$2"),
                (format!(r"(?m)^(\s*)export\s+(interface\s+{}\b)", regex::escape(n)), "$1$2"),
                (format!(r"(?m)^(\s*)export\s+(type\s+{}\b)", regex::escape(n)), "$1$2"),
                (format!(r"(?m)^(\s*)export\s+((?:const\s+)?enum\s+{}\b)", regex::escape(n)), "$1$2"),
            ];
            for (pat, rep) in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    new_content = re.replace_all(&new_content, *rep).to_string();
                }
            }
        }
        if new_content != content {
            if !args.no_backup { let _ = create_backup(f, &label); }
            write_atomic(f, &new_content, content.starts_with('\u{FEFF}'))?;
            trimmed += names.len();
        }
    }
    println!("\n{} {} exports trimmed", "Done:".green().bold(), trimmed.to_string().green());
    Ok(())
}

fn normalize_path(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = abs.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
}

/// Map a normalized path back to the raw walker path for display/IO.
/// The normalized form is the canonical absolute path, which is directly
/// usable for reading and writing, so just return it as-is.
fn raw_for(norm: &Path) -> &Path {
    norm
}
