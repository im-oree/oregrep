use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::short_path;
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct TraceArgs {
    /// Function/method name to trace
    name: String,
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Context lines around each call site
    #[arg(short = 'C', long, default_value = "1")]
    context: usize,
    /// Include definition lines
    #[arg(short = 'D', long)]
    include_defs: bool,
}

pub fn run(args: TraceArgs) -> Result<()> {
    let call_re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(&args.name)))?;
    let def_re = regex::Regex::new(&format!(
        r"(?:function\s+{}\b|(?:const|let|var)\s+{}\s*[:=]|class\s+{}\b|fn\s+{}\b|def\s+{}\b)",
        regex::escape(&args.name), regex::escape(&args.name), regex::escape(&args.name),
        regex::escape(&args.name), regex::escape(&args.name)))?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut total = 0usize;
    let mut file_count = 0usize;
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();
        let hits: Vec<usize> = lines.iter().enumerate().filter_map(|(i, l)| {
            if !call_re.is_match(l) { return None; }
            if !args.include_defs && def_re.is_match(l) { return None; }
            Some(i)
        }).collect();
        if hits.is_empty() { continue; }
        total += hits.len();
        file_count += 1;
        println!("\n{}", short_path(&args.path, f).cyan().bold());
        let mut printed = std::collections::HashSet::new();
        for &m in &hits {
            let s = m.saturating_sub(args.context);
            let e = (m + args.context + 1).min(lines.len());
            for i in s..e {
                if printed.contains(&i) { continue; }
                printed.insert(i);
                let lineno = i + 1;
                if i == m {
                    let hl = call_re.replace_all(lines[i], |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("  {}: {}", lineno.to_string().green(), hl);
                } else {
                    println!("  {}| {}", lineno.to_string().dimmed(), lines[i].dimmed());
                }
            }
        }
    }
    eprintln!("\n{} {} call sites in {} files", "Total:".bold(), total.to_string().yellow(), file_count.to_string().yellow());
    Ok(())
}
