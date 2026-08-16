use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct ImpactArgs {
    /// Target file
    file: PathBuf,

    #[arg(default_value = ".")]
    root: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Max depth of upstream traversal (default 5)
    #[arg(short = 'd', long, default_value = "5")]
    depth: usize,
}

pub fn run(args: ImpactArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let g = build_graph(&args.root, args.ext.as_deref(), args.exclude.as_deref())?;
    let target = std::fs::canonicalize(&args.file)?;
    let target = strip_prefix(&target);

    let mut layers: Vec<Vec<PathBuf>> = vec![vec![target.clone()]];
    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(target.clone());
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    queue.push_back((target.clone(), 0));

    while let Some((node, d)) = queue.pop_front() {
        if d >= args.depth { continue; }
        if let Some(importers) = g.deps_reverse.get(&node) {
            let next_depth = d + 1;
            for imp in importers {
                if visited.insert(imp.clone()) {
                    if layers.len() <= next_depth {
                        layers.resize(next_depth + 1, Vec::new());
                    }
                    layers[next_depth].push(imp.clone());
                    queue.push_back((imp.clone(), next_depth));
                }
            }
        }
    }

    println!("{} {}", "Impact of changing:".cyan().bold(), args.file.display().to_string().yellow());
    for (d, layer) in layers.iter().enumerate() {
        if layer.is_empty() { continue; }
        println!("\n{} depth {} ({} files)", "─".dimmed(), d.to_string().green(), layer.len().to_string().yellow());
        for p in layer {
            println!("  {}", short_path(&args.root, p).cyan());
        }
    }
    let total: usize = layers.iter().map(|l| l.len()).sum();
    eprintln!("\n{} {} files transitively affected", "Total:".bold(), (total - 1).to_string().yellow());
    Ok(())
}

fn strip_prefix(p: &std::path::Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { p.to_path_buf() }
}
