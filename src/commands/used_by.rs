use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::symbols::{collect_source_files, extract_imports, resolve_ts_import};
use crate::engine::walker::{parse_excludes, parse_extensions};

#[derive(Args)]
pub struct UsedByArgs {
    file: PathBuf,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Show which named import(s) each importer uses
    #[arg(short = 'n', long)]
    names: bool,
}

pub fn run(args: UsedByArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let target_abs = std::fs::canonicalize(&args.file)?;
    let target_clean = strip_ext_prefix(&target_abs);

    let ext = args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into()]);
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();
    let files = collect_source_files(&args.path, &ext, &exc)?;

    let mut importers: Vec<(PathBuf, Vec<String>)> = Vec::new();
    for (p, c) in &files {
        if std::fs::canonicalize(p).map(|x| strip_ext_prefix(&x)) .unwrap_or_default() == target_clean { continue; }
        let imps = extract_imports(c, p);
        for imp in &imps {
            if let Some(resolved) = resolve_ts_import(p, &imp.source) {
                if let Ok(resolved_abs) = std::fs::canonicalize(&resolved) {
                    if strip_ext_prefix(&resolved_abs) == target_clean {
                        let names = if imp.named.is_empty() { vec!["(*)".to_string()] } else { imp.named.clone() };
                        importers.push((p.clone(), names));
                        break;
                    }
                }
            }
        }
    }

    println!("{} {}", "Used by:".cyan().bold(), args.file.display().to_string().yellow());
    if importers.is_empty() {
        println!("  {}", "(no importers found)".dimmed());
        return Ok(());
    }
    for (f, names) in &importers {
        if args.names {
            println!("  {} {} {}", f.display().to_string().cyan(), "→".dimmed(), names.join(", ").magenta());
        } else {
            println!("  {}", f.display().to_string().cyan());
        }
    }
    eprintln!("\n{} {} importers", "Total:".bold(), importers.len().to_string().yellow());
    Ok(())
}

fn strip_ext_prefix(p: &std::path::Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { p.to_path_buf() }
}
