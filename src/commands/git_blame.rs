use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitBlameArgs {
    /// File to blame
    file: String,

    /// Only a line range: "10", "10-20", "10:20"
    #[arg(short = 'L', long)]
    range: Option<String>,

    /// Show email instead of name
    #[arg(short = 'e', long)]
    email: bool,
}

pub fn run(args: GitBlameArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["blame".to_string()];
    if args.email { cmd.push("-e".to_string()); }
    if let Some(r) = &args.range {
        let normalized = r.replace('-', ",").replace(':', ",");
        cmd.push("-L".to_string());
        cmd.push(normalized);
    }
    cmd.push("--".to_string());
    cmd.push(args.file.clone());
    let args_ref: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&args_ref)?;
    println!("{}", format!("Blame: {}", args.file).cyan().bold());
    println!();
    print!("{}", out);
    Ok(())
}
