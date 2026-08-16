use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitChangelogArgs {
    /// Since tag/commit/date (e.g. "v1.0.0", "HEAD~50", "2 weeks ago")
    #[arg(short = 's', long)]
    since: Option<String>,

    /// Until tag/commit/date
    #[arg(short = 'u', long)]
    until: Option<String>,

    /// Group by conventional-commit type (feat, fix, chore, etc.)
    #[arg(short = 'g', long, default_value = "true")]
    group: bool,

    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Include commit hash
    #[arg(short = 'H', long)]
    hash: bool,

    /// Include author name
    #[arg(short = 'a', long)]
    author: bool,
}

pub fn run(args: GitChangelogArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["log".to_string(), "--pretty=format:%h\t%an\t%s".to_string(), "--no-merges".to_string()];
    if let Some(s) = &args.since {
        // If since looks like a ref, use range; else --since=
        if s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '~' || c == '^' || c == '_' || c == '-') && !s.contains(' ') {
            let range = if let Some(u) = &args.until { format!("{}..{}", s, u) } else { format!("{}..HEAD", s) };
            cmd.push(range);
        } else {
            cmd.push(format!("--since={}", s));
            if let Some(u) = &args.until { cmd.push(format!("--until={}", u)); }
        }
    } else if let Some(u) = &args.until {
        cmd.push(format!("--until={}", u));
    }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;

    let mut groups: std::collections::BTreeMap<String, Vec<(String, String, String)>> = std::collections::BTreeMap::new();
    let re = regex::Regex::new(r"^(feat|fix|chore|docs|style|refactor|test|perf|build|ci|revert)(\([^)]+\))?!?:\s*(.+)$").unwrap();

    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 3 { continue; }
        let hash = parts[0].to_string();
        let author = parts[1].to_string();
        let subject = parts[2].to_string();
        let kind = re.captures(&subject).map(|c| c[1].to_string()).unwrap_or_else(|| "other".to_string());
        groups.entry(kind).or_default().push((hash, author, subject));
    }

    let mut result = String::new();
    result.push_str("# Changelog\n\n");
    if let (Some(s), Some(u)) = (&args.since, &args.until) {
        result.push_str(&format!("Range: {} → {}\n\n", s, u));
    } else if let Some(s) = &args.since {
        result.push_str(&format!("Since: {}\n\n", s));
    }
    if args.group {
        let order = ["feat", "fix", "perf", "refactor", "chore", "docs", "test", "style", "build", "ci", "revert", "other"];
        for kind in order.iter() {
            if let Some(entries) = groups.get(*kind) {
                if entries.is_empty() { continue; }
                result.push_str(&format!("## {}\n\n", kind_display(kind)));
                for (h, a, s) in entries {
                    let mut line = format!("- {}", s);
                    if args.hash { line.push_str(&format!(" ({})", h)); }
                    if args.author { line.push_str(&format!(" — {}", a)); }
                    line.push('\n');
                    result.push_str(&line);
                }
                result.push('\n');
            }
        }
    } else {
        for (_, entries) in groups {
            for (h, a, s) in entries {
                let mut line = format!("- {}", s);
                if args.hash { line.push_str(&format!(" ({})", h)); }
                if args.author { line.push_str(&format!(" — {}", a)); }
                line.push('\n');
                result.push_str(&line);
            }
        }
    }

    match &args.output {
        Some(p) => {
            std::fs::write(p, &result)?;
            println!("{} {}", "Wrote:".green().bold(), p.display().to_string().cyan());
        }
        None => print!("{}", result),
    }
    Ok(())
}

fn kind_display(k: &str) -> &'static str {
    match k {
        "feat" => "Features",
        "fix" => "Fixes",
        "perf" => "Performance",
        "refactor" => "Refactoring",
        "chore" => "Chores",
        "docs" => "Documentation",
        "test" => "Tests",
        "style" => "Style",
        "build" => "Build",
        "ci" => "CI",
        "revert" => "Reverts",
        _ => "Other",
    }
}
