use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct SearchOrArgs {
    /// Patterns (any one match counts). Repeat -p N times.
    #[arg(short = 'p', long = "pattern")]
    patterns: Vec<String>,

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
    #[arg(short = 'l', long)]
    files_only: bool,
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: SearchOrArgs) -> Result<()> {
    if args.patterns.is_empty() {
        anyhow::bail!("Provide at least one -p PATTERN");
    }
    let regexes: Vec<regex::Regex> = args.patterns.iter().map(|p| {
        let pat = if args.literal { regex::escape(p) } else { p.clone() };
        RegexBuilder::new(&pat).case_insensitive(args.ignore_case).build()
    }).collect::<Result<Vec<_>, _>>()?;

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
        let matched: Vec<&String> = args.patterns.iter().zip(regexes.iter())
            .filter_map(|(p, r)| if r.is_match(&content) { Some(p) } else { None })
            .collect();
        if matched.is_empty() { continue; }
        hits += 1;
        if args.files_only {
            println!("{}", f.display());
        } else if args.verbose {
            let names: Vec<String> = matched.iter().map(|s| s.to_string()).collect();
            println!("{}  ({})",
                f.display().to_string().cyan(),
                names.join(" | ").dimmed()
            );
        } else {
            println!("{}", f.display().to_string().cyan());
        }
    }
    eprintln!("\n{} files match ANY of {} patterns",
        hits.to_string().yellow(),
        args.patterns.len().to_string().yellow()
    );
    Ok(())
}
