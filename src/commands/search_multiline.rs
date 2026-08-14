use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct SearchMultilineArgs {
    /// Pattern (can span multiple lines with .* and \n)
    pattern: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'i', long)]
    ignore_case: bool,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    /// Show the actual matched text (not just filename)
    #[arg(short = 'p', long)]
    print_matches: bool,

    /// Show first N lines of each match
    #[arg(long, default_value = "5")]
    max_lines: usize,

    #[arg(short = 'l', long)]
    files_only: bool,
}

pub fn run(args: SearchMultilineArgs) -> Result<()> {
    let re = RegexBuilder::new(&args.pattern)
        .case_insensitive(args.ignore_case)
        .dot_matches_new_line(true)   // . matches \n
        .multi_line(true)
        .build()?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;

    let mut total_matches = 0usize;
    let mut files_matched = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let matches: Vec<_> = re.find_iter(&content).collect();
        if matches.is_empty() { continue; }
        files_matched += 1;
        total_matches += matches.len();

        if args.files_only {
            println!("{}", f.display());
            continue;
        }

        println!("\n{}  ({} matches)",
            f.display().to_string().cyan().bold(),
            matches.len().to_string().yellow()
        );
        if args.print_matches {
            for (idx, m) in matches.iter().enumerate() {
                let start_line = content[..m.start()].matches('\n').count() + 1;
                println!("  {} at line {}", format!("[{}]", idx + 1).dimmed(), start_line.to_string().green());
                for (i, line) in m.as_str().lines().enumerate() {
                    if i >= args.max_lines {
                        println!("    {}", "…".dimmed());
                        break;
                    }
                    println!("    {}", line.red());
                }
            }
        }
    }

    eprintln!("\n{} multiline matches in {} files",
        total_matches.to_string().yellow(),
        files_matched.to_string().yellow()
    );
    Ok(())
}
