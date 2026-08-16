use anyhow::Result;
use clap::Args;
use colored::*;
use std::process::{Command, Stdio};

/// PowerShell escape hatch — run a PowerShell one-liner directly.
/// Cross-platform: uses pwsh if available, else powershell (Windows), else fails.
#[derive(Args)]
pub struct PsArgs {
    /// PowerShell script/command to execute
    command: String,

    /// Use specific PowerShell binary (pwsh or powershell)
    #[arg(long)]
    shell: Option<String>,

    /// Don't print the exit code footer
    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: PsArgs) -> Result<()> {
    let ps = args.shell.unwrap_or_else(pick_powershell);

    if !args.quiet {
        eprintln!("{} {} {}", "→".cyan(), ps.dimmed(), "-Command ...".dimmed());
    }

    let status = Command::new(&ps)
        .args(["-NoProfile", "-NonInteractive", "-Command", &args.command])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            if !args.quiet {
                eprintln!("{} exit {}", "→".dimmed(), code.to_string().yellow());
            }
            if code != 0 {
                std::process::exit(code);
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to spawn {}: {}. Try --shell powershell", ps, e);
        }
    }
    Ok(())
}

fn pick_powershell() -> String {
    // Prefer pwsh (cross-platform) if available
    if which::which("pwsh").is_ok() {
        return "pwsh".to_string();
    }
    // Windows fallback
    #[cfg(windows)]
    { return "powershell".to_string(); }
    #[cfg(not(windows))]
    { "pwsh".to_string() }
}
