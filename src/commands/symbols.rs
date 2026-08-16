use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::symbols::{collect_source_files, extract_symbols};
use crate::engine::walker::{parse_excludes, parse_extensions};

#[derive(Args)]
pub struct SymbolsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Filter by kind (fn, class, hook, comp, const, type, enum, interface, struct, trait, mod)
    #[arg(short = 'k', long)]
    kind: Option<String>,

    /// Only exported symbols (default: everything)
    #[arg(short = 'E', long)]
    exported: bool,

    /// Filter by name substring
    #[arg(short = 'n', long)]
    name: Option<String>,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,

    /// Group by file
    #[arg(short = 'g', long)]
    group: bool,

    /// Just count per kind
    #[arg(short = 'c', long)]
    count: bool,
}

pub fn run(args: SymbolsArgs) -> Result<()> {
    let ext = args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into(), "rs".into(), "py".into()]);
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();
    let files = collect_source_files(&args.path, &ext, &exc)?;

    let mut all: Vec<crate::engine::symbols::Symbol> = Vec::new();
    for (p, c) in &files {
        all.extend(extract_symbols(c, p));
    }
    if args.exported { all.retain(|s| s.exported); }
    if let Some(k) = &args.kind {
        let want = k.to_lowercase();
        all.retain(|s| s.kind.short() == want.as_str());
    }
    if let Some(n) = &args.name {
        let n_lc = n.to_lowercase();
        all.retain(|s| s.name.to_lowercase().contains(&n_lc));
    }

    if args.json { println!("{}", serde_json::to_string_pretty(&all)?); return Ok(()); }
    if args.count {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for s in &all { *counts.entry(s.kind.short()).or_insert(0) += 1; }
        let mut pairs: Vec<(&&str, &usize)> = counts.iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(a.1));
        for (k, v) in pairs {
            println!("  {:<8} {}", k.cyan(), v.to_string().yellow());
        }
        println!("\n{} {}", "Total:".bold(), all.len().to_string().yellow());
        return Ok(());
    }

    if args.group {
        let mut by_file: std::collections::BTreeMap<PathBuf, Vec<&crate::engine::symbols::Symbol>> = std::collections::BTreeMap::new();
        for s in &all { by_file.entry(s.file.clone()).or_default().push(s); }
        for (f, ss) in &by_file {
            println!("\n{}", f.display().to_string().cyan().bold());
            for s in ss {
                let star = if s.exported { "*".green().to_string() } else { " ".to_string() };
                println!("  {} {:<6} L{:<4} {}", star, s.kind.short().magenta(), s.line.to_string().dimmed(), s.name.yellow());
            }
        }
    } else {
        for s in &all {
            let star = if s.exported { "*".green().to_string() } else { " ".to_string() };
            println!("{} {:<6} {}:{}  {}",
                star,
                s.kind.short().magenta(),
                s.file.display().to_string().cyan(),
                s.line.to_string().dimmed(),
                s.name.yellow()
            );
        }
    }
    eprintln!("\n{} {} symbols across {} files", "Total:".bold(), all.len().to_string().yellow(), files.len().to_string().dimmed());
    Ok(())
}
