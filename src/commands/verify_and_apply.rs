use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::parse_patch_file;
use crate::engine::proc::run_cmd_in;

/// Apply a .orepatch atomically, then run a verify command (e.g. compile-ts).
/// If verify fails, all touched files are automatically restored from backup.
///
/// This is the "safe patch" workflow in one command — replaces manual
/// sequence "..." "..." --rollback-on-fail chains.
#[derive(Args)]
pub struct VerifyAndApplyArgs {
    /// Path to .orepatch file (or - for stdin, or use --inline)
    #[arg(default_value = "")]
    source: String,

    /// Inline .orepatch content
    #[arg(long, conflicts_with = "source")]
    inline: Option<String>,

    /// Verify command to run after apply. Defaults to auto-detected via verify-compile.
    #[arg(long = "with", short = 'w')]
    verify_cmd: Option<String>,

    /// Backup label (default: VAA-<timestamp>)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Working directory for verify command
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Literal mode
    #[arg(long)]
    literal: bool,

    /// Skip backup (dangerous — no rollback possible)
    #[arg(long)]
    no_backup: bool,

    /// Skip the verify step (just apply atomically)
    #[arg(long)]
    no_verify: bool,
}

pub fn run(args: VerifyAndApplyArgs) -> Result<()> {
    let cwd = args.dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap());

    // The ore executable running right now — use it for sub-invocations so
    // this works even when `ore` isn't on PATH.
    let ore_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().replace('\\', "\\\\"))
        .unwrap_or_else(|_| "ore".to_string());
    let ore_exe_quoted = format!("\"{}\"", ore_exe);

    // Backup label with a VAA prefix so it's easy to spot in backup lists
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| format!("VAA_{}", chrono::Local::now().format("%Y%m%d_%H%M%S")));

    println!("{}", "── verify-and-apply ────────────────────────────".dimmed());

    // Load patch content to find which files will be touched (for backup+rollback)
    let patch_content = if let Some(inline) = &args.inline {
        crate::engine::patch::unescape_arg(inline)
    } else if args.source == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else if args.source.is_empty() {
        anyhow::bail!("Provide a patch file, --inline, or - for stdin");
    } else {
        let p = PathBuf::from(&args.source);
        if !p.exists() {
            anyhow::bail!("Patch file not found: {}", args.source);
        }
        crate::engine::encoding::read_file_smart(&p)?
    };

    let ops = parse_patch_file(&patch_content)?;
    if ops.is_empty() {
        anyhow::bail!("No patch operations found");
    }

    // Unique files to back up
    let files: BTreeSet<PathBuf> = ops
        .iter()
        .map(|op| PathBuf::from(&op.file))
        .filter(|p| p.exists())
        .collect();

    println!(
        "{} {} operation{} on {} file{}",
        "Plan:".cyan(),
        ops.len().to_string().yellow(),
        if ops.len() == 1 { "" } else { "s" },
        files.len().to_string().yellow(),
        if files.len() == 1 { "" } else { "s" }
    );

    // Phase 1: backup all files
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new();
    if !args.no_backup {
        println!("\n{}", "Phase 1: Backup".cyan().bold());
        for f in &files {
            match create_backup(f, &label) {
                Ok(bp) => {
                    println!("  {} {} → {}", "✓".green(), f.display(), bp.display().to_string().dimmed());
                    backups.push((f.clone(), bp));
                }
                Err(e) => {
                    eprintln!("  {} backup failed for {}: {}", "✗".red(), f.display(), e);
                    anyhow::bail!("Aborting — backup failed. No changes made.");
                }
            }
        }
    }

    // Phase 2: apply atomically via patch-batch
    println!("\n{}", "Phase 2: Apply patches (atomic)".cyan().bold());
    let source_arg = if args.inline.is_some() {
        // Write inline to temp and pass path
        let tmp = std::env::temp_dir().join(format!(".ore-vaa-{}.orepatch", label));
        std::fs::write(&tmp, &patch_content)?;
        tmp.to_string_lossy().into_owned()
    } else {
        args.source.clone()
    };

    let mut patch_cmd = format!(
        "{} patch-batch \"{}\" --atomic --no-backup",
        ore_exe_quoted, source_arg
    );
    if args.literal {
        patch_cmd.push_str(" --literal");
    }

    let patch_result = run_cmd_in(&patch_cmd, Some(&cwd), true, false)?;
    if !patch_result.success() {
        eprintln!("\n{} patch-batch failed. Rolling back...", "✗".red().bold());
        rollback(&backups)?;
        eprintln!("{}", "Aborted before verify. Files restored.".yellow());
        std::process::exit(1);
    }

    // Phase 3: verify
    if !args.no_verify {
        let verify_cmd = args.verify_cmd.clone().unwrap_or_else(|| "verify-compile .".to_string());
        println!("\n{} {}", "Phase 3: Verify".cyan().bold(), format!("({})", verify_cmd).dimmed());

        let full_cmd = if verify_cmd.trim_start().starts_with("ore ") {
            verify_cmd
        } else {
            format!("{} {}", ore_exe_quoted, verify_cmd)
        };

        let verify_result = run_cmd_in(&full_cmd, Some(&cwd), true, false)?;
        if !verify_result.success() {
            eprintln!("\n{} verify failed (exit {}). Rolling back...",
                "✗".red().bold(),
                verify_result.exit_code
            );
            rollback(&backups)?;
            eprintln!("{}", "Files restored to pre-patch state.".yellow());
            std::process::exit(verify_result.exit_code);
        }
    } else {
        println!("\n{}", "Phase 3: Verify skipped (--no-verify)".dimmed());
    }

    println!("\n{}", "── SUCCESS ─────────────────────────────────────".green().bold());
    println!("{} {} file{} patched and verified", "✓".green().bold(),
        files.len().to_string().yellow(),
        if files.len() == 1 { "" } else { "s" }
    );
    if !args.no_backup {
        println!("{} label: {}", "Backups kept —".dimmed(), label.cyan());
    }

    Ok(())
}

fn rollback(backups: &[(PathBuf, PathBuf)]) -> Result<()> {
    for (target, backup) in backups {
        match std::fs::copy(backup, target) {
            Ok(_) => println!("  {} restored {}", "↻".yellow(), target.display()),
            Err(e) => eprintln!("  {} failed to restore {}: {}", "⚠".red(), target.display(), e),
        }
    }
    Ok(())
}
