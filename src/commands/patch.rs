use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::io::Read;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{apply_patch, parse_patch_file, read_for_patch, write_atomic, PatchMode};

#[derive(Args)]
pub struct PatchArgs {
    /// File to patch (required unless --patch-file or --stdin)
    file: Option<PathBuf>,

    /// Text to find
    #[arg(short = 'f', long, conflicts_with_all = ["patch_file", "stdin"])]
    find: Option<String>,

    /// Text to replace with
    #[arg(short = 'r', long, conflicts_with_all = ["patch_file", "stdin"])]
    replace: Option<String>,

    /// Load patches from a .orepatch file
    #[arg(long, conflicts_with_all = ["find", "replace", "stdin"])]
    patch_file: Option<PathBuf>,

    /// Read patch spec from stdin
    #[arg(long, conflicts_with_all = ["find", "replace", "patch_file"])]
    stdin: bool,

    /// Replace all occurrences (default: fail if not exactly 1 match)
    #[arg(short = 'a', long)]
    all: bool,

    /// Replace only the Nth occurrence (1-indexed)
    #[arg(short = 'n', long)]
    nth: Option<usize>,

    /// Replace only the first occurrence
    #[arg(long)]
    first: bool,

    /// Replace only the last occurrence
    #[arg(long)]
    last: bool,

    /// Skip creating a backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label (default: timestamp)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run: show what would change, don't write
    #[arg(long)]
    dry_run: bool,

    /// Literal mode: do not unescape \n \t \\ in find/replace (use for paths/verbatim strings)
    #[arg(long)]
    literal: bool,

    /// Context before: only match if this text appears before --find
    #[arg(long)]
    context_before: Option<String>,

    /// Context after: only match if this text appears after --find
    #[arg(long)]
    context_after: Option<String>,

    /// Idempotent: skip if replacement text already exists in file. No error.
    #[arg(long)]
    if_not_exists: bool,
}

pub fn run(args: PatchArgs) -> Result<()> {
    // Determine mode
    let mode = if args.all {
        PatchMode::All
    } else if let Some(n) = args.nth {
        PatchMode::Nth(n)
    } else if args.first {
        PatchMode::First
    } else if args.last {
        PatchMode::Last
    } else {
        PatchMode::Once
    };

    // Route: patch-file, stdin, or single
    if let Some(ref patch_file) = args.patch_file {
        let content = crate::engine::encoding::read_file_smart(&patch_file)
            .with_context(|| format!("Failed to read patch file: {}", patch_file.display()))?;
        return apply_batch(&content, &args, mode);
    }

    if args.stdin {
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content)?;
        return apply_batch(&content, &args, mode);
    }

    // Single-file mode
    let file = args
        .file
        .clone()
        .ok_or_else(|| anyhow::anyhow!("File argument required (or use --patch-file / --stdin)"))?;
    let find = args
        .find
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--find required"))?;
    let replace = args.replace.clone().unwrap_or_default();

    apply_single(&file, &find, &replace, mode, &args)
}

fn apply_single(
    file: &std::path::Path,
    find: &str,
    replace: &str,
    mode: PatchMode,
    args: &PatchArgs,
) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    let (content, had_bom, newline) = read_for_patch(file)?;

    // Unescape \n \t \\ unless --literal was passed (for paths/verbatim strings).
    // Then normalize all newlines to the file's actual line ending style.
    let find_unesc = if args.literal {
        find.to_string()
    } else {
        crate::engine::patch::unescape_arg(find)
    };
    let replace_unesc = if args.literal {
        replace.to_string()
    } else {
        crate::engine::patch::unescape_arg(replace)
    };
    let find_norm = find_unesc.replace("\r\n", "\n").replace('\n', newline);
    let replace_norm = replace_unesc.replace("\r\n", "\n").replace('\n', newline);

    // Idempotent check: if replace text already present, no-op
    if args.if_not_exists && !replace_norm.is_empty() && content.contains(&replace_norm) {
        println!(
            "{} {} — replacement text already present, skipping",
            "Skipped:".dimmed(),
            file.display().to_string().dimmed()
        );
        return Ok(());
    }

    // Context disambiguation: if --context-before / --context-after given,
    // narrow down which occurrence to patch by finding one bracketed by both.
    let (new_content, result) = if args.context_before.is_some() || args.context_after.is_some() {
        let ctx_before = args.context_before.as_deref().map(|s| {
            (if args.literal { s.to_string() } else { crate::engine::patch::unescape_arg(s) })
                .replace("\r\n", "\n").replace('\n', newline)
        });
        let ctx_after = args.context_after.as_deref().map(|s| {
            (if args.literal { s.to_string() } else { crate::engine::patch::unescape_arg(s) })
                .replace("\r\n", "\n").replace('\n', newline)
        });
        apply_with_context(&content, &find_norm, &replace_norm, ctx_before.as_deref(), ctx_after.as_deref())?
    } else {
        apply_patch(&content, &find_norm, &replace_norm, mode)?
    };

    if args.dry_run {
        println!("{} {}",
            "[DRY RUN]".yellow().bold(),
            file.display().to_string().cyan()
        );
        println!("  {} matches found, {} would be replaced",
            result.matches_found.to_string().yellow(),
            result.replacements_made.to_string().green()
        );
        return Ok(());
    }

    // Backup
    if !args.no_backup {
        let label = args
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let backup_path = create_backup(file, &label)?;
        println!("{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    // Atomic write
    write_atomic(file, &new_content, had_bom)?;

    println!("{} {} ({} replacement{})",
        "Patched:".green().bold(),
        file.display().to_string().cyan(),
        result.replacements_made.to_string().yellow(),
        if result.replacements_made == 1 { "" } else { "s" }
    );

    Ok(())
}

fn apply_batch(patch_content: &str, args: &PatchArgs, mode: PatchMode) -> Result<()> {
    let ops = parse_patch_file(patch_content)?;
    println!("{} {} patch operations", "Loaded:".cyan(), ops.len().to_string().yellow());

    let mut succeeded = 0;
    let mut failed = 0;

    for (i, op) in ops.iter().enumerate() {
        let file = PathBuf::from(&op.file);
        println!("\n{} {} {}",
            format!("[{}/{}]", i + 1, ops.len()).dimmed(),
            "Applying to:".cyan(),
            file.display().to_string().yellow()
        );
        match apply_single(&file, &op.find, &op.replace, mode, args) {
            Ok(_) => succeeded += 1,
            Err(e) => {
                eprintln!("  {} {}", "FAILED:".red().bold(), e);
                failed += 1;
            }
        }
    }

    println!("\n{}: {} succeeded, {} failed",
        "Batch complete".bold(),
        succeeded.to_string().green(),
        failed.to_string().red()
    );

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn apply_with_context(
    content: &str,
    find: &str,
    replace: &str,
    ctx_before: Option<&str>,
    ctx_after: Option<&str>,
) -> Result<(String, crate::engine::patch::PatchResult)> {
    let matches: Vec<usize> = content.match_indices(find).map(|(i, _)| i).collect();
    if matches.is_empty() {
        anyhow::bail!("Find pattern not found in content");
    }

    // Filter matches by context
    let mut valid: Vec<usize> = Vec::new();
    for &m in &matches {
        let before_ok = match ctx_before {
            Some(cb) => {
                let scan = &content[..m];
                scan.rfind(cb).is_some()
            }
            None => true,
        };
        let after_ok = match ctx_after {
            Some(ca) => {
                let scan = &content[m + find.len()..];
                scan.contains(ca)
            }
            None => true,
        };
        if before_ok && after_ok {
            valid.push(m);
        }
    }

    if valid.is_empty() {
        anyhow::bail!("Find pattern found {} times but none matched context constraints", matches.len());
    }
    if valid.len() > 1 {
        anyhow::bail!(
            "Find pattern with context still matches {} times — narrow context further",
            valid.len()
        );
    }

    let target = valid[0];
    let mut new_content = String::with_capacity(content.len());
    new_content.push_str(&content[..target]);
    new_content.push_str(replace);
    new_content.push_str(&content[target + find.len()..]);

    Ok((
        new_content,
        crate::engine::patch::PatchResult {
            matches_found: matches.len(),
            replacements_made: 1,
        },
    ))
}
