use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, find_cycles, short_path};

#[derive(Args)]
pub struct AnalyzeCircularArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: AnalyzeCircularArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;
    let cycles = find_cycles(&g);
    if cycles.is_empty() {
        println!("{}", "No circular imports detected.".green().bold());
        return Ok(());
    }
    println!("{} {} cycle{} detected",
        "Circular imports:".red().bold(),
        cycles.len().to_string().yellow(),
        if cycles.len() == 1 { "" } else { "s" });
    for (i, cyc) in cycles.iter().take(args.top).enumerate() {
        println!("\n{} cycle #{}", "──".magenta(), (i + 1).to_string().yellow());
        for (j, p) in cyc.iter().enumerate() {
            let arrow = if j + 1 < cyc.len() { " →".dimmed().to_string() } else { String::new() };
            println!("  {}{}", short_path(&args.path, p).cyan(), arrow);
        }
    }
    if cycles.len() > args.top {
        println!("\n  {}", format!("… and {} more", cycles.len() - args.top).dimmed());
    }
    std::process::exit(1);
}
