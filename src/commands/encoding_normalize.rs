use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;

/// Normalize file encoding + line endings to prevent patch failures from
/// encoding mismatches. Defaults to UTF-8 with LF line endings, no BOM.
#[derive(Args)]
pub struct EncodingNormalizeArgs {
    /// File(s) to normalize
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Target line ending: lf (default), crlf, keep
    #[arg(long, default_value = "lf")]
    to: String,

    /// Add UTF-8 BOM
    #[arg(long)]
    add_bom: bool,

    /// Remove UTF-8 BOM if present (default: on)
    #[arg(long, default_value = "true")]
    strip_bom: bool,

    /// Skip backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: EncodingNormalizeArgs) -> Result<()> {
    let label = args
        .label
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    let mut normalized = 0usize;
    let mut unchanged = 0usize;

    for file in &args.files {
        if !file.exists() {
            eprintln!("{} {}", "SKIP:".yellow(), file.display());
            continue;
        }

        let raw = std::fs::read(file)?;
        let had_bom = raw.starts_with(&[0xEF, 0xBB, 0xBF]);
        let content = read_file_smart(file)?;

        // Normalize newlines
        let lf_only = content.replace("\r\n", "\n").replace('\r', "\n");
        let new_content = match args.to.as_str() {
            "lf" => lf_only,
            "crlf" => lf_only.replace('\n', "\r\n"),
            "keep" => content.clone(),
            _ => anyhow::bail!("Unknown --to value: {}. Use lf, crlf, or keep.", args.to),
        };

        // BOM policy: --add-bom forces a BOM on; otherwise keep the BOM only
        // if it was already there and --strip-bom is off.
        let should_write_bom = args.add_bom || (had_bom && !args.strip_bom);

        // Check if anything actually changes
        let bom_change = had_bom != should_write_bom;
        let content_change = content != new_content;

        if !bom_change && !content_change {
            unchanged += 1;
            println!(
                "{} {} (already normalized)",
                "OK:".dimmed(),
                file.display().to_string().dimmed()
            );
            continue;
        }

        if args.dry_run {
            println!(
                "{} {} (would normalize: {}{}{})",
                "[DRY]".yellow(),
                file.display().to_string().cyan(),
                if content_change { "newlines " } else { "" },
                if bom_change && should_write_bom { "add-bom " } else { "" },
                if bom_change && !should_write_bom { "strip-bom " } else { "" }
            );
            continue;
        }

        // Backup
        if !args.no_backup {
            let bp = create_backup(file, &label)?;
            println!("{} {}", "Backup:".dimmed(), bp.display().to_string().dimmed());
        }

        write_atomic(file, &new_content, should_write_bom)?;
        normalized += 1;

        println!(
            "{} {} → {} {}",
            "Normalized:".green().bold(),
            file.display().to_string().cyan(),
            args.to.to_uppercase().yellow(),
            if should_write_bom { "with BOM".dimmed() } else { "no BOM".dimmed() }
        );
    }

    println!(
        "\n{} {} normalized, {} unchanged",
        "Done:".bold(),
        normalized.to_string().green(),
        unchanged.to_string().dimmed()
    );

    Ok(())
}
