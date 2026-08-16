use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub action: ScheduleAction,
}

#[derive(Subcommand)]
pub enum ScheduleAction {
    /// Create a scheduled task (Windows Task Scheduler)
    Create {
        name: String,
        command: String,
        /// Schedule: DAILY | HOURLY | MINUTE | ONCE
        #[arg(short = 't', long, default_value = "DAILY")]
        interval: String,
        /// Start time (HH:MM for daily, or N for minute/hourly)
        #[arg(short = 'a', long, default_value = "09:00")]
        at: String,
    },
    /// List tasks matching a prefix (default: ore-)
    List {
        #[arg(short = 'p', long, default_value = "ore-")]
        prefix: String,
    },
    /// Delete a scheduled task
    Rm {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Run a task now
    Run { name: String },
}

pub fn run(args: ScheduleArgs) -> Result<()> {
    match args.action {
        ScheduleAction::Create { name, command, interval, at } => {
            #[cfg(windows)]
            {
                let full_name = if name.starts_with("ore-") { name.clone() } else { format!("ore-{}", name) };
                let cmd = format!(
                    "schtasks /Create /TN \"{}\" /TR \"cmd /C {}\" /SC {} /ST {} /F",
                    full_name, command.replace('"', "\\\""), interval, at
                );
                let r = run_cmd(&cmd, false, false)?;
                if r.success() {
                    println!("{} {}", "Task created:".green().bold(), full_name.cyan());
                } else {
                    eprintln!("{}", r.stderr);
                    anyhow::bail!("schtasks failed (exit {})", r.exit_code);
                }
            }
            #[cfg(not(windows))]
            {
                anyhow::bail!("Schedule create is Windows-only for now (use cron on Unix)");
            }
        }
        ScheduleAction::List { prefix } => {
            #[cfg(windows)]
            {
                let cmd = format!("schtasks /Query /FO CSV /NH | findstr /I \"{}\"", prefix);
                let r = run_cmd(&cmd, false, true)?;
                if r.stdout.trim().is_empty() {
                    println!("{}", "(no matching tasks)".dimmed());
                } else {
                    for line in r.stdout.lines() {
                        let cols: Vec<&str> = line.split("\",\"").collect();
                        if cols.len() >= 2 {
                            let name = cols[0].trim_matches('"');
                            let status = cols.get(2).map(|s| s.trim_matches('"')).unwrap_or("");
                            println!("  {} {}", name.cyan(), status.dimmed());
                        } else {
                            println!("  {}", line);
                        }
                    }
                }
            }
            #[cfg(not(windows))]
            {
                anyhow::bail!("Schedule list is Windows-only for now");
            }
        }
        ScheduleAction::Rm { name, yes } => {
            let full_name = if name.starts_with("ore-") { name.clone() } else { format!("ore-{}", name) };
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Delete task '{}'?", full_name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            #[cfg(windows)]
            {
                let cmd = format!("schtasks /Delete /TN \"{}\" /F", full_name);
                let r = run_cmd(&cmd, false, false)?;
                if r.success() {
                    println!("{} {}", "Deleted:".green(), full_name.cyan());
                } else {
                    anyhow::bail!("schtasks failed: {}", r.stderr);
                }
            }
            #[cfg(not(windows))]
            {
                anyhow::bail!("Schedule rm is Windows-only for now");
            }
        }
        ScheduleAction::Run { name } => {
            let full_name = if name.starts_with("ore-") { name.clone() } else { format!("ore-{}", name) };
            #[cfg(windows)]
            {
                let cmd = format!("schtasks /Run /TN \"{}\"", full_name);
                let r = run_cmd(&cmd, false, false)?;
                if r.success() {
                    println!("{} {}", "Ran:".green(), full_name.cyan());
                } else {
                    anyhow::bail!("schtasks run failed: {}", r.stderr);
                }
            }
            #[cfg(not(windows))]
            {
                anyhow::bail!("Schedule run is Windows-only for now");
            }
        }
    }
    Ok(())
}
