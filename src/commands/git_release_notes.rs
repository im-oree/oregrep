use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitReleaseNotesArgs {
    /// Version/tag being released
    version: String,

    /// Previous tag/ref to compare from (default: previous tag)
    #[arg(short = 'p', long)]
    previous: Option<String>,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: GitReleaseNotesArgs) -> Result<()> {
    ensure_repo()?;
    let prev = if let Some(p) = args.previous.clone() { p }
        else {
            // Find last tag
            let tags = git(&["tag", "--sort=-creatordate"]).unwrap_or_default();
            tags.lines().next().unwrap_or("HEAD~20").to_string()
        };

    let log = git(&["log", &format!("{}..HEAD", prev), "--pretty=format:%h\t%an\t%s", "--no-merges"])?;
    let mut lines: Vec<(String, String, String)> = Vec::new();
    for l in log.lines() {
        let parts: Vec<&str> = l.splitn(3, '\t').collect();
        if parts.len() >= 3 {
            lines.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
        }
    }

    // Contributors
    let mut contributors: Vec<String> = lines.iter().map(|(_, a, _)| a.clone()).collect();
    contributors.sort();
    contributors.dedup();

    // Group by kind
    let re = regex::Regex::new(r"^(feat|fix|chore|docs|style|refactor|test|perf|build|ci|revert)(\([^)]+\))?!?:\s*(.+)$").unwrap();
    let mut groups: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for (_, _, s) in &lines {
        let kind = re.captures(s).map(|c| c[1].to_string()).unwrap_or_else(|| "other".to_string());
        groups.entry(kind).or_default().push(s.clone());
    }

    let mut out = String::new();
    out.push_str(&format!("# Release {}\n\n", args.version));
    out.push_str(&format!("_{} commits since {}_\n\n", lines.len(), prev));

    let order = ["feat", "fix", "perf", "refactor", "chore", "docs", "test", "style", "build", "ci", "revert", "other"];
    for kind in order.iter() {
        if let Some(entries) = groups.get(*kind) {
            if entries.is_empty() { continue; }
            out.push_str(&format!("## {}\n\n", kind_display(kind)));
            for s in entries { out.push_str(&format!("- {}\n", s)); }
            out.push('\n');
        }
    }

    out.push_str("## Contributors\n\n");
    for c in &contributors { out.push_str(&format!("- {}\n", c)); }

    match &args.output {
        Some(p) => {
            std::fs::write(p, &out)?;
            println!("{} {}", "Wrote:".green().bold(), p.display().to_string().cyan());
        }
        None => print!("{}", out),
    }
    Ok(())
}

fn kind_display(k: &str) -> &'static str {
    match k {
        "feat" => "New Features",
        "fix" => "Bug Fixes",
        "perf" => "Performance Improvements",
        "refactor" => "Refactoring",
        "chore" => "Chores",
        "docs" => "Documentation",
        "test" => "Tests",
        "style" => "Style",
        "build" => "Build",
        "ci" => "CI",
        "revert" => "Reverts",
        _ => "Other Changes",
    }
}
