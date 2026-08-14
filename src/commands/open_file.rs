use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::commands::show::launch_editor;

#[derive(Args)]
pub struct OpenFileArgs {
    /// File or directory to open
    path: PathBuf,

    /// Editor / handler to use (default: OS default)
    #[arg(short = 'e', long)]
    editor: Option<String>,

    /// Open the containing folder in Explorer instead of the file
    #[arg(short = 'F', long)]
    folder: bool,
}

pub fn run(args: OpenFileArgs) -> Result<()> {
    if !args.path.exists() {
        anyhow::bail!("Path not found: {}", args.path.display());
    }

    // Compute target
    let target = if args.folder {
        if args.path.is_dir() {
            args.path.clone()
        } else {
            args.path.parent()
                .map(|p| if p.as_os_str().is_empty() { PathBuf::from(".") } else { p.to_path_buf() })
                .unwrap_or_else(|| PathBuf::from("."))
        }
    } else {
        args.path.clone()
    };

    // Canonicalize for a nicer display, but fall back to raw if that fails
    let display_path = std::fs::canonicalize(&target)
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            // strip Windows \\?\ prefix
            s.strip_prefix(r"\\?\").map(String::from).unwrap_or(s)
        })
        .unwrap_or_else(|_| target.display().to_string());

    if let Some(editor) = &args.editor {
        launch_editor(editor, &target, false)?;
        eprintln!("{} {} {}",
            "Opened:".green().bold(),
            display_path.cyan(),
            format!("(with {})", editor).dimmed());
        return Ok(());
    }

    // OS default handler
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &target.to_string_lossy()])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&target).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(&target).spawn()?;
    }

    eprintln!("{} {}", "Opened:".green().bold(), display_path.cyan());
    Ok(())
}
