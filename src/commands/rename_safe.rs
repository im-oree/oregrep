use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::backup::{create_backup, restore_backup};
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::proc::run_cmd;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct RenameSafeArgs {
    old: String,
    new: String,
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Verify command to run after rename (default: auto-detect tsc/cargo)
    #[arg(short = 'v', long)]
    verify: Option<String>,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: RenameSafeArgs) -> Result<()> {
    if args.old == args.new { anyhow::bail!("Old and new names are the same"); }
    let re = Regex::new(&format!(r"\b{}\b", regex::escape(&args.old)))?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut edits: Vec<(PathBuf, usize)> = Vec::new();
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let n = re.find_iter(&content).count();
        if n > 0 { edits.push((f.clone(), n)); }
    }

    let total: usize = edits.iter().map(|(_, n)| n).sum();
    println!("{} '{}' → '{}' ({} occurrences, {} files)",
        "Rename-safe:".cyan().bold(),
        args.old.red(),
        args.new.green(),
        total.to_string().yellow(),
        edits.len().to_string().yellow());

    if args.dry_run {
        for (f, n) in &edits {
            println!("  {} {}  ({}×)", "~".yellow(), f.display().to_string().cyan(), n.to_string().dimmed());
        }
        println!("\n{}", "[DRY RUN — nothing changed]".yellow().bold());
        return Ok(());
    }

    // Determine verify command
    let verify_cmd = args.verify.clone().unwrap_or_else(|| {
        if args.path.join("Cargo.toml").exists() { "cargo check --quiet".to_string() }
        else if args.path.join("tsconfig.json").exists() { "npx tsc --noEmit".to_string() }
        else if args.path.join("package.json").exists() { "npx tsc --noEmit".to_string() }
        else { "echo no verify configured".to_string() }
    });
    println!("  {} verify: {}", "→".dimmed(), verify_cmd.magenta());

    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Rename in {} files, verify with `{}`? (auto-rollback on fail)", edits.len(), verify_cmd), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    // Unique per invocation (microseconds + PID): second-precision labels get
    // overwritten by back-to-back runs and rollback would restore a stale snapshot.
    let label = format!("RENAME_{}_{}", chrono::Local::now().format("%Y%m%d_%H%M%S%f"), std::process::id());
    // Apply
    for (f, _) in &edits {
        let content = read_file_smart(f)?;
        let new_content = re.replace_all(&content, args.new.as_str()).into_owned();
        if new_content != content {
            let _ = create_backup(f, &label);
            write_atomic(f, &new_content, content.starts_with('\u{FEFF}'))?;
        }
    }
    println!("  {} rename applied", "OK".green());

    // Verify
    println!("  {} running verify...", "▶".cyan());
    let r = run_cmd(&verify_cmd, false, false)?;
    if r.success() {
        println!("  {} verify passed", "OK".green().bold());
        Ok(())
    } else {
        println!("  {} verify FAILED (exit {})", "FAIL".red().bold(), r.exit_code);
        if !r.stdout.is_empty() { println!("{}", r.stdout); }
        if !r.stderr.is_empty() { eprintln!("{}", r.stderr); }
        println!("\n  {} rolling back...", "↺".yellow());
        for (f, _) in &edits {
            let _ = restore_backup(f, &label);
        }
        println!("  {} rolled back {} files", "OK".green(), edits.len().to_string().yellow());
        anyhow::bail!("Rename reverted due to verify failure.");
    }
}
