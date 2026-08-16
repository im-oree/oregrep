use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{collect_source_files, extract_imports, resolve_ts_import};
use crate::engine::walker::{parse_excludes, parse_extensions};

#[derive(Args)]
pub struct NeighborsArgs {
    file: PathBuf,

    #[arg(default_value = ".")]
    path: PathBuf,

    /// Max recursion depth
    #[arg(short = 'd', long, default_value = "2")]
    depth: usize,

    /// Include upstream (files that import this)
    #[arg(short = 'u', long)]
    upstream: bool,

    /// Include downstream (files this imports) — default true
    #[arg(short = 'D', long, default_value = "true")]
    downstream: bool,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Pack neighbors into a bundle (like ore pack)
    #[arg(short = 'p', long)]
    pack: bool,

    /// Output file for pack mode
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: NeighborsArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let ext = args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into()]);
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();
    let all_files = collect_source_files(&args.path, &ext, &exc)?;

    let start = std::fs::canonicalize(&args.file)?;
    let start = strip_prefix(&start);

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut layers: Vec<Vec<PathBuf>> = Vec::new();
    visited.insert(start.clone());
    layers.push(vec![start.clone()]);

    for _ in 0..args.depth {
        let mut next_layer: HashSet<PathBuf> = HashSet::new();
        for f in layers.last().unwrap().clone() {
            // Downstream: parse f's imports
            if args.downstream {
                if let Ok(content) = read_file_smart(&f) {
                    for imp in extract_imports(&content, &f) {
                        if let Some(resolved) = resolve_ts_import(&f, &imp.source) {
                            if let Ok(abs) = std::fs::canonicalize(&resolved) {
                                let c = strip_prefix(&abs);
                                if !visited.contains(&c) { visited.insert(c.clone()); next_layer.insert(c); }
                            }
                        }
                    }
                }
            }
            // Upstream: which files in all_files import f?
            if args.upstream {
                for (op, oc) in &all_files {
                    let op_abs = std::fs::canonicalize(op).map(|x| strip_prefix(&x)).unwrap_or_else(|_| op.clone());
                    if visited.contains(&op_abs) { continue; }
                    for imp in extract_imports(oc, op) {
                        if let Some(resolved) = resolve_ts_import(op, &imp.source) {
                            if let Ok(abs) = std::fs::canonicalize(&resolved) {
                                if strip_prefix(&abs) == f {
                                    visited.insert(op_abs.clone());
                                    next_layer.insert(op_abs.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        if next_layer.is_empty() { break; }
        layers.push(next_layer.into_iter().collect());
    }

    if args.pack {
        let mut all_paths: Vec<PathBuf> = visited.into_iter().collect();
        all_paths.sort();
        let mut out = String::new();
        for p in &all_paths {
            if let Ok(content) = read_file_smart(p) {
                out.push_str(&format!("## {}\n\n```\n{}\n```\n\n", p.display(), content));
            }
        }
        if let Some(op) = args.output {
            if let Some(parent) = op.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&op, &out)?;
            println!("{} {}  ({} files, {} bytes)", "Packed:".green().bold(), op.display().to_string().cyan(), all_paths.len().to_string().yellow(), out.len().to_string().dimmed());
        } else {
            print!("{}", out);
        }
        return Ok(());
    }

    println!("{} {}", "Neighbors of:".cyan().bold(), args.file.display().to_string().yellow());
    for (depth, layer) in layers.iter().enumerate() {
        println!("\n{} depth {} ({} files)", "─".dimmed(), depth.to_string().green(), layer.len().to_string().yellow());
        for p in layer {
            println!("  {}", p.display().to_string().cyan());
        }
    }
    eprintln!("\n{} {} unique files", "Total:".bold(), visited_count(&layers).to_string().yellow());
    Ok(())
}

fn strip_prefix(p: &std::path::Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { p.to_path_buf() }
}

fn visited_count(layers: &[Vec<PathBuf>]) -> usize {
    let mut s = HashSet::new();
    for l in layers { for p in l { s.insert(p.clone()); } }
    s.len()
}
