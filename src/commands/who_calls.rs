use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Reverse call graph: show every call site of a symbol with context.
/// Equivalent to `refs` + `cat-around` but in one command with cleaner output.
#[derive(Args)]
pub struct WhoCallsArgs {
    /// Symbol to find call sites for
    pub symbol: String,

    /// Path to search
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(short = 'e', long)]
    pub ext: Option<String>,

    #[arg(short = 'x', long)]
    pub exclude: Option<String>,

    /// Lines of context around each call site
    #[arg(short = 'C', long, default_value = "3")]
    pub context: usize,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Skip the file where the symbol is DEFINED (only external callers)
    #[arg(long)]
    pub external_only: bool,

    /// Files-only summary (no code)
    #[arg(short = 'l', long)]
    pub files_only: bool,

    /// Cap calls shown per file (default: 10)
    #[arg(long, default_value = "10")]
    pub per_file: usize,
}

pub fn run(args: WhoCallsArgs) -> Result<()> {
    // Match `symbol(` — a call, not a mention
    let call_pattern = format!(r"\b{}\s*\(", regex::escape(&args.symbol));
    let call_re = RegexBuilder::new(&call_pattern)
        .case_insensitive(args.ignore_case)
        .build()?;

    // Match a DEFINITION so we can skip that line
    let def_pattern = format!(
        r"^\s*(?:pub\s+)?(?:export\s+)?(?:async\s+)?(?:function|fn|def|const|let|var|class|impl)\s+{}\b",
        regex::escape(&args.symbol)
    );
    let def_re = RegexBuilder::new(&def_pattern).build()?;

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

    let mut total_calls = 0usize;
    let mut files_with_calls = 0usize;
    let mut definition_file: Option<PathBuf> = None;

    // First pass: find the definition (for --external-only)
    if args.external_only {
        for f in &files {
            let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
            if def_re.is_match(&content) {
                definition_file = Some(f.clone());
                break;
            }
        }
    }

    println!("{} {} in {}",
        "who-calls:".cyan().bold(),
        args.symbol.yellow(),
        args.path.display().to_string().dimmed()
    );

    for f in &files {
        if args.external_only && Some(f) == definition_file.as_ref() { continue; }

        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        let hits: Vec<usize> = lines.iter().enumerate()
            .filter_map(|(i, line)| {
                let t = line.trim_start();
                if t.starts_with("//") || t.starts_with("*") || t.starts_with("#") { return None; }
                if def_re.is_match(line) { return None; }  // skip definition line
                if call_re.is_match(line) { Some(i) } else { None }
            })
            .collect();

        if hits.is_empty() { continue; }
        files_with_calls += 1;
        total_calls += hits.len();

        if args.files_only {
            println!("  {} ({})", f.display().to_string().cyan(), hits.len().to_string().yellow());
            continue;
        }

        println!("\n{} ({} calls)", f.display().to_string().cyan().bold(), hits.len().to_string().yellow());
        let shown = hits.iter().take(args.per_file);
        for &h in shown {
            let s = h.saturating_sub(args.context);
            let e = (h + args.context + 1).min(lines.len());
            for i in s..e {
                let ln = i + 1;
                let text = lines[i];
                if i == h {
                    let hl = call_re.replace_all(text, |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("  {:>5} │ {}", ln.to_string().green(), hl);
                } else {
                    println!("  {:>5} │ {}", ln.to_string().dimmed(), text.dimmed());
                }
            }
            println!();
        }
        if hits.len() > args.per_file {
            println!("  {} ({} more calls in this file, use --per-file to see more)",
                "...".dimmed(), (hits.len() - args.per_file).to_string().dimmed());
        }
    }

    eprintln!("{} {} call sites across {} files{}",
        "Total:".bold(),
        total_calls.to_string().yellow(),
        files_with_calls.to_string().yellow(),
        if let Some(df) = &definition_file {
            format!(" (defined in {})", df.display().to_string().dimmed())
        } else { String::new() }
    );

    Ok(())
}
