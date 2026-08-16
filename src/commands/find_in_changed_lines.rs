use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;

use crate::engine::git::{ensure_repo, git};

/// Find a pattern only in lines that were changed (added or modified) in the
/// git diff. Answers "did MY recent changes touch anything matching X?"
#[derive(Args)]
pub struct FindInChangedLinesArgs {
    /// Pattern to search for
    pattern: String,

    /// Compare against this ref (default: HEAD)
    #[arg(default_value = "HEAD")]
    base: String,

    /// Include staged changes
    #[arg(long, default_value = "true")]
    staged: bool,

    /// Include unstaged changes
    #[arg(long, default_value = "true")]
    unstaged: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Treat pattern as literal
    #[arg(short = 'F', long)]
    literal: bool,
}

pub fn run(args: FindInChangedLinesArgs) -> Result<()> {
    ensure_repo()?;

    let pat = if args.literal {
        regex::escape(&args.pattern)
    } else {
        args.pattern.clone()
    };
    let re = RegexBuilder::new(&pat)
        .case_insensitive(args.ignore_case)
        .build()?;

    let mut all_diff = String::new();

    if args.staged {
        if let Ok(s) = git(&["diff", "--cached", "--unified=0", &args.base]) {
            all_diff.push_str(&s);
        }
    }
    if args.unstaged {
        if let Ok(s) = git(&["diff", "--unified=0", &args.base]) {
            all_diff.push_str(&s);
        }
    }

    if all_diff.trim().is_empty() {
        println!("{} no changes to search", "Nothing:".yellow());
        return Ok(());
    }

    let mut current_file = String::new();
    let mut current_line = 0usize;
    let mut matches = 0usize;

    for line in all_diff.lines() {
        if line.starts_with("+++ b/") {
            current_file = line[6..].to_string();
            continue;
        }
        if line.starts_with("@@") {
            // @@ -old,count +new,count @@
            if let Some(new_part) = line.split('+').nth(1) {
                let num_str: String = new_part.chars().take_while(|c| c.is_ascii_digit()).collect();
                current_line = num_str.parse().unwrap_or(0);
            }
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            let content = &line[1..];
            if re.is_match(content) {
                matches += 1;
                let highlighted = re.replace_all(content, |c: &regex::Captures| {
                    c[0].red().bold().to_string()
                });
                println!(
                    "{}:{}: {}",
                    current_file.cyan(),
                    current_line.to_string().green(),
                    highlighted
                );
            }
            current_line += 1;
        }
    }

    eprintln!(
        "\n{} {} matches in changed lines (vs {})",
        "Found:".bold(),
        matches.to_string().yellow(),
        args.base.dimmed()
    );

    Ok(())
}
