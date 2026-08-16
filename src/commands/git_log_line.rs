use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

/// Show every commit that touched a specific line.
/// Uses `git log -L :line,line:file` to trace line history.
#[derive(Args)]
pub struct GitLogLineArgs {
    /// Location as file:line (e.g. src/foo.ts:42)
    location: String,

    /// Also show the diff for each commit
    #[arg(short = 'p', long)]
    patch: bool,

    /// Limit results
    #[arg(short = 'n', long, default_value = "20")]
    limit: usize,
}

pub fn run(args: GitLogLineArgs) -> Result<()> {
    ensure_repo()?;

    let (file, line) = parse_location(&args.location)?;
    let line_spec = format!("{},{}:{}", line, line, file);
    let limit_str = args.limit.to_string();

    let mut cmd: Vec<String> = vec![
        "log".to_string(),
        "--color=always".to_string(),
        "-L".to_string(),
        line_spec,
        "-n".to_string(),
        limit_str,
    ];

    if !args.patch {
        cmd.push("-s".to_string()); // suppress diff output
    }

    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;

    println!("{} history of {}:{}", "→".cyan(), file.cyan().bold(), line.to_string().yellow());
    println!();
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
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
