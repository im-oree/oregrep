use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{changed_files, ensure_repo, git, FileFilter};

#[derive(Args)]
pub struct GitStageArgs {
    /// Stage all changed files
    #[arg(long)]
    all: bool,

    /// Only files matching (substring)
    #[arg(long)]
    only: Option<String>,

    /// Exclude files matching
    #[arg(long)]
    except: Option<String>,

    /// Only files whose basename starts with
    #[arg(long)]
    starts: Option<String>,

    /// Only files in this subdirectory
    #[arg(long)]
    changed_in: Option<String>,

    /// Only files whose content contains this substring
    #[arg(long)]
    matching: Option<String>,

    /// Preview which files would be staged
    #[arg(long)]
    dry_run: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitStageArgs) -> Result<()> {
    ensure_repo()?;

    if !args.all && args.only.is_none() && args.except.is_none() &&
       args.starts.is_none() && args.changed_in.is_none() {
        anyhow::bail!("Provide --all or one of --only / --except / --starts / --changed-in");
    }

    let files = changed_files()?;
    let filter = FileFilter {
        only: args.only,
        except: args.except,
        starts: args.starts,
        matching: args.matching,
        changed_in: args.changed_in,
    };
    let paths: Vec<String> = files.iter().map(|(_, p)| p.clone()).collect();
    let kept = filter.apply(paths);

    if kept.is_empty() {
        println!("{}", "No matching files.".yellow());
        return Ok(());
    }

    println!("{}", "Will stage:".cyan());
    for p in &kept {
        println!("  {}", p.cyan());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing staged]".yellow().bold());
        return Ok(());
    }

    let ok = crate::engine::confirm::confirm(
        &format!("Stage {} files?", kept.len()),
        args.yes,
    )?;
    if !ok {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    let mut cmd: Vec<String> = vec!["add".to_string(), "--".to_string()];
    for p in &kept { cmd.push(p.clone()); }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    git(&refs)?;
    println!("{} {} files staged", "OK".green().bold(), kept.len().to_string().yellow());
    Ok(())
}
