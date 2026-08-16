use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct CheckDepsArgs {
    /// Comma-separated list of tools to check (default: common ones)
    #[arg(short = 't', long)]
    tools: Option<String>,
}

pub fn run(args: CheckDepsArgs) -> Result<()> {
    let list: Vec<String> = if let Some(t) = &args.tools {
        t.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec!["git", "node", "npm", "cargo", "rustc", "python"].into_iter().map(String::from).collect()
    };
    let mut ok = 0usize;
    let mut missing: Vec<String> = Vec::new();
    for t in &list {
        let cmd = format!("{} --version", t);
        let r = run_cmd(&cmd, false, true)?;
        if r.success() {
            ok += 1;
            println!("  {} {}  {}", "OK".green().bold(), t.cyan(), r.stdout.lines().next().unwrap_or("").trim().dimmed());
        } else {
            missing.push(t.clone());
            println!("  {} {}", "MISSING".red().bold(), t.cyan());
        }
    }
    println!("\n{} {} ok, {} missing", "Summary:".bold(), ok.to_string().green(), missing.len().to_string().red());
    if !missing.is_empty() { std::process::exit(1); }
    Ok(())
}
