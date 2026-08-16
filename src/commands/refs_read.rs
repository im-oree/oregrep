use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Find only READ sites of a symbol (excludes writes/assignments).
#[derive(Args)]
pub struct RefsReadArgs {
    symbol: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    #[arg(short = 'C', long, default_value = "1")]
    context: usize,

    #[arg(short = 'i', long)]
    ignore_case: bool,

    #[arg(short = 'l', long)]
    lines_only: bool,
}

pub fn run(args: RefsReadArgs) -> Result<()> {
    let p = regex::escape(&args.symbol);
    // Match the symbol as a plain word (the regex crate has no look-around,
    // so write/annotation exclusion is done with separate regexes below).
    let read_pattern = format!(r"\b{}\b", p);
    let re = RegexBuilder::new(&read_pattern)
        .case_insensitive(args.ignore_case)
        .multi_line(true)
        .build()?;

    // Write sites: X = (not ==), X +=, X++, X--
    let write_pattern = format!(r"\b{}\s*(=[^=]|=$|[+\-*/]=|\+\+|--)", p);
    let write_re = RegexBuilder::new(&write_pattern)
        .case_insensitive(args.ignore_case)
        .build()?;

    // Object-literal / type annotation keys: X:
    let annot_pattern = format!(r"\b{}\s*:", p);
    let annot_re = RegexBuilder::new(&annot_pattern)
        .case_insensitive(args.ignore_case)
        .build()?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: false,
        respect_gitignore: true,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;

    let mut total = 0usize;
    let mut files_hit = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        let mut hits: Vec<usize> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let stripped = line.trim_start();
            if stripped.starts_with("//") || stripped.starts_with("*") { continue; }
            // Include if read pattern matches AND it's not a write/annotation site
            if re.is_match(line) && !write_re.is_match(line) && !annot_re.is_match(line) {
                hits.push(i);
            }
        }

        if hits.is_empty() { continue; }
        files_hit += 1;
        total += hits.len();

        if args.lines_only {
            for i in &hits {
                println!("{}:{}", f.display(), i + 1);
            }
            continue;
        }

        println!("\n{}", f.display().to_string().cyan().bold());
        let mut printed = std::collections::HashSet::new();
        for &h in &hits {
            let s = h.saturating_sub(args.context);
            let e = (h + args.context + 1).min(lines.len());
            for i in s..e {
                if printed.contains(&i) { continue; }
                printed.insert(i);
                let ln = i + 1;
                let text = lines[i];
                if i == h {
                    let hl = re.replace_all(text, |c: &regex::Captures| c[0].cyan().bold().to_string());
                    println!("  {}: {}", ln.to_string().green(), hl);
                } else {
                    println!("  {}| {}", ln.to_string().dimmed(), text.dimmed());
                }
            }
        }
    }

    eprintln!("\n{} {} read sites for {:?} in {} files",
        "refs-read:".bold(),
        total.to_string().yellow(),
        args.symbol,
        files_hit.to_string().yellow()
    );

    Ok(())
}
