use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_symbols, SymbolKind};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct OrganizeArgs {
    /// Root directory (files at top level get grouped, subdirs are analyzed)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Grouping: type | feature (default type)
    #[arg(short = 'b', long, default_value = "type")]
    by: String,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Actually perform the moves (default: plan only)
    #[arg(long)]
    apply: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: OrganizeArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    // Only look at files sitting directly in args.path (organize top-level items)
    let all = collect_files(&cfg)?;
    let top_level: Vec<PathBuf> = all.into_iter().filter(|p| p.parent() == Some(args.path.as_path())).collect();

    if top_level.is_empty() {
        println!("{}", "No top-level files to organize.".yellow());
        return Ok(());
    }

    // Classify
    let mut moves: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for f in &top_level {
        let bucket = match args.by.as_str() {
            "type" => classify_by_type(f)?,
            "feature" => classify_by_feature(f),
            _ => classify_by_type(f)?,
        };
        moves.entry(bucket).or_default().push(f.clone());
    }

    println!("{} '{}' by {}",
        "Organize plan:".cyan().bold(),
        args.path.display().to_string().yellow(),
        args.by.green());
    let mut keys: Vec<&String> = moves.keys().collect();
    keys.sort();
    for k in keys {
        let files = &moves[k];
        println!("\n  {}/", k.green().bold());
        for f in files {
            let fname = f.file_name().unwrap().to_string_lossy();
            println!("    {} {}", "+".green(), fname.cyan());
        }
    }

    if !args.apply {
        println!("\n{}", "[PLAN — pass --apply to execute]".yellow().bold());
        return Ok(());
    }

    if !args.yes {
        let total: usize = moves.values().map(|v| v.len()).sum();
        let ok = crate::engine::confirm::confirm(&format!("Move {} files into {} folders?", total, moves.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
    for (bucket, files) in &moves {
        let target_dir = args.path.join(bucket);
        std::fs::create_dir_all(&target_dir)?;
        for f in files {
            if !args.no_backup {
                let _ = crate::engine::backup::create_backup(f, &label);
            }
            let target = target_dir.join(f.file_name().unwrap());
            std::fs::rename(f, &target).or_else(|_| {
                std::fs::copy(f, &target)?;
                std::fs::remove_file(f)
            })?;
            println!("  {} {}", "moved:".dimmed(), target.display().to_string().cyan());
        }
    }
    println!("\n{} organized", "Done:".green().bold());
    println!("{}", "Note: import paths may need updating — run `ore move-with-imports` per file for full refactor.".dimmed());
    Ok(())
}

fn classify_by_type(f: &std::path::Path) -> Result<String> {
    let fname = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let content = read_file_smart(f).unwrap_or_default();
    let syms = extract_symbols(&content, f);
    let has_hook = syms.iter().any(|s| matches!(s.kind, SymbolKind::Hook));
    let has_comp = syms.iter().any(|s| matches!(s.kind, SymbolKind::Component));
    let has_class = syms.iter().any(|s| matches!(s.kind, SymbolKind::Class));
    let mostly_types = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Type | SymbolKind::Interface)).count() > syms.len() / 2;

    let bucket = if fname.contains(".test.") || fname.contains(".spec.") { "tests" }
        else if fname.starts_with("use") && has_hook { "hooks" }
        else if has_comp { "components" }
        else if has_class { "classes" }
        else if mostly_types { "types" }
        else if fname.ends_with(".d.ts") { "types" }
        else if fname.contains("util") || fname.contains("helper") { "utils" }
        else if syms.is_empty() { "misc" }
        else { "lib" };
    Ok(bucket.to_string())
}

fn classify_by_feature(f: &std::path::Path) -> String {
    // Naive: use the first token before "-" or "." as feature name
    let fname = f.file_stem().and_then(|n| n.to_str()).unwrap_or("misc");
    let bucket = fname.split(|c: char| c == '-' || c == '_' || c == '.').next().unwrap_or("misc");
    bucket.to_string()
}
