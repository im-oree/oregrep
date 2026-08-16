use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

/// Blame a specific line and show its introducing commit with full message.
/// Format: file:line (parses like other file:line commands).
#[derive(Args)]
pub struct GitBlameLineArgs {
    /// Location as file:line (e.g. src/foo.ts:42)
    location: String,

    /// Show full commit message + diff of the introducing commit
    #[arg(short = 'f', long)]
    full: bool,
}

pub fn run(args: GitBlameLineArgs) -> Result<()> {
    ensure_repo()?;

    let (file, line) = parse_location(&args.location)?;

    // Get blame for exactly this line
    let line_str = line.to_string();
    let blame_range = format!("{},{}", line_str, line_str);
    let blame_out = git(&["blame", "-p", "-L", &blame_range, "--", &file])?;

    // Parse porcelain output — first line is: <sha> <src_line> <res_line> [num_lines]
    let first_line = blame_out.lines().next().unwrap_or("");
    let sha = first_line.split_whitespace().next().unwrap_or("").to_string();

    if sha.is_empty() {
        anyhow::bail!("Could not determine commit for {}:{}", file, line);
    }

    // Uncommitted line — blame reports an all-zero sha ("Not Committed Yet")
    if sha.chars().all(|c| c == '0') {
        println!("{} {}:{}", "→".cyan(), file.cyan().bold(), line.to_string().yellow());
        println!();
        println!("{}", "This line is part of an uncommitted change (working tree).".yellow());
        println!("{}", "No commit to show yet — commit the change first, or blame another line.".dimmed());
        return Ok(());
    }

    // Get commit info
    let author = git(&["log", "-1", "--pretty=format:%an", &sha])?.trim().to_string();
    let email = git(&["log", "-1", "--pretty=format:%ae", &sha])?.trim().to_string();
    let date = git(&["log", "-1", "--pretty=format:%ad", "--date=short", &sha])?.trim().to_string();
    let subject = git(&["log", "-1", "--pretty=format:%s", &sha])?.trim().to_string();
    let body = git(&["log", "-1", "--pretty=format:%b", &sha])?;

    // Get actual line content
    let line_content = git(&["show", &format!("{}:{}", sha, file)])
        .ok()
        .and_then(|content| {
            content.lines().nth(line - 1).map(|l| l.to_string())
        })
        .unwrap_or_else(|| "<line content unavailable>".to_string());

    println!("{} {}:{}", "→".cyan(), file.cyan().bold(), line.to_string().yellow());
    println!();
    println!("{}     {}", "Commit:".dimmed(), sha.yellow());
    println!("{}     {} <{}>", "Author:".dimmed(), author.cyan(), email.dimmed());
    println!("{}       {}", "Date:".dimmed(), date.dimmed());
    println!("{}    {}", "Subject:".dimmed(), subject.bold());
    if !body.trim().is_empty() {
        println!();
        for l in body.lines() {
            println!("  {}", l.dimmed());
        }
    }
    println!();
    println!("{}", "Line content:".dimmed());
    println!("  {}", line_content);

    if args.full {
        println!();
        println!("{}", "Full commit diff:".dimmed());
        let diff = git(&["show", "--color=always", &sha])?;
        print!("{}", diff);
    }

    Ok(())
}

fn parse_location(s: &str) -> Result<(String, usize)> {
    let bytes = s.as_bytes();
    for i in (0..s.len()).rev() {
        if bytes[i] == b':' {
            let after = &s[i + 1..];
            if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
                let line: usize = after.parse()?;
                return Ok((s[..i].to_string(), line));
            }
        }
    }
    anyhow::bail!("Invalid location: {}. Use file:line format.", s);
}
