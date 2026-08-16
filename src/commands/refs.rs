use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct RefsArgs {
    symbol: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Show N context lines around each match
    #[arg(short = 'C', long, default_value = "0")]
    context: usize,

    /// Files only
    #[arg(short = 'l', long)]
    files_only: bool,

    /// Include definitions (default: skip lines that look like definitions)
    #[arg(short = 'D', long)]
    include_defs: bool,
}

pub fn run(args: RefsArgs) -> Result<()> {
    let re = RegexBuilder::new(&format!(r"\b{}\b", regex::escape(&args.symbol))).build()?;
    let def_re = regex::Regex::new(&format!(
        r"(?:function\s+{}\b|(?:const|let|var)\s+{}\s*[:=]|class\s+{}\b|interface\s+{}\b|type\s+{}\b|enum\s+{}\b|fn\s+{}\b|struct\s+{}\b|trait\s+{}\b|def\s+{}\b)",
        regex::escape(&args.symbol), regex::escape(&args.symbol), regex::escape(&args.symbol),
        regex::escape(&args.symbol), regex::escape(&args.symbol), regex::escape(&args.symbol),
        regex::escape(&args.symbol), regex::escape(&args.symbol), regex::escape(&args.symbol),
        regex::escape(&args.symbol)
    ))?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;
    let mut total_refs = 0usize;
    let mut files_with = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();
        let matches: Vec<usize> = lines.iter().enumerate().filter_map(|(i, l)| {
            if !re.is_match(l) { return None; }
            if !args.include_defs && def_re.is_match(l) { return None; }
            Some(i)
        }).collect();
        if matches.is_empty() { continue; }
        files_with += 1;
        total_refs += matches.len();
        if args.files_only { println!("{}", f.display()); continue; }
        println!("\n{}", f.display().to_string().cyan().bold());
        let mut printed = std::collections::HashSet::new();
        for &m in &matches {
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
    eprintln!("\n{} {} references in {} files", "Total:".bold(), total_refs.to_string().yellow(), files_with.to_string().yellow());
    Ok(())
}
