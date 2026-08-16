use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct AnalyzeCouplingArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: AnalyzeCouplingArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;

    // Coupling score = fanout + fanin (files that are both heavy consumers and heavy providers)
    let mut rows: Vec<(PathBuf, usize, usize, usize)> = g.deps.iter().map(|(f, deps)| {
        let fanin = g.deps_reverse.get(f).map(|s| s.len()).unwrap_or(0);
        let fanout = deps.len();
        let score = fanin + fanout;
        (f.clone(), fanout, fanin, score)
    }).collect();
    rows.sort_by(|a, b| b.3.cmp(&a.3));

    println!("{}", "Coupling (fanin+fanout, higher = more entangled):".cyan().bold());
    println!("{:>7} {:>5} {:>5}  {}", "score".dimmed(), "→out".dimmed(), "in←".dimmed(), "file".dimmed());
    for (p, o, i, s) in rows.iter().take(args.top) {
        let color = if *s >= 20 { "red" } else if *s >= 10 { "yellow" } else { "green" };
        println!("{:>7} {:>5} {:>5}  {}",
            s.to_string().color(color).bold(),
            o.to_string().dimmed(),
            i.to_string().dimmed(),
            short_path(&args.path, p).cyan());
    }
    Ok(())
}
