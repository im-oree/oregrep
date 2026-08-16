use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

/// Compare how a pattern appears/differs across two git branches.
#[derive(Args)]
pub struct CompareBranchesArgs {
    branch_a: String,
    branch_b: String,

    /// Optional pattern to filter the diff by
    pattern: Option<String>,

    /// Show file names only
    #[arg(short = 'l', long)]
    files_only: bool,

    /// Limit context lines
    #[arg(short = 'U', long, default_value = "3")]
    unified: usize,
}

pub fn run(args: CompareBranchesArgs) -> Result<()> {
    ensure_repo()?;

    let range = format!("{}..{}", args.branch_a, args.branch_b);
    let unified_str = format!("--unified={}", args.unified);

    if args.files_only {
        let out = git(&["diff", "--name-only", &range])?;
        let files: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();

        if let Some(ref pat) = args.pattern {
            let filtered: Vec<&&str> = files.iter().filter(|f| {
                let content = git(&["show", &format!("{}:{}", args.branch_b, f)]).unwrap_or_default();
                content.contains(pat)
            }).collect();
            println!("{} {} files differ AND contain '{}'",
                "compare-branches:".bold(),
                filtered.len().to_string().yellow(),
                pat.cyan()
            );
            for f in filtered {
                println!("  {}", f.cyan());
            }
        } else {
            println!("{} {} files differ between {} and {}",
                "compare-branches:".bold(),
                files.len().to_string().yellow(),
                args.branch_a.cyan(),
                args.branch_b.cyan()
            );
            for f in files {
                println!("  {}", f.cyan());
            }
        }
        return Ok(());
    }

    let diff = git(&["diff", "--color=always", &unified_str, &range])?;

    if let Some(ref pat) = args.pattern {
        // Filter diff hunks to only those containing the pattern
        let mut in_hunk = false;
        let mut hunk_lines: Vec<String> = Vec::new();
        let mut hunk_has_pattern = false;
        let mut current_file = String::new();

        for line in diff.lines() {
            if line.starts_with("diff --git") || line.starts_with("+++ ") {
                if in_hunk && hunk_has_pattern {
                    for l in &hunk_lines { println!("{}", l); }
                }
                hunk_lines.clear();
                hunk_has_pattern = false;
                in_hunk = false;
                if line.starts_with("+++ ") {
                    current_file = line[6..].to_string();
                }
                println!("{}", line);
                continue;
            }
            if line.starts_with("@@") {
                if in_hunk && hunk_has_pattern {
                    for l in &hunk_lines { println!("{}", l); }
                }
                hunk_lines.clear();
                hunk_has_pattern = false;
                in_hunk = true;
                hunk_lines.push(line.to_string());
                let _ = current_file;
                continue;
            }
            if in_hunk {
                if line.contains(pat) { hunk_has_pattern = true; }
                hunk_lines.push(line.to_string());
            }
        }
        if in_hunk && hunk_has_pattern {
            for l in &hunk_lines { println!("{}", l); }
        }
    } else {
        print!("{}", diff);
    }

    Ok(())
}
