use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::{Path, PathBuf};

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct RouteArgs {
    file: PathBuf,
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    /// Depth to expand callers/callees
    #[arg(short = 'd', long, default_value = "2")]
    depth: usize,
}

pub fn run(args: RouteArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let g = build_graph(&args.root, args.ext.as_deref(), None)?;
    let target = normalize_path(&args.file);

    println!("{} {}", "Route:".cyan().bold(), args.file.display().to_string().yellow());

    // Upstream (callers)
    println!("\n{} (files that import this)", "▲ Callers".magenta().bold());
    print_upstream(&g, &target, &args.root, args.depth, 0);

    // Downstream (callees)
    println!("\n{} (files this imports)", "▼ Callees".magenta().bold());
    print_downstream(&g, &target, &args.root, args.depth, 0);
    Ok(())
}

fn print_upstream(g: &crate::engine::analysis::Graph, node: &PathBuf, root: &Path, max_depth: usize, depth: usize) {
    if depth >= max_depth { return; }
    if let Some(rev) = g.deps_reverse.get(node) {
        let mut list: Vec<&PathBuf> = rev.iter().collect();
        list.sort();
        for p in list {
            println!("{}{}  {}", "  ".repeat(depth), "←".dimmed(), short_path(root, p).cyan());
            print_upstream(g, p, root, max_depth, depth + 1);
        }
    }
}

fn print_downstream(g: &crate::engine::analysis::Graph, node: &PathBuf, root: &Path, max_depth: usize, depth: usize) {
    if depth >= max_depth { return; }
    if let Some(deps) = g.deps.get(node) {
        let mut list: Vec<&PathBuf> = deps.iter().collect();
        list.sort();
        for p in list {
            println!("{}{}  {}", "  ".repeat(depth), "→".dimmed(), short_path(root, p).cyan());
            print_downstream(g, p, root, max_depth, depth + 1);
        }
    }
}

fn normalize_path(p: &Path) -> PathBuf {
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = abs.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
}
