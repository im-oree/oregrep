use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_symbols, SymbolKind};

#[derive(Args)]
pub struct HubArgs {
    /// Directory of files to build a barrel index for
    dir: PathBuf,

    /// Output file (default: <dir>/index.ts or <dir>/mod.rs based on contents)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Only include exported symbols
    #[arg(short = 'E', long, default_value = "true")]
    exported_only: bool,

    /// Star-export (export * from ...) instead of named re-exports
    #[arg(short = 's', long)]
    star: bool,

    /// Overwrite existing hub file
    #[arg(long)]
    force: bool,

    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: HubArgs) -> Result<()> {
    if !args.dir.is_dir() { anyhow::bail!("Not a directory: {}", args.dir.display()); }
    let cfg = crate::engine::walker::WalkConfig {
        root: args.dir.clone(),
        skip_backups: true,
        ..Default::default()
    };
    let mut files = crate::engine::walker::collect_files(&cfg)?;
    files.retain(|p| p.parent() == Some(args.dir.as_path()));

    // Determine language and output name
    let has_ts = files.iter().any(|f| matches!(f.extension().and_then(|e| e.to_str()), Some("ts") | Some("tsx")));
    let has_rs = files.iter().any(|f| f.extension().and_then(|e| e.to_str()) == Some("rs"));
    let has_py = files.iter().any(|f| f.extension().and_then(|e| e.to_str()) == Some("py"));

    let (lang, default_name) = if has_ts { ("ts", "index.ts") }
        else if has_rs { ("rs", "mod.rs") }
        else if has_py { ("py", "__init__.py") }
        else { anyhow::bail!("No supported source files (ts/tsx/rs/py) in {}", args.dir.display()); };

    let output = args.output.clone().unwrap_or_else(|| args.dir.join(default_name));
    if output.exists() && !args.force {
        anyhow::bail!("Hub file exists: {} (use --force)", output.display());
    }

    let mut entries: Vec<(PathBuf, Vec<String>)> = Vec::new();
    for f in &files {
        if f == &output { continue; }
        let ext_ok = match lang {
            "ts" => matches!(f.extension().and_then(|e| e.to_str()), Some("ts") | Some("tsx")),
            "rs" => f.extension().and_then(|e| e.to_str()) == Some("rs"),
            "py" => f.extension().and_then(|e| e.to_str()) == Some("py"),
            _ => false,
        };
        if !ext_ok { continue; }
        let content = read_file_smart(f)?;
        let syms = extract_symbols(&content, f);
        let names: Vec<String> = syms.into_iter()
            .filter(|s| !args.exported_only || s.exported)
            .filter(|s| !matches!(s.kind, SymbolKind::Impl))
            .map(|s| s.name)
            .collect();
        if !names.is_empty() { entries.push((f.clone(), names)); }
    }

    let mut out = String::new();
    match lang {
        "ts" => {
            for (f, names) in &entries {
                let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if args.star {
                    out.push_str(&format!("export * from './{}';\n", stem));
                } else {
                    out.push_str(&format!("export {{ {} }} from './{}';\n", names.join(", "), stem));
                }
            }
        }
        "rs" => {
            for (f, names) in &entries {
                let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("").replace('-', "_");
                out.push_str(&format!("pub mod {};\n", stem));
                if !args.star {
                    for n in names {
                        out.push_str(&format!("pub use {}::{};\n", stem, n));
                    }
                }
            }
        }
        "py" => {
            for (f, names) in &entries {
                let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if args.star {
                    out.push_str(&format!("from .{} import *\n", stem));
                } else {
                    out.push_str(&format!("from .{} import {}\n", stem, names.join(", ")));
                }
            }
        }
        _ => {}
    }

    println!("{} {}", "Hub:".cyan().bold(), output.display().to_string().green());
    println!("  {} {} files, {} entries", "→".dimmed(),
        entries.len().to_string().yellow(),
        entries.iter().map(|(_, n)| n.len()).sum::<usize>().to_string().yellow());

    if args.dry_run {
        println!("\n{}", "[DRY RUN — barrel content below]".yellow().bold());
        print!("{}", out);
        return Ok(());
    }
    std::fs::write(&output, out)?;
    println!("\n{} {}", "Wrote:".green().bold(), output.display().to_string().cyan());
    Ok(())
}
