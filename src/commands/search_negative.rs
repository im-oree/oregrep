use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct SearchNegativeArgs {
    /// Pattern that should NOT appear
    pattern: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'F', long)]
    literal: bool,
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

    /// Also require this pattern to be present (find files that contain X but NOT Y)
    #[arg(short = 'r', long)]
    require: Option<String>,

    #[arg(short = 'l', long)]
    files_only: bool,
}

pub fn run(args: SearchNegativeArgs) -> Result<()> {
    let neg_pat = if args.literal { regex::escape(&args.pattern) } else { args.pattern.clone() };
    let neg_re = RegexBuilder::new(&neg_pat).case_insensitive(args.ignore_case).build()?;
    let req_re = args.require.as_ref().map(|p| {
        let pat = if args.literal { regex::escape(p) } else { p.clone() };
        RegexBuilder::new(&pat).case_insensitive(args.ignore_case).build()
    }).transpose()?;

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

    let mut hits = 0usize;
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        if neg_re.is_match(&content) { continue; }
        if let Some(r) = &req_re {
            if !r.is_match(&content) { continue; }
        }
        hits += 1;
        if args.files_only {
            println!("{}", f.display());
        } else {
            println!("{}", f.display().to_string().cyan());
        }
    }
    eprintln!("\n{} files DO NOT contain '{}'{}",
        hits.to_string().yellow(),
        args.pattern,
        args.require.as_ref().map(|r| format!(" (and DO contain '{}')", r)).unwrap_or_default()
    );
    Ok(())
}
