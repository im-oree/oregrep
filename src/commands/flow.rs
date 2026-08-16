use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Show data flow for a symbol: definition, callers, what those callers do.
/// Simpler than a full call graph — regex-based, fast, good enough.
#[derive(Args)]
pub struct FlowArgs {
    symbol: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Depth of caller expansion (default: 1)
    #[arg(short = 'd', long, default_value = "1")]
    depth: usize,

    /// Context lines around each ref
    #[arg(short = 'C', long, default_value = "2")]
    context: usize,
}

pub fn run(args: FlowArgs) -> Result<()> {
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

    // 1. Find DEFINITION
    let def_pattern = format!(
        r"(?m)^[ \t]*(?:pub\s+)?(?:export\s+)?(?:async\s+)?(?:function|fn|def|const|let|var)\s+{}\b",
        regex::escape(&args.symbol)
    );
    let def_re = RegexBuilder::new(&def_pattern).build()?;

    println!("{}", format!("═══ FLOW: {} ═══", args.symbol).cyan().bold());
    println!();
    println!("{}", "Definition:".yellow().bold());

    let mut def_file: Option<PathBuf> = None;
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        if let Some(m) = def_re.find(&content) {
            let line_num = content[..m.start()].lines().count() + 1;
            println!("  {} {}:{}", "→".green(), f.display().to_string().cyan(), line_num.to_string().yellow());
            let lines: Vec<&str> = content.lines().collect();
            let s = (line_num - 1).saturating_sub(args.context);
            let e = (line_num + args.context).min(lines.len());
            for i in s..e {
                let marker = if i == line_num - 1 { ">".yellow().to_string() } else { " ".to_string() };
                println!("    {} {:>5} │ {}", marker, (i + 1).to_string().dimmed(), lines[i]);
            }
            def_file = Some(f.clone());
            break;
        }
    }
    if def_file.is_none() {
        println!("  {} not found — searching for calls anyway", "⚠".yellow());
    }

    // 2. Find CALLERS
    let call_pattern = format!(r"\b{}\s*\(", regex::escape(&args.symbol));
    let call_re = RegexBuilder::new(&call_pattern).build()?;

    println!("\n{}", "Callers:".yellow().bold());
    let mut caller_files: Vec<(PathBuf, Vec<usize>)> = Vec::new();
    for f in &files {
        if Some(f) == def_file.as_ref() {
            continue; // skip the definition file when listing external callers
        }
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();
        let hits: Vec<usize> = lines.iter().enumerate()
            .filter_map(|(i, l)| {
                let t = l.trim_start();
                if t.starts_with("//") || t.starts_with("*") { return None; }
                // Skip the definition line itself
                if def_re.is_match(l) { return None; }
                if call_re.is_match(l) { Some(i) } else { None }
            })
            .collect();
        if !hits.is_empty() {
            caller_files.push((f.clone(), hits));
        }
    }

    if caller_files.is_empty() {
        println!("  {} no callers found", "(none)".dimmed());
    }

    for (f, hits) in &caller_files {
        println!("\n  {} ({} calls)", f.display().to_string().cyan(), hits.len().to_string().yellow());
        let content = read_file_smart(f)?;
        let lines: Vec<&str> = content.lines().collect();
        for &h in hits.iter().take(5) {
            let s = h.saturating_sub(args.context);
            let e = (h + args.context + 1).min(lines.len());
            for i in s..e {
                let text = lines[i];
                if i == h {
                    let hl = call_re.replace_all(text, |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("    {:>5} │ {}", (i + 1).to_string().green(), hl);
                } else {
                    println!("    {:>5} │ {}", (i + 1).to_string().dimmed(), text.dimmed());
                }
            }
            println!();
        }
        if hits.len() > 5 {
            println!("    {} ({} more calls in this file)", "...".dimmed(), (hits.len() - 5).to_string().dimmed());
        }
    }

    // 3. If depth > 1, recurse on each caller function
    if args.depth > 1 {
        println!("\n{}", "Note: depth > 1 not yet implemented (would trace what callers do next)".dimmed());
    }

    eprintln!("\n{} {} callers across {} files",
        "flow:".bold(),
        caller_files.iter().map(|(_, h)| h.len()).sum::<usize>().to_string().yellow(),
        caller_files.len().to_string().yellow()
    );

    Ok(())
}
