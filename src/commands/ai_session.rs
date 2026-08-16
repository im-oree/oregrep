use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::ai::session::{delete, list, load};

#[derive(Args)]
pub struct AiSessionArgs {
    #[command(subcommand)]
    pub action: SessAction,
}

#[derive(Subcommand)]
pub enum SessAction {
    /// List saved sessions with message counts
    List,
    /// Show a session's messages
    Show { name: String, #[arg(short = 'n', long, default_value = "50")] limit: i64 },
    /// Delete a session
    Rm { name: String, #[arg(short = 'y', long)] yes: bool },
}

pub fn run(args: AiSessionArgs) -> Result<()> {
    match args.action {
        SessAction::List => {
            let sessions = list()?;
            if sessions.is_empty() { println!("{}", "(no sessions)".dimmed()); return Ok(()); }
            println!("{}", "Sessions:".cyan().bold());
            for (name, created, updated, count) in sessions {
                let updated_s = chrono::DateTime::from_timestamp(updated, 0)
                    .map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                let _ = created;
                println!("  {}  {} msgs  updated {}", name.cyan(), count.to_string().yellow(), updated_s.dimmed());
            }
        }
        SessAction::Show { name, limit } => {
            let msgs = load(&name, Some(limit))?;
            if msgs.is_empty() { println!("{}", "(empty session)".dimmed()); return Ok(()); }
            for m in msgs {
                let role_c = match m.role.as_str() {
                    "user" => "user".magenta(),
                    "assistant" => "assistant".cyan(),
                    "system" => "system".dimmed(),
                    _ => m.role.as_str().dimmed(),
                };
                let ts = chrono::DateTime::from_timestamp(m.ts, 0)
                    .map(|d| d.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
                    .unwrap_or_default();
                println!("\n[{} {}]\n{}", ts.dimmed(), role_c, m.content.trim());
            }
        }
        SessAction::Rm { name, yes } => {
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Delete session '{}'?", name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            let removed = delete(&name)?;
            if removed { println!("{} {}", "Removed:".green(), name.cyan()); }
            else { println!("{} {} not found", "!".yellow(), name.cyan()); }
        }
    }
    Ok(())
}
