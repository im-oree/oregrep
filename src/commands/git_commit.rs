use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{changed_files, ensure_repo, git, FileFilter};

#[derive(Args)]
pub struct GitCommitArgs {
    /// Commit message
    #[arg(short = 'm', long)]
    message: String,

    /// Commit all currently modified tracked files (git commit -am)
    #[arg(long)]
    all: bool,

    /// Only files matching (substring). Auto-stages then commits.
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

    /// Preview which files would be committed
    #[arg(long)]
    dry_run: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitCommitArgs) -> Result<()> {
    ensure_repo()?;

    if args.all && (args.only.is_none() && args.except.is_none() && args.starts.is_none() && args.changed_in.is_none()) {
        // Simple git commit -am
        if args.dry_run {
            println!("{}", "[DRY RUN] Would run: git commit -am '<msg>'".yellow());
            return Ok(());
        }
        git(&["commit", "-am", &args.message])?;
        println!("{}", "Committed.".green().bold());
        return Ok(());
    }

    if args.only.is_none() && args.except.is_none() && args.starts.is_none() && args.changed_in.is_none() {
        // Just commit whatever is already staged
        if args.dry_run {
            println!("{}", "[DRY RUN] Would run: git commit -m '<msg>'".yellow());
            return Ok(());
        }
        git(&["commit", "-m", &args.message])?;
        println!("{}", "Committed staged changes.".green().bold());
        return Ok(());
    }

    // Filter-based: stage matching files, then commit them
    let files = changed_files()?;
    let filter = FileFilter {
        only: args.only.clone(),
        except: args.except.clone(),
        starts: args.starts.clone(),
        matching: args.matching.clone(),
        changed_in: args.changed_in.clone(),
    };
    let paths: Vec<String> = files.iter().map(|(_, p)| p.clone()).collect();
    let kept = filter.apply(paths);

    if kept.is_empty() {
        println!("{}", "No matching files to commit.".yellow());
        return Ok(());
    }

    println!("{}", "Will commit:".cyan());
    for p in &kept {
        println!("  {}", p.cyan());
    }
    println!("{} {}", "Message:".dimmed(), args.message.yellow());

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing committed]".yellow().bold());
        return Ok(());
    }

    let ok = crate::engine::confirm::confirm(
        &format!("Commit {} files?", kept.len()),
        args.yes,
    )?;
    if !ok {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    // Stage
    let mut stage_cmd: Vec<String> = vec!["add".to_string(), "--".to_string()];
    for p in &kept { stage_cmd.push(p.clone()); }
    let refs: Vec<&str> = stage_cmd.iter().map(|s| s.as_str()).collect();
    git(&refs)?;

    // Commit only those paths
    let mut commit_cmd: Vec<String> = vec!["commit".to_string(), "-m".to_string(), args.message.clone(), "--".to_string()];
    for p in &kept { commit_cmd.push(p.clone()); }
    let refs2: Vec<&str> = commit_cmd.iter().map(|s| s.as_str()).collect();
    git(&refs2)?;

    println!("{} {} files committed", "OK".green().bold(), kept.len().to_string().yellow());
    Ok(())
}
