use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct InstallIfMissingArgs {
    /// Tool name (or comma-separated list)
    tools: String,

    /// Install source: winget | choco | npm | cargo | scoop
    #[arg(short = 's', long, default_value = "winget")]
    via: String,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: InstallIfMissingArgs) -> Result<()> {
    let list: Vec<String> = args.tools.split(',').map(|s| s.trim().to_string()).collect();
    for t in &list {
        let check_cmd = format!("{} --version", t);
        let r = run_cmd(&check_cmd, false, true)?;
        if r.success() {
            println!("  {} {} already installed", "OK".green(), t.cyan());
            continue;
        }
        println!("  {} {} not installed — attempting install via {}", "!".yellow(), t.cyan(), args.via.yellow());
        if !args.yes {
            let ok = crate::engine::confirm::confirm(&format!("Install {}?", t), false)?;
            if !ok { println!("  {}", "skipped".dimmed()); continue; }
        }
        let install_cmd = match args.via.as_str() {
            "winget" => format!("winget install --id {} --accept-source-agreements --accept-package-agreements", t),
            "choco" => format!("choco install {} -y", t),
            "scoop" => format!("scoop install {}", t),
            "npm" => format!("npm install -g {}", t),
            "cargo" => format!("cargo install {}", t),
            other => anyhow::bail!("Unknown install source: {}", other),
        };
        let r2 = run_cmd(&install_cmd, true, false)?;
        if r2.success() {
            println!("  {} installed {}", "OK".green().bold(), t.cyan());
        } else {
            println!("  {} failed to install {} (exit {})", "FAIL".red().bold(), t.cyan(), r2.exit_code);
        }
    }
    Ok(())
}
