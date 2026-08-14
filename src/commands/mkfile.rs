use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::confirm::confirm;

#[derive(Args)]
pub struct MkfileArgs {
    /// File to create
    file: PathBuf,

    /// Initial content (use \n for newlines). If omitted, file is empty.
    #[arg(short = 'c', long)]
    content: Option<String>,

    /// Create parent dirs
    #[arg(short = 'p', long)]
    parents: bool,

    /// Overwrite if exists
    #[arg(long)]
    force: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: MkfileArgs) -> Result<()> {
    if args.file.exists() && !args.force {
        let ok = confirm(&format!("File exists: {}. Overwrite?", args.file.display()), args.yes)?;
        if !ok {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }
    if args.parents {
        if let Some(p) = args.file.parent() {
            if !p.as_os_str().is_empty() && !p.exists() {
                std::fs::create_dir_all(p)?;
            }
        }
    }
    let content = args.content.as_deref().unwrap_or("").replace("\\n", "\n");
    std::fs::write(&args.file, content.as_bytes())?;
    println!("  {} {} ({} bytes)",
        "OK".green(),
        args.file.display().to_string().cyan(),
        content.len().to_string().yellow()
    );
    Ok(())
}
