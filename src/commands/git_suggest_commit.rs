use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::commit_msg::{analyze_diff, compose_message, detect_convention};
use crate::engine::git::ensure_repo;

#[derive(Args)]
pub struct GitSuggestCommitArgs {
    #[arg(short = 's', long)]
    staged: bool,
}

pub fn run(args: GitSuggestCommitArgs) -> Result<()> {
    ensure_repo()?;
    let a = analyze_diff(args.staged)?;
    if a.files.is_empty() {
        println!("{}", "No changes to analyze.".yellow());
        return Ok(());
    }

    let style = detect_convention();
    println!("{}", "Suggested message:".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", compose_message(&a, &style, true));
    println!("{}", "─".repeat(60).dimmed());
    println!();
    println!("{}", "Rationale:".cyan().bold());
    println!("  {} {} files changed, +{} -{}", "•".dimmed(),
        a.files.len().to_string().yellow(),
        a.total_added.to_string().green(),
        a.total_removed.to_string().red());
    println!("  {} detected style: {}", "•".dimmed(), style.magenta());
    if a.is_docs_only { println!("  {} docs-only change", "•".dimmed()); }
    if a.is_test_only { println!("  {} test-only change", "•".dimmed()); }
    if a.is_config_only { println!("  {} config-only change", "•".dimmed()); }
    if a.is_deps_change { println!("  {} touches dependency files", "•".dimmed()); }
    if !a.new_files.is_empty() { println!("  {} {} new files", "•".dimmed(), a.new_files.len().to_string().green()); }
    if !a.deleted_files.is_empty() { println!("  {} {} deleted files", "•".dimmed(), a.deleted_files.len().to_string().red()); }
    if !a.new_symbols.is_empty() { println!("  {} {} new symbols detected: {}", "•".dimmed(), a.new_symbols.len(), a.new_symbols.iter().take(5).cloned().collect::<Vec<_>>().join(", ").dimmed()); }
    if !a.removed_symbols.is_empty() { println!("  {} {} removed symbols detected", "•".dimmed(), a.removed_symbols.len()); }
    println!("  {} file categories:", "•".dimmed());
    let mut buckets: Vec<_> = a.buckets.iter().collect();
    buckets.sort_by(|x, y| y.1.len().cmp(&x.1.len()));
    for (cat, files) in buckets.iter().take(5) {
        println!("      {} → {} file{}", cat.magenta(), files.len().to_string().yellow(), if files.len() > 1 { "s" } else { "" });
    }
    Ok(())
}
