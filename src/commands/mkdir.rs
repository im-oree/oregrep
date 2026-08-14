use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct MkdirArgs {
    /// Directory paths to create
    paths: Vec<PathBuf>,
}

pub fn run(args: MkdirArgs) -> Result<()> {
    if args.paths.is_empty() {
        anyhow::bail!("At least one path required");
    }
    for p in &args.paths {
        if p.exists() {
            println!("  {} {} (exists)", "SKIP".yellow(), p.display().to_string().dimmed());
            continue;
        }
        std::fs::create_dir_all(p)?;
        println!("  {} {}", "OK".green(), p.display().to_string().cyan());
    }
    Ok(())
}
