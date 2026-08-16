use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;

use crate::engine::compile::load_report;

#[derive(Args)]
pub struct ErrorsLastArgs {
    /// Show warnings too
    #[arg(short = 'w', long)]
    warnings: bool,

    /// Group by file
    #[arg(short = 'g', long)]
    group: bool,

    /// Show raw output
    #[arg(short = 'r', long)]
    raw: bool,

    /// JSON
    #[arg(short = 'j', long)]
    json: bool,

    /// Only errors in this file (substring)
    #[arg(short = 'f', long)]
    file: Option<String>,
}

pub fn run(args: ErrorsLastArgs) -> Result<()> {
    let report = match load_report()? {
        Some(r) => r,
        None => { println!("{}", "No cached compile report. Run compile-ts / compile-rust first.".yellow()); return Ok(()); }
    };

    if args.json { println!("{}", serde_json::to_string_pretty(&report)?); return Ok(()); }
    if args.raw { println!("{}", report.raw_output); return Ok(()); }

    println!("{} {}  ({} exit {})", "Last compile:".cyan().bold(), report.tool.yellow(),
        report.timestamp.dimmed(), report.exit_code);

    let mut errs = report.errors.clone();
    let mut warns = report.warnings.clone();

    if let Some(f) = &args.file {
        errs.retain(|e| e.file.contains(f));
        warns.retain(|w| w.file.contains(f));
    }

    if args.group {
        let mut by_file: HashMap<String, (Vec<_>, Vec<_>)> = HashMap::new();
        for e in &errs { by_file.entry(e.file.clone()).or_default().0.push(e.clone()); }
        if args.warnings {
            for w in &warns { by_file.entry(w.file.clone()).or_default().1.push(w.clone()); }
        }
        let mut files: Vec<&String> = by_file.keys().collect();
        files.sort();
        for f in files {
            let (es, ws) = &by_file[f];
            println!("\n{} ({} errors, {} warnings)", f.cyan().bold(), es.len().to_string().red(), ws.len().to_string().yellow());
            for e in es {
                println!("  {} L{}:{} {} {}", "err".red(), e.line, e.column, e.code.yellow(), e.message);
            }
            for w in ws {
                println!("  {} L{}:{} {} {}", "warn".yellow(), w.line, w.column, w.code.dimmed(), w.message);
            }
        }
    } else {
        for e in &errs {
            let loc = if e.line > 0 { format!("{}:{}:{}", e.file, e.line, e.column) } else { e.file.clone() };
            println!("  {} {} {} {}", "err".red().bold(), e.code.yellow(), loc.cyan(), e.message);
        }
        if args.warnings {
            for w in &warns {
                let loc = if w.line > 0 { format!("{}:{}:{}", w.file, w.line, w.column) } else { w.file.clone() };
                println!("  {} {} {} {}", "warn".yellow().bold(), w.code.dimmed(), loc.cyan(), w.message);
            }
        }
    }

    println!("\n{} {} errors, {} warnings",
        "Total:".bold(),
        errs.len().to_string().red(),
        warns.len().to_string().yellow());
    Ok(())
}
