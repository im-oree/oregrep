use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct TouchArgs {
    /// File(s) to create or update mtime
    files: Vec<PathBuf>,

    /// Create parent directories if missing
    #[arg(short = 'p', long)]
    parents: bool,
}

pub fn run(args: TouchArgs) -> Result<()> {
    if args.files.is_empty() {
        anyhow::bail!("At least one file required");
    }
    for f in &args.files {
        if args.parents {
            if let Some(p) = f.parent() {
                if !p.as_os_str().is_empty() && !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
        }
        if f.exists() {
            // Update mtime by opening for write append
            let now = std::time::SystemTime::now();
            let file = std::fs::OpenOptions::new().write(true).open(f)?;
            file.set_modified(now)?;
            println!("  {} {} (mtime updated)", "OK".green(), f.display().to_string().cyan());
        } else {
            std::fs::File::create(f)?;
            println!("  {} {} (created)", "OK".green(), f.display().to_string().cyan());
        }
    }
    Ok(())
}
