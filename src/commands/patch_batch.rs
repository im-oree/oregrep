use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{
    apply_patch, parse_patch_file, read_for_patch, unescape_arg, write_atomic, PatchMode, PatchOp,
};

#[derive(Args)]
pub struct PatchBatchArgs {
    /// Path to .orepatch file, or - to read from stdin. Omit if using --inline.
    #[arg(default_value = "")]
    source: String,

    /// Inline patch content as a single string. Use the same .orepatch format.
    /// Separate ops with === on its own line. Separate sections with --- on its own line.
    #[arg(long, conflicts_with = "source")]
    inline: Option<String>,

    /// All-or-nothing: if any find fails, no files are written
    #[arg(long)]
    atomic: bool,

    /// Pre-flight report: show which hunks apply/fail without writing
    #[arg(long)]
    report: bool,

    /// Pre-flight verification: check all ops apply without writing
    /// (exit 0 = all apply, 1 = any fail)
    #[arg(long)]
    verify: bool,

    /// If any write fails, restore every file modified by this run from its backup
    #[arg(long)]
    rollback_on_fail: bool,

    /// Patch mode for all operations: once, all, first, last
    #[arg(long, default_value = "once")]
    mode: String,

    /// Skip backups for all operations
    #[arg(long)]
    no_backup: bool,

    /// Backup label for all operations (default: timestamp)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Stop on first failure (default: attempt all, report failures)
    #[arg(long, conflicts_with = "continue_on_fail")]
    stop_on_fail: bool,

    /// Explicitly continue past failures (this is the default; useful for
    /// clarity when combined with --report or in scripts)
    #[arg(long, conflicts_with = "stop_on_fail")]
    continue_on_fail: bool,

    /// Literal mode: do not unescape \n \t \\ in find/replace strings from the patch file
    #[arg(long)]
    literal: bool,

    /// Validate the patch content and exit — no dry-pass, no writes
    #[arg(long)]
    validate: bool,

    /// Idempotent: skip ops where replacement is already present in the file
    #[arg(long)]
    idempotent: bool,
}

struct OpResult {
    op_index: usize,
    file: String,
    success: bool,
    error: Option<String>,
    new_content: Option<String>,
    had_bom: bool,
}

pub fn run(args: PatchBatchArgs) -> Result<()> {
    // Read patch source: --inline arg, stdin (-), or file path
    let patch_content = if let Some(inline) = args.inline.clone() {
        // Unescape \n \t \\ so the single-arg format is usable from the CLI/GUI.
        // If the caller is passing literal escape sequences in a shell arg they need
        // to become real characters for the parser.
        crate::engine::patch::unescape_arg(&inline)
    } else if args.source == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else if args.source.is_empty() {
        anyhow::bail!("Provide a patch file path, use --inline \"...\", or pipe via '-'");
    } else {
        let p = PathBuf::from(&args.source);
        if !p.exists() {
            anyhow::bail!("Patch file not found: {}", args.source);
        }
        crate::engine::encoding::read_file_smart(&p)?
    };

    let ops = parse_patch_file(&patch_content)?;
    if ops.is_empty() {
        anyhow::bail!("No patch operations found in {}", args.source);
    }

    // --validate: report structure and exit
    if args.validate {
        let v = crate::engine::patch::validate_patch_content(&patch_content)?;
        println!(
            "{} {} operation{} on {} unique file{}",
            "Validation OK:".green().bold(),
            v.op_count.to_string().yellow(),
            if v.op_count == 1 { "" } else { "s" },
            v.file_count.to_string().yellow(),
            if v.file_count == 1 { "" } else { "s" }
        );
        for f in &v.files {
            println!("  {}", f.cyan());
        }
        return Ok(());
    }

    let mode = parse_mode(&args.mode)?;
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    let source_label = if args.inline.is_some() {
        "--inline".to_string()
    } else if args.source == "-" {
        "stdin".to_string()
    } else {
        args.source.clone()
    };
    println!(
        "{} {} operation{} from {}",
        "Loaded:".cyan(),
        ops.len().to_string().yellow(),
        if ops.len() == 1 { "" } else { "s" },
        source_label.dimmed()
    );

    if args.report {
        return run_report(&ops, mode, args.literal, args.idempotent);
    }

    if args.verify {
        return run_verify(&ops, mode, args.literal, args.atomic, args.idempotent);
    }

    // Dry-pass: attempt all patches in memory
    let results = dry_pass(&ops, mode, args.literal, args.idempotent);

    let failed: Vec<&OpResult> = results.iter().filter(|r| !r.success).collect();
    let _succeeded: Vec<&OpResult> = results.iter().filter(|r| r.success).collect();

    // Print per-op status
    for r in &results {
        let prefix = format!("[{}/{}]", r.op_index + 1, ops.len());
        if r.success {
            println!(
                "{} {} {} {}",
                prefix.dimmed(),
                "✓".green(),
                r.file.cyan(),
                "ready".dimmed()
            );
        } else {
            eprintln!(
                "{} {} {} — {}",
                prefix.dimmed(),
                "✗".red().bold(),
                r.file.cyan(),
                r.error.as_deref().unwrap_or("unknown error").red()
            );
        }
    }

    println!();

    if args.atomic && !failed.is_empty() {
        eprintln!(
            "{} {}/{} operations would fail. {} mode — nothing written.",
            "ABORT:".red().bold(),
            failed.len().to_string().red(),
            ops.len().to_string().yellow(),
            "--atomic".yellow()
        );
        std::process::exit(1);
    }

    if !failed.is_empty() && args.stop_on_fail {
        eprintln!(
            "{} first failure at op {}. Use --atomic for all-or-nothing.",
            "STOPPED:".red().bold(),
            failed[0].op_index + 1
        );
        std::process::exit(1);
    }

    // Write phase: only write successful ops
    let mut written = 0usize;
    let mut write_failed = 0usize;
    // Files actually written this run — rollback restores only these
    let mut written_files: HashSet<PathBuf> = HashSet::new();
    // Back up each file at most once per run so a same-file multi-op patch
    // doesn't let a later backup overwrite the earlier one with an
    // intermediate state (backup labels are second-resolution timestamps).
    let mut backed_up: HashSet<PathBuf> = HashSet::new();
    // file -> backup path, used by --rollback-on-fail to restore originals
    let mut backup_map: HashMap<PathBuf, PathBuf> = HashMap::new();

    for r in results.iter().filter(|r| r.success) {
        let file_path = PathBuf::from(&r.file);
        let new_content = r.new_content.as_ref().unwrap();

        // Idempotent skip: if the "new" content equals the current disk content, no write
        if args.idempotent {
            if let Ok(current) = crate::engine::encoding::read_file_smart(&file_path) {
                if &current == new_content {
                    println!(
                        "{} {} (already applied)",
                        "Skipped:".dimmed(),
                        r.file.dimmed()
                    );
                    continue;
                }
            }
        }

        // Backup (once per file)
        if !args.no_backup && !backed_up.contains(&file_path) {
            backed_up.insert(file_path.clone());
            match create_backup(&file_path, &label) {
                Ok(bp) => {
                    backup_map.insert(file_path.clone(), bp.clone());
                    println!(
                        "{} {}",
                        "Backup:".dimmed(),
                        bp.display().to_string().dimmed()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{} backup failed for {}: {}",
                        "⚠".yellow(),
                        r.file,
                        e
                    );
                }
            }
        }

        match write_atomic(&file_path, new_content, r.had_bom) {
            Ok(_) => {
                written_files.insert(file_path.clone());
                println!(
                    "{} {}",
                    "Patched:".green().bold(),
                    r.file.cyan()
                );
                written += 1;
            }
            Err(e) => {
                eprintln!(
                    "{} write failed for {}: {}",
                    "✗".red().bold(),
                    r.file,
                    e
                );
                write_failed += 1;
            }
        }
    }

    if write_failed > 0 && args.rollback_on_fail {
        eprintln!();
        if args.no_backup {
            eprintln!(
                "{} --no-backup set — cannot roll back. Files may be partially modified.",
                "WARN:".yellow().bold()
            );
        } else {
            eprintln!(
                "{} write failure detected — restoring {} file(s) from backups...",
                "ROLLBACK:".red().bold(),
                written_files.len().to_string().yellow()
            );
            let mut restored = 0usize;
            let mut restore_failed = 0usize;
            for (file_path, bp) in &backup_map {
                if !written_files.contains(file_path) {
                    continue; // write never succeeded — nothing to restore
                }
                // Clear the read-only attribute (Windows) so the restore can overwrite
                if let Ok(meta) = std::fs::metadata(file_path) {
                    let mut perms = meta.permissions();
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(file_path, perms);
                }
                match std::fs::copy(bp, file_path) {
                    Ok(_) => {
                        restored += 1;
                        eprintln!("  ↺ {}", file_path.display().to_string().dimmed());
                    }
                    Err(e) => {
                        restore_failed += 1;
                        eprintln!(
                            "  ✗ restore failed for {}: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
            eprintln!(
                "{} {} restored, {} failed",
                "Rollback done:".bold(),
                restored.to_string().green(),
                restore_failed.to_string().red()
            );
        }
        std::process::exit(1);
    }

    println!(
        "\n{} {}/{} written, {} failed to find, {} failed to write",
        "Done:".bold(),
        written.to_string().green(),
        ops.len().to_string().yellow(),
        failed.len().to_string().red(),
        write_failed.to_string().red()
    );

    if !failed.is_empty() || write_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_verify(ops: &[PatchOp], mode: PatchMode, literal: bool, atomic: bool, idempotent: bool) -> Result<()> {
    let results = dry_pass(ops, mode, literal, idempotent);
    let failed: Vec<&OpResult> = results.iter().filter(|r| !r.success).collect();

    for r in &results {
        let prefix = format!("[{}/{}]", r.op_index + 1, ops.len());
        if r.success {
            println!(
                "{} {} {} {}",
                prefix.dimmed(),
                "✓".green(),
                r.file.cyan(),
                "ready".dimmed()
            );
        } else {
            eprintln!(
                "{} {} {} — {}",
                prefix.dimmed(),
                "✗".red().bold(),
                r.file.cyan(),
                r.error.as_deref().unwrap_or("unknown error").red()
            );
        }
    }

    println!();
    if !failed.is_empty() {
        eprintln!(
            "{} {}/{} operations would fail — nothing written (--verify).",
            "VERIFY FAIL:".red().bold(),
            failed.len().to_string().red(),
            ops.len().to_string().yellow()
        );
        if atomic {
            eprintln!("{} --atomic set — aborting.", "Note:".yellow());
        }
        std::process::exit(1);
    }
    println!(
        "{} {}/{} operations OK — nothing written (--verify).",
        "VERIFY PASS:".green().bold(),
        ops.len().to_string().green(),
        ops.len().to_string().yellow()
    );
    Ok(())
}

fn run_report(ops: &[PatchOp], mode: PatchMode, literal: bool, idempotent: bool) -> Result<()> {
    println!("{}", "Pre-flight report (no files written):".yellow().bold());
    println!();

    let results = dry_pass(ops, mode, literal, idempotent);
    let mut pass = 0usize;
    let mut fail = 0usize;

    for r in &results {
        if r.success {
            pass += 1;
            println!(
                "  {} [{}] {}",
                "PASS".green().bold(),
                (r.op_index + 1).to_string().dimmed(),
                r.file.cyan()
            );
        } else {
            fail += 1;
            eprintln!(
                "  {} [{}] {} — {}",
                "FAIL".red().bold(),
                (r.op_index + 1).to_string().dimmed(),
                r.file.cyan(),
                r.error.as_deref().unwrap_or("unknown").red()
            );
        }
    }

    println!();
    println!(
        "{} {}/{} would succeed, {} would fail",
        "Report:".bold(),
        pass.to_string().green(),
        ops.len().to_string().yellow(),
        fail.to_string().red()
    );

    if fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn dry_pass(ops: &[PatchOp], mode: PatchMode, literal: bool, idempotent: bool) -> Vec<OpResult> {
    let mut results = Vec::new();
    // Chained working state: for consecutive ops on the same file, apply each op
    // on top of the previous op's result instead of re-reading from disk.
    // This makes same-file multi-op patches compose sequentially (op2 sees op1's
    // edits), instead of all ops being computed against the original file and
    // the later write clobbering the earlier one.
    let mut working: HashMap<PathBuf, (String, bool)> = HashMap::new();

    for (i, op) in ops.iter().enumerate() {
        let file_path = PathBuf::from(&op.file);

        let result = (|| -> Result<(String, bool)> {
            let (content, had_bom, newline) = if let Some((c, bom)) = working.get(&file_path) {
                // Chain: base this op on the previous op's output for the same file
                let nl = if c.contains("\r\n") { "\r\n" } else { "\n" };
                (c.clone(), *bom, nl)
            } else {
                if !file_path.exists() {
                    anyhow::bail!("File not found: {}", op.file);
                }
                read_for_patch(&file_path)?
            };

            let find_unesc = if literal {
                op.find.clone()
            } else {
                unescape_arg(&op.find)
            };
            let replace_unesc = if literal {
                op.replace.clone()
            } else {
                unescape_arg(&op.replace)
            };
            let find_norm = find_unesc.replace("\r\n", "\n").replace('\n', newline);
            let replace_norm = replace_unesc.replace("\r\n", "\n").replace('\n', newline);

            // Idempotent: if replace already in content AND find not present, no-op
            if idempotent && !replace_norm.is_empty() && content.contains(&replace_norm) && !content.contains(&find_norm) {
                // Return unchanged content — write phase will detect no change
                return Ok((content.clone(), had_bom));
            }

            let (new_content, _) = apply_patch(&content, &find_norm, &replace_norm, mode)?;
            // Chain: remember this op's result for subsequent ops on the same file
            working.insert(file_path.clone(), (new_content.clone(), had_bom));
            Ok((new_content, had_bom))
        })();

        match result {
            Ok((new_content, had_bom)) => results.push(OpResult {
                op_index: i,
                file: op.file.clone(),
                success: true,
                error: None,
                new_content: Some(new_content),
                had_bom,
            }),
            Err(e) => results.push(OpResult {
                op_index: i,
                file: op.file.clone(),
                success: false,
                error: Some(e.to_string()),
                new_content: None,
                had_bom: false,
            }),
        }
    }

    results
}

fn parse_mode(s: &str) -> Result<PatchMode> {
    match s {
        "once" => Ok(PatchMode::Once),
        "all" => Ok(PatchMode::All),
        "first" => Ok(PatchMode::First),
        "last" => Ok(PatchMode::Last),
        _ => anyhow::bail!("Unknown mode {:?}. Use: once, all, first, last", s),
    }
}
