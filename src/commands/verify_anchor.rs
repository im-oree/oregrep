use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::patch::unescape_arg;

#[derive(Args)]
pub struct VerifyAnchorArgs {
    /// File to search
    file: PathBuf,

    /// Text to find (supports \n for multiline anchors)
    #[arg(short = 'f', long)]
    find: String,

    /// Quiet: no output, only exit code (0=found, 1=not found)
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Print match count instead of found/not-found
    #[arg(short = 'c', long)]
    count: bool,

    /// Print first match line number only
    #[arg(short = 'n', long)]
    line: bool,

    /// Case-insensitive matching
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Treat find as a regular expression
    #[arg(short = 'x', long)]
    regex: bool,
}

pub fn run(args: VerifyAnchorArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let content = read_file_smart(&args.file)?;

    // Unescape \n \t \\ so multiline anchors work
    let find_unesc = unescape_arg(&args.find);

    // Normalize the content newlines to LF for consistent matching
    let content_norm = content.replace("\r\n", "\n").replace('\r', "\n");
    // Also normalize the find pattern
    let find_norm = find_unesc.replace("\r\n", "\n").replace('\r', "\n");

    if args.regex {
        run_regex(&args, &content_norm, &find_norm)
    } else {
        run_literal(&args, &content_norm, &find_norm)
    }
}

fn run_literal(args: &VerifyAnchorArgs, content: &str, find: &str) -> Result<()> {
    let (content_cmp, find_cmp) = if args.ignore_case {
        (content.to_lowercase(), find.to_lowercase())
    } else {
        (content.to_string(), find.to_string())
    };

    let match_count = content_cmp.match_indices(find_cmp.as_str()).count();

    // Find first line number — scan for first occurrence
    let first_line = if match_count > 0 {
        let first_byte = content_cmp
            .find(find_cmp.as_str())
            .unwrap_or(0);
        let before = &content[..first_byte];
        before.lines().count() + 1
    } else {
        0
    };

    report(args, match_count, first_line)
}

fn run_regex(args: &VerifyAnchorArgs, content: &str, pattern: &str) -> Result<()> {
    let pat = if args.ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };

    let re = Regex::new(&pat)
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

    let match_count = re.find_iter(content).count();

    let first_line = if match_count > 0 {
        let first_byte = re.find(content).map(|m| m.start()).unwrap_or(0);
        let before = &content[..first_byte];
        before.lines().count() + 1
    } else {
        0
    };

    report(args, match_count, first_line)
}

fn report(args: &VerifyAnchorArgs, match_count: usize, first_line: usize) -> Result<()> {
    let found = match_count > 0;

    if args.quiet {
        if !found {
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.count {
        if found {
            println!("{}", match_count);
        } else {
            println!("0");
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.line {
        if found {
            println!("{}", first_line);
        } else {
            eprintln!("{}", "NOT FOUND".red().bold());
            std::process::exit(1);
        }
        return Ok(());
    }

    // Default: human-readable output
    if found {
        println!(
            "{} line {} ({} match{})",
            "FOUND:".green().bold(),
            first_line.to_string().yellow(),
            match_count.to_string().cyan(),
            if match_count == 1 { "" } else { "es" }
        );
    } else {
        eprintln!(
            "{} {:?} not found in {}",
            "NOT FOUND:".red().bold(),
            args.find,
            args.file.display()
        );
        std::process::exit(1);
    }

    Ok(())
}
