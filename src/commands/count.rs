use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct CountArgs {
    /// Pattern (regex by default)
    pattern: String,

    /// Path (file or directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'F', long)]
    literal: bool,
    #[arg(short = 'i', long)]
    ignore_case: bool,
    #[arg(short = 'w', long)]
    word: bool,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Show per-file counts (default: only totals)
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: CountArgs) -> Result<()> {
    let mut pattern = if args.literal { regex::escape(&args.pattern) } else { args.pattern.clone() };
    if args.word { pattern = format!(r"\b{}\b", pattern); }
    let re = RegexBuilder::new(&pattern).case_insensitive(args.ignore_case).build()?;

    let files: Vec<PathBuf> = if args.path.is_file() {
        vec![args.path.clone()]
    } else {
        let cfg = WalkConfig {
            root: args.path.clone(),
            extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
            excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
            ..Default::default()
        };
        collect_files(&cfg)?
    };

    let mut total_matches = 0usize;
    let mut total_files = 0usize;
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let n = re.find_iter(&content).count();
        if n > 0 {
            total_files += 1;
            total_matches += n;
            if args.verbose {
                println!("{}: {}", f.display().to_string().cyan(), n.to_string().yellow());
            }
        }
    }
    println!("{}: {} matches in {} files",
        "Total".bold(),
        total_matches.to_string().green(),
        total_files.to_string().green()
    );
    Ok(())
}
