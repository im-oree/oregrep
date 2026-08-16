use anyhow::Result;
use clap::Args;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::backup::list_backups;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct DiffArgs {
    /// First file (or "backup" to diff current file against latest backup)
    pub file_a: PathBuf,

    /// Second file (omit if using --backup)
    pub file_b: Option<PathBuf>,

    /// Diff current file against its latest backup
    #[arg(long)]
    pub backup: bool,

    /// Specific backup label to compare against
    #[arg(long)]
    pub label: Option<String>,

    /// Show line numbers
    #[arg(short = 'n', long, default_value = "true")]
    pub number: bool,

    /// Number of context lines (default 3)
    #[arg(short = 'C', long, default_value = "3")]
    pub context: usize,

    /// Stats only (additions, deletions counts)
    #[arg(short = 's', long)]
    pub stats: bool,
}

pub fn run(args: DiffArgs) -> Result<()> {
    let (path_a, path_b) = resolve_paths(&args)?;

    if !path_a.exists() {
        anyhow::bail!("File not found: {}", path_a.display());
    }
    if !path_b.exists() {
        anyhow::bail!("File not found: {}", path_b.display());
    }

    let content_a = read_file_smart(&path_a)?;
    let content_b = read_file_smart(&path_b)?;

    let diff = TextDiff::from_lines(&content_a, &content_b);

    let mut added = 0;
    let mut removed = 0;

    if args.stats {
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => added += 1,
                ChangeTag::Delete => removed += 1,
                _ => {}
            }
        }
        println!("{} vs {}",
            path_a.display().to_string().cyan(),
            path_b.display().to_string().cyan()
        );
        println!("  {} added, {} removed",
            format!("+{}", added).green(),
            format!("-{}", removed).red()
        );
        return Ok(());
    }

    println!("{} {} {}",
        "---".red(),
        path_a.display().to_string().cyan(),
        format!("({} bytes)", content_a.len()).dimmed()
    );
    println!("{} {} {}",
        "+++".green(),
        path_b.display().to_string().cyan(),
        format!("({} bytes)", content_b.len()).dimmed()
    );

    for group in diff.grouped_ops(args.context) {
        println!("{}", "@@".magenta());
        for op in group {
            for change in diff.iter_changes(&op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => { removed += 1; "-".red().to_string() }
                    ChangeTag::Insert => { added += 1; "+".green().to_string() }
                    ChangeTag::Equal => " ".dimmed().to_string(),
                };
                let line_a = change.old_index().map(|i| (i + 1).to_string()).unwrap_or_default();
                let line_b = change.new_index().map(|i| (i + 1).to_string()).unwrap_or_default();
                let text = change.value().trim_end_matches('\n');
                let colored_text = match change.tag() {
                    ChangeTag::Delete => text.red().to_string(),
                    ChangeTag::Insert => text.green().to_string(),
                    ChangeTag::Equal => text.dimmed().to_string(),
                };
                if args.number {
                    println!("{:>5} {:>5} {} {}",
                        line_a.dimmed(),
                        line_b.dimmed(),
                        sign,
                        colored_text
                    );
                } else {
                    println!("{} {}", sign, colored_text);
                }
            }
        }
    }

    println!("\n{} added, {} removed",
        format!("+{}", added).green(),
        format!("-{}", removed).red()
    );

    Ok(())
}

fn resolve_paths(args: &DiffArgs) -> Result<(PathBuf, PathBuf)> {
    if args.backup || args.label.is_some() {
        let backups = list_backups(&args.file_a)?;
        if backups.is_empty() {
            anyhow::bail!("No backups found for {}", args.file_a.display());
        }

        let backup_path = if let Some(label) = &args.label {
            let parent = args.file_a.parent().unwrap_or_else(|| std::path::Path::new("."));
            let fname = args.file_a.file_name().unwrap().to_string_lossy();
            let candidate = parent.join(format!("{}.bak{}", fname, label));
            if !candidate.exists() {
                anyhow::bail!("Backup not found: {}", candidate.display());
            }
            candidate
        } else {
            backups
                .iter()
                .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
                .cloned()
                .unwrap()
        };

        // diff shows backup as "old" (left) and current as "new" (right)
        return Ok((backup_path, args.file_a.clone()));
    }

    let b = args
        .file_b
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Second file required (or use --backup)"))?;

    Ok((args.file_a.clone(), b))
}
