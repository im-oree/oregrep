use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

/// Show every commit that touched anything matching PATTERN, going back N commits.
/// Uses `git log -S<pattern>` (pickaxe) to find introducing/removing changes.
#[derive(Args)]
pub struct SearchDiffArgs {
    /// Pattern to search across git history diffs
    pattern: String,

    /// How far back: HEAD~N (default) or a ref
    #[arg(default_value = "HEAD~30")]
    since: String,

    /// Show full diff for each commit
    #[arg(short = 'p', long)]
    patch: bool,

    /// Regex mode (default: literal pickaxe)
    #[arg(short = 'x', long)]
    regex: bool,

    /// Limit commit count
    #[arg(short = 'n', long, default_value = "50")]
    limit: usize,
}

pub fn run(args: SearchDiffArgs) -> Result<()> {
    ensure_repo()?;

    let limit_str = args.limit.to_string();
    let range = format!("{}..HEAD", args.since);

    let mut cmd: Vec<String> = vec![
        "log".to_string(),
        "--color=always".to_string(),
        "-n".to_string(),
        limit_str,
    ];

    if args.regex {
        cmd.push(format!("-G{}", args.pattern));
    } else {
        cmd.push(format!("-S{}", args.pattern));
        cmd.push("--pickaxe-all".to_string());
    }

    if args.patch {
        cmd.push("-p".to_string());
    } else {
        cmd.push("--pretty=format:%C(yellow)%h%C(reset) %C(cyan)%an%C(reset) %C(dim)%ar%C(reset) %s".to_string());
    }

    cmd.push(range);

    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = match git(&refs) {
        Ok(o) => o,
        Err(_e) => {
            // `since..HEAD` may predate the repo's first commit — retry over full history
            eprintln!(
                "{} range {} not found — searching full history",
                "Note:".yellow(),
                args.since.dimmed()
            );
            let mut full = cmd.clone();
            full.pop(); // drop the range argument
            let refs_full: Vec<&str> = full.iter().map(|s| s.as_str()).collect();
            git(&refs_full)?
        }
    };

    let mode = if args.regex { "regex" } else { "literal" };
    println!(
        "{} '{}' ({}) since {}",
        "search-diff:".cyan().bold(),
        args.pattern.yellow(),
        mode.dimmed(),
        args.since.dimmed()
    );
    println!();
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
    Ok(())
}
