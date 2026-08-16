use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::path::PathBuf;

/// Persistent multi-patch buffer — build up a set of edits, review, apply.
#[derive(Args)]
pub struct PatchSessionArgs {
    #[command(subcommand)]
    action: SessionAction,

    #[arg(long)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Start a new named session
    Start { name: String },
    /// Add a .orepatch to the current session (from stdin, file, or --inline)
    Add {
        #[arg(default_value = "")]
        source: String,
        #[arg(long)]
        inline: Option<String>,
    },
    /// Show current session contents
    Show,
    /// List all saved sessions
    List,
    /// Apply the current session
    Apply {
        #[arg(long)]
        atomic: bool,
        #[arg(long = "verify", short = 'w')]
        verify_cmd: Option<String>,
    },
    /// Clear the current session
    Clear,
    /// Delete a named session
    Rm { name: String },
    /// Switch to a different named session
    Switch { name: String },
}

pub fn run(args: PatchSessionArgs) -> Result<()> {
    let base = args.dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap());
    let sessions_dir = base.join(".ore").join("patch-sessions");
    let current_file = base.join(".ore").join("patch-session-current");

    // The ore executable running right now — use it for sub-invocations so
    // this works even when `ore` isn't on PATH.
    let ore_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().replace('\\', "\\\\"))
        .unwrap_or_else(|_| "ore".to_string());
    let ore_exe_quoted = format!("\"{}\"", ore_exe);

    std::fs::create_dir_all(&sessions_dir)?;

    match args.action {
        SessionAction::Start { name } => {
            let path = sessions_dir.join(format!("{}.orepatch", name));
            std::fs::write(&path, "")?;
            std::fs::write(&current_file, &name)?;
            println!("{} session '{}'", "Started:".green().bold(), name.cyan());
        }
        SessionAction::Add { source, inline } => {
            let name = std::fs::read_to_string(&current_file)
                .map_err(|_| anyhow::anyhow!("No active session. Run 'patch-session start <name>' first."))?;
            let content = if let Some(inline) = inline {
                inline
            } else if source == "-" || source.is_empty() {
                use std::io::Read;
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            } else {
                std::fs::read_to_string(&source)?
            };

            let path = sessions_dir.join(format!("{}.orepatch", name.trim()));
            let mut existing = std::fs::read_to_string(&path).unwrap_or_default();
            if !existing.is_empty() && !existing.ends_with('\n') {
                existing.push('\n');
            }
            if !existing.is_empty() {
                existing.push_str("===\n");
            }
            existing.push_str(&content);
            std::fs::write(&path, &existing)?;

            let ops = crate::engine::patch::parse_patch_file(&existing).unwrap_or_default();
            println!("{} session '{}' now has {} ops",
                "Added to:".green().bold(),
                name.trim().cyan(),
                ops.len().to_string().yellow()
            );
        }
        SessionAction::Show => {
            let name = std::fs::read_to_string(&current_file)
                .map_err(|_| anyhow::anyhow!("No active session"))?;
            let path = sessions_dir.join(format!("{}.orepatch", name.trim()));
            let content = std::fs::read_to_string(&path)?;
            let ops = crate::engine::patch::parse_patch_file(&content).unwrap_or_default();
            println!("{} '{}' ({} ops)", "Session:".cyan().bold(), name.trim().yellow(), ops.len().to_string().yellow());
            println!();
            print!("{}", content);
        }
        SessionAction::List => {
            let entries: Vec<_> = std::fs::read_dir(&sessions_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("orepatch"))
                .collect();
            if entries.is_empty() {
                println!("{}", "(no sessions)".dimmed());
                return Ok(());
            }
            let current = std::fs::read_to_string(&current_file).unwrap_or_default();
            let current = current.trim();
            for e in entries {
                let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                let ops = crate::engine::patch::parse_patch_file(&content).unwrap_or_default();
                let marker = if name == current { "▶".green().to_string() } else { " ".to_string() };
                println!("  {} {} ({} ops)", marker, name.cyan(), ops.len().to_string().yellow());
            }
        }
        SessionAction::Apply { atomic, verify_cmd } => {
            let name = std::fs::read_to_string(&current_file)
                .map_err(|_| anyhow::anyhow!("No active session"))?;
            let path = sessions_dir.join(format!("{}.orepatch", name.trim()));

            let cmd = if let Some(v) = verify_cmd {
                format!("{} verify-and-apply \"{}\" --with \"{}\"", ore_exe_quoted, path.display(), v)
            } else {
                let mut c = format!("{} patch-batch \"{}\"", ore_exe_quoted, path.display());
                if atomic { c.push_str(" --atomic"); }
                c
            };
            println!("{} {}", "Applying:".cyan(), cmd.dimmed());
            let result = crate::engine::proc::run_cmd_in(&cmd, Some(&base), true, false)?;
            if !result.success() {
                std::process::exit(result.exit_code);
            }
        }
        SessionAction::Clear => {
            let name = std::fs::read_to_string(&current_file)
                .map_err(|_| anyhow::anyhow!("No active session"))?;
            let path = sessions_dir.join(format!("{}.orepatch", name.trim()));
            std::fs::write(&path, "")?;
            println!("{} session '{}'", "Cleared:".green().bold(), name.trim().cyan());
        }
        SessionAction::Rm { name } => {
            let path = sessions_dir.join(format!("{}.orepatch", name));
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("{} session '{}'", "Removed:".green().bold(), name.cyan());
            } else {
                eprintln!("{} no session named '{}'", "Not found:".yellow(), name);
            }
        }
        SessionAction::Switch { name } => {
            let path = sessions_dir.join(format!("{}.orepatch", name));
            if !path.exists() {
                anyhow::bail!("Session '{}' does not exist. Use 'patch-session start' to create.", name);
            }
            std::fs::write(&current_file, &name)?;
            println!("{} to session '{}'", "Switched:".green().bold(), name.cyan());
        }
    }
    Ok(())
}
