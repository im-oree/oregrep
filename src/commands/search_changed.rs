use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::{is_binary, read_file_smart};
use crate::engine::git::{changed_files, ensure_repo};

#[derive(Args)]
pub struct SearchChangedArgs {
    /// Pattern to search for
    pattern: String,

    #[arg(short = 'F', long)]
    literal: bool,
    #[arg(short = 'i', long)]
    ignore_case: bool,
    #[arg(short = 'w', long)]
    word: bool,

    /// Only staged changes
    #[arg(long)]
    staged: bool,

    /// Only unstaged changes
    #[arg(long)]
    unstaged: bool,

    /// Only untracked changes
    #[arg(long)]
    untracked: bool,

    /// Show context lines
    #[arg(short = 'C', long, default_value = "0")]
    context: usize,

    /// Files only
    #[arg(short = 'l', long)]
    files_only: bool,
}

pub fn run(args: SearchChangedArgs) -> Result<()> {
    ensure_repo()?;
    let mut pattern = if args.literal { regex::escape(&args.pattern) } else { args.pattern.clone() };
    if args.word { pattern = format!(r"\b{}\b", pattern); }
    let re = RegexBuilder::new(&pattern).case_insensitive(args.ignore_case).build()?;

    let files = changed_files()?;
    let filtered: Vec<String> = files.into_iter().filter(|(status, _)| {
        let sc = status.chars().next().unwrap_or(' ');
        let uc = status.chars().nth(1).unwrap_or(' ');
        let is_untracked = status.starts_with('?');
        let is_staged = sc != ' ' && !is_untracked;
        let is_unstaged = uc != ' ' && !is_untracked;
        if args.staged || args.unstaged || args.untracked {
            (args.staged && is_staged) || (args.unstaged && is_unstaged) || (args.untracked && is_untracked)
        } else { true }
    }).map(|(_, p)| p).collect();

    let mut total_matches = 0usize;
    let mut files_matched = 0usize;

    for path_str in &filtered {
        let path = PathBuf::from(path_str);
        if !path.exists() || !path.is_file() { continue; }
        if let Ok(true) = is_binary(&path) { continue; }
        let content = match read_file_smart(&path) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();
        let matched: Vec<usize> = lines.iter().enumerate()
            .filter_map(|(i, l)| if re.is_match(l) { Some(i) } else { None })
            .collect();
        if matched.is_empty() { continue; }
        files_matched += 1;
        total_matches += matched.len();

        if args.files_only {
            println!("{}", path.display());
            continue;
        }

        println!("\n{}", path.display().to_string().cyan().bold());
        let mut printed: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for &m in &matched {
            let s = m.saturating_sub(args.context);
            let e = (m + args.context + 1).min(lines.len());
            for i in s..e {
                if printed.contains(&i) { continue; }
                printed.insert(i);
                let lineno = i + 1;
                if i == m {
                    let hl = re.replace_all(lines[i], |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("  {}: {}", lineno.to_string().green(), hl);
                } else {
                    println!("  {}| {}", lineno.to_string().dimmed(), lines[i].dimmed());
                }
            }
        }
    }
    eprintln!("\n{} matches in {} changed files",
        total_matches.to_string().yellow(),
        files_matched.to_string().yellow()
    );
    Ok(())
}
