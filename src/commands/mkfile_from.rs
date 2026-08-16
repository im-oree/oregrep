use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::io::Read;
use std::path::PathBuf;

use crate::engine::backup::create_backup;

#[derive(Args)]
pub struct MkfileFromArgs {
    /// Destination file to create or overwrite
    file: PathBuf,

    /// Use clipboard content as the source
    #[arg(long, conflicts_with_all = ["stdin", "from_file"])]
    clipboard: bool,

    /// Read content from stdin
    #[arg(long, conflicts_with_all = ["clipboard", "from_file"])]
    stdin: bool,

    /// Copy content from an existing file
    #[arg(long = "file", value_name = "SOURCE", conflicts_with_all = ["clipboard", "stdin"])]
    from_file: Option<PathBuf>,

    /// Overwrite without prompting if file already exists
    #[arg(long, short = 'f')]
    force: bool,

    /// Skip creating a backup when overwriting
    #[arg(long)]
    no_backup: bool,

    /// Backup label when overwriting (default: timestamp)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Strip UTF-8 BOM from source content
    #[arg(long)]
    strip_bom: bool,
}

pub fn run(args: MkfileFromArgs) -> Result<()> {
    // Exactly one source must be specified
    if !args.clipboard && !args.stdin && args.from_file.is_none() {
        anyhow::bail!(
            "Specify one source: --clipboard, --stdin, or --file <path>"
        );
    }

    // Read content from the chosen source
    let raw: Vec<u8> = if args.clipboard {
        read_clipboard()?
    } else if args.stdin {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .context("Failed to read stdin")?;
        buf
    } else {
        let src = args.from_file.as_ref().unwrap();
        if !src.exists() {
            anyhow::bail!("Source file not found: {}", src.display());
        }
        std::fs::read(src)
            .with_context(|| format!("Failed to read source: {}", src.display()))?
    };

    // Strip BOM if requested
    let content_bytes: &[u8] = if args.strip_bom && raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw
    };

    // Convert to string (lossy UTF-8)
    let content = String::from_utf8_lossy(content_bytes).into_owned();

    // Handle existing file
    if args.file.exists() {
        if !args.force {
            eprint!(
                "{} {} already exists. Overwrite? [y/N] ",
                "Warning:".yellow().bold(),
                args.file.display().to_string().cyan()
            );
            let mut answer = String::new();
            std::io::stdin()
                .read_line(&mut answer)
                .context("Failed to read answer")?;
            if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
                println!("{}", "Aborted.".dimmed());
                return Ok(());
            }
        }

        // Backup existing file before overwrite
        if !args.no_backup {
            let label = args
                .label
                .clone()
                .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
            let backup_path = create_backup(&args.file, &label)
                .with_context(|| format!("Failed to backup {}", args.file.display()))?;
            println!(
                "{} {}",
                "Backup:".dimmed(),
                backup_path.display().to_string().dimmed()
            );
        }
    }

    // Create parent directories if needed
    if let Some(parent) = args.file.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directories: {}", parent.display()))?;
            println!(
                "{} {}",
                "Created dirs:".dimmed(),
                parent.display().to_string().dimmed()
            );
        }
    }

    // Write the file
    std::fs::write(&args.file, content.as_bytes())
        .with_context(|| format!("Failed to write: {}", args.file.display()))?;

    let byte_count = content.len();
    let line_count = content.lines().count();
    let source_label = if args.clipboard {
        "clipboard"
    } else if args.stdin {
        "stdin"
    } else {
        "file"
    };

    println!(
        "{} {} ({} bytes, {} lines, from {})",
        "Created:".green().bold(),
        args.file.display().to_string().cyan(),
        byte_count.to_string().yellow(),
        line_count.to_string().yellow(),
        source_label.dimmed()
    );

    Ok(())
}

fn read_clipboard() -> Result<Vec<u8>> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    let text = clipboard
        .get_text()
        .context("Failed to read clipboard text (clipboard may be empty or contain non-text data)")?;
    Ok(text.into_bytes())
}
