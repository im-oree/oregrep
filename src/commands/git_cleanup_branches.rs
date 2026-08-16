use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitCleanupBranchesArgs {
    /// Branch to consider "the trunk" (default: main, then master)
    #[arg(short = 'b', long)]
    base: Option<String>,

    /// Also delete branches with no upstream
    #[arg(long)]
    include_orphans: bool,

    /// Force-delete unmerged branches too
    #[arg(long)]
    force: bool,

    #[arg(long)]
    dry_run: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitCleanupBranchesArgs) -> Result<()> {
    ensure_repo()?;

    let base = if let Some(b) = &args.base { b.clone() }
        else {
            let all = git(&["branch"]).unwrap_or_default();
            if all.lines().any(|l| l.trim_start_matches("* ").trim() == "main") { "main".to_string() }
            else if all.lines().any(|l| l.trim_start_matches("* ").trim() == "master") { "master".to_string() }
            else { anyhow::bail!("Could not detect base branch. Use --base"); }
        };

    // Merged branches
    let merged = git(&["branch", "--merged", &base])?;
    let mut to_delete: Vec<String> = Vec::new();
    for line in merged.lines() {
        let b = line.trim_start_matches("* ").trim().to_string();
        if b.is_empty() || b == base { continue; }
        to_delete.push(b);
    }

    // Orphans (no upstream)
    if args.include_orphans {
        let all = git(&["for-each-ref", "--format=%(refname:short) %(upstream)", "refs/heads/"])?;
        for line in all.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts.get(0).unwrap_or(&"").to_string();
            let has_upstream = parts.len() >= 2 && !parts[1].is_empty();
            if !has_upstream && name != base && !to_delete.contains(&name) {
                to_delete.push(name);
            }
        }
    }

    if to_delete.is_empty() {
        println!("{}", "No branches to clean up.".green());
        return Ok(());
    }

    println!("{} {} local branches:", "Will delete:".cyan().bold(), to_delete.len().to_string().yellow());
    for b in &to_delete { println!("  {} {}", "-".red(), b); }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing deleted]".yellow().bold());
        return Ok(());
    }

    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Delete {} branches?", to_delete.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let flag = if args.force { "-D" } else { "-d" };
    let mut deleted = 0usize;
    let mut errors = 0usize;
    for b in &to_delete {
        match git(&["branch", flag, b]) {
            Ok(_) => { deleted += 1; println!("  {} {}", "deleted".green(), b.dimmed()); }
            Err(e) => { errors += 1; eprintln!("  {} {}: {}", "err".red(), b, e); }
        }
    }
    println!("\n{} {} deleted, {} errors", "Done:".green().bold(), deleted.to_string().green(), errors.to_string().red());
    Ok(())
}
