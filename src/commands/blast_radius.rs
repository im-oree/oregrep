use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::engine::analysis::{build_graph, short_path};
use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::extract_imports;

#[derive(Args)]
pub struct BlastRadiusArgs {
    /// Symbol name (function/const/class/type)
    symbol: String,
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Max transitive depth
    #[arg(short = 'd', long, default_value = "3")]
    depth: usize,
}

pub fn run(args: BlastRadiusArgs) -> Result<()> {
    let g = build_graph(&args.root, args.ext.as_deref(), args.exclude.as_deref())?;

    // Normalize the symbol keys once (imports resolve to canonical paths,
    // but g.symbols is keyed by raw walker paths).
    let mut norm_symbols: HashMap<PathBuf, Vec<crate::engine::symbols::Symbol>> = HashMap::new();
    for (raw, syms) in &g.symbols {
        norm_symbols.insert(normalize_path(raw), syms.clone());
    }

    // 1) Find files that DEFINE the symbol
    let mut definers: Vec<PathBuf> = Vec::new();
    for (f, syms) in &norm_symbols {
        if syms.iter().any(|s| s.name == args.symbol) {
            definers.push(f.clone());
        }
    }
    if definers.is_empty() {
        anyhow::bail!("Symbol '{}' not found in any file", args.symbol);
    }
    println!("{} defined in {} file(s):", format!("Symbol '{}'", args.symbol).cyan().bold(), definers.len().to_string().yellow());
    for d in &definers { println!("  {}", short_path(&args.root, d).dimmed()); }

    // 2) Find direct importers who bring in this named symbol
    let mut direct: HashSet<PathBuf> = HashSet::new();
    for (f, _) in &norm_symbols {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let imps = extract_imports(&content, f);
        for imp in imps {
            if imp.named.iter().any(|n| n == &args.symbol) || imp.default.as_deref() == Some(args.symbol.as_str()) {
                // Verify resolved to one of definers
                if let Some(resolved) = crate::engine::symbols::resolve_ts_import(f, &imp.source) {
                    let c = normalize_path(&resolved);
                    if definers.iter().any(|d| normalize_path(d) == c) {
                        direct.insert(f.clone());
                    }
                }
            }
        }
    }

    // 3) Transitive: any file that imports something from a direct-user
    let mut layers: Vec<Vec<PathBuf>> = vec![direct.iter().cloned().collect()];
    let mut visited: HashSet<PathBuf> = direct.iter().cloned().collect();
    let mut queue: VecDeque<(PathBuf, usize)> = direct.iter().cloned().map(|p| (p, 1)).collect();

    while let Some((n, d)) = queue.pop_front() {
        if d >= args.depth { continue; }
        if let Some(rev) = g.deps_reverse.get(&n) {
            let mut next_layer: Vec<PathBuf> = Vec::new();
            for imp in rev {
                if visited.insert(imp.clone()) {
                    next_layer.push(imp.clone());
                    queue.push_back((imp.clone(), d + 1));
                }
            }
            if !next_layer.is_empty() {
                if layers.len() <= d { layers.resize(d + 1, Vec::new()); }
                for p in next_layer { layers[d].push(p); }
            }
        }
    }

    println!("\n{} depths:", "Blast radius".red().bold());
    let mut total = 0usize;
    for (d, layer) in layers.iter().enumerate() {
        if layer.is_empty() { continue; }
        println!("\n{} depth {}: {} file(s)", "─".dimmed(), d.to_string().green(), layer.len().to_string().yellow());
        for p in layer {
            println!("  {}", short_path(&args.root, p).cyan());
            total += 1;
        }
    }
    eprintln!("\n{} {} files affected if '{}' changes",
        "Summary:".bold(), total.to_string().yellow(), args.symbol.yellow());
    Ok(())
}

fn normalize_path(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = abs.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
}
