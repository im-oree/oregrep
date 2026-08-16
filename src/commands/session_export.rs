use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::history::list_recent;
use crate::engine::index::open_index;

#[derive(Args)]
pub struct SessionExportArgs {
    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Working directory (default: current dir)
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Include last N history entries (default: 50)
    #[arg(long, default_value = "50")]
    limit: usize,

    /// Include git status summary
    #[arg(long, default_value = "true")]
    git: bool,

    /// Include notes
    #[arg(long, default_value = "true")]
    notes: bool,
}

pub fn run(args: SessionExportArgs) -> Result<()> {
    let cwd = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut doc = String::new();

    doc.push_str("# Session Export\n\n");
    doc.push_str(&format!(
        "**Generated:** {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    doc.push_str(&format!("**Directory:** {}\n\n", cwd.display()));

    // ── Git status ──
    if args.git {
        doc.push_str("## Git Status\n\n");
        match std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&cwd)
            .output()
        {
            Ok(output) if output.status.success() => {
                let status = String::from_utf8_lossy(&output.stdout);
                if status.trim().is_empty() {
                    doc.push_str("Clean working tree.\n\n");
                } else {
                    doc.push_str("```\n");
                    doc.push_str(&status);
                    doc.push_str("```\n\n");
                }
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                doc.push_str(&format!("Git error: {}\n\n", err.trim()));
            }
            Err(_) => {
                doc.push_str("Git not available.\n\n");
            }
        }

        // Branch
        match std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()
        {
            Ok(output) if output.status.success() => {
                let branch = String::from_utf8_lossy(&output.stdout);
                doc.push_str(&format!("**Branch:** `{}`\n\n", branch.trim()));
            }
            _ => {}
        }

        // Recent commits
        match std::process::Command::new("git")
            .args(["log", "--oneline", "-5"])
            .current_dir(&cwd)
            .output()
        {
            Ok(output) if output.status.success() => {
                let log = String::from_utf8_lossy(&output.stdout);
                if !log.trim().is_empty() {
                    doc.push_str("### Recent Commits\n\n```\n");
                    doc.push_str(&log);
                    doc.push_str("```\n\n");
                }
            }
            _ => {}
        }
    }

    // ── Notes ──
    if args.notes {
        let notes_path = cwd.join(".ore").join("notes.json");
        if notes_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&notes_path) {
                if let Ok(notes) =
                    serde_json::from_str::<std::collections::BTreeMap<String, String>>(&content)
                {
                    if !notes.is_empty() {
                        doc.push_str("## Project Notes\n\n");
                        for (key, value) in &notes {
                            doc.push_str(&format!("- **{}**: {}\n", key, value));
                        }
                        doc.push_str("\n");
                    }
                }
            }
        }
    }

    // ── History ──
    doc.push_str("## Operation History\n\n");
    match open_index(&cwd).and_then(|conn| list_recent(&conn, args.limit as i64, None, true)) {
        Ok(entries) => {
            if entries.is_empty() {
                doc.push_str("No operations recorded.\n\n");
            } else {
                doc.push_str("| Time | Operation | File | Details |\n");
                doc.push_str("|------|-----------|------|---------|\n");
                for entry in &entries {
                    let ts = chrono::DateTime::from_timestamp(entry.timestamp, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| entry.timestamp.to_string());
                    doc.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        ts,
                        entry.operation,
                        entry.file.as_deref().unwrap_or(""),
                        entry
                            .details
                            .as_deref()
                            .map(|d| d.replace('|', "\\|"))
                            .unwrap_or_default()
                    ));
                }
                doc.push_str("\n");
            }
        }
        Err(e) => {
            doc.push_str(&format!("History unavailable: {}\n\n", e));
        }
    }

    // ── Files modified (from git) ──
    doc.push_str("## Files Modified (git diff)\n\n");
    match std::process::Command::new("git")
        .args(["diff", "--name-status", "HEAD"])
        .current_dir(&cwd)
        .output()
    {
        Ok(output) if output.status.success() => {
            let diff = String::from_utf8_lossy(&output.stdout);
            if diff.trim().is_empty() {
                doc.push_str("No uncommitted changes.\n\n");
            } else {
                doc.push_str("```\n");
                doc.push_str(&diff);
                doc.push_str("```\n\n");
            }
        }
        _ => {
            doc.push_str("Git diff unavailable.\n\n");
        }
    }

    // ── Output ──
    if let Some(ref out_path) = args.output {
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(out_path, &doc)?;
        println!(
            "{} {} ({} bytes)",
            "Exported:".green().bold(),
            out_path.display().to_string().cyan(),
            doc.len().to_string().yellow()
        );
    } else {
        print!("{}", doc);
    }

    Ok(())
}
