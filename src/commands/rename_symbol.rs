use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct RenameSymbolArgs {
    /// Old symbol name
    old: String,

    /// New symbol name
    new: String,

    /// Path to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: RenameSymbolArgs) -> Result<()> {
    if args.old == args.new { anyhow::bail!("Old and new names are the same"); }
    let re = Regex::new(&format!(r"\b{}\b", regex::escape(&args.old)))?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into(), "rs".into(), "py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut edits: Vec<(PathBuf, usize)> = Vec::new(); // (file, occurrence-count)
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let n = re.find_iter(&content).count();
        if n > 0 { edits.push((f.clone(), n)); }
    }

    let total_occurrences: usize = edits.iter().map(|(_, n)| n).sum();
    println!("{} '{}' → '{}'  ({} occurrences in {} files)",
        "Rename:".cyan().bold(),
        args.old.red(),
        args.new.green(),
        total_occurrences.to_string().yellow(),
        edits.len().to_string().yellow());
    for (f, n) in &edits {
        println!("  {} {}  ({}×)", "~".yellow(), f.display().to_string().cyan(), n.to_string().dimmed());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing changed]".yellow().bold());
        return Ok(());
    }

    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Rename in {} files?", edits.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
    for (f, _) in &edits {
        let content = read_file_smart(f)?;
        let new_content = re.replace_all(&content, args.new.as_str()).into_owned();
        if new_content != content {
            if !args.no_backup { let _ = create_backup(f, &label); }
            write_atomic(f, &new_content, content.starts_with('\u{FEFF}'))?;
        }
    }
    println!("\n{} rename applied across {} files", "Done:".green().bold(), edits.len().to_string().green());
    Ok(())
}
