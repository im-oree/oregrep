use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::state::{
    current_session_name, load_session, save_session, sessions_dir, set_current_session, Session, SessionEvent,
};

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub action: SessionAction,
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Start a session (all backup ops after this can be tracked)
    Start { name: String },
    /// End the current session
    End,
    /// Show the currently active session
    Current,
    /// List all saved sessions
    List,
    /// Show the log of events in a session (or the current one)
    Log { name: Option<String> },
    /// Add a manual note to the current session
    Note { message: String },
    /// Manually record a backup event (usually done automatically)
    Record { file: String, backup: String },
    /// Delete a session's log
    Rm {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show the sessions directory
    Path,
}

pub fn run(args: SessionArgs) -> Result<()> {
    match args.action {
        SessionAction::Start { name } => {
            let session = Session {
                name: name.clone(),
                started_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                events: vec![],
            };
            save_session(&session)?;
            set_current_session(Some(&name))?;
            println!("{} {} (at {})", "Session started:".green().bold(), name.cyan(), session.started_at.dimmed());
        }
        SessionAction::End => {
            match current_session_name()? {
                Some(n) => {
                    set_current_session(None)?;
                    let s = load_session(&n)?;
                    println!("{} {} ({} events recorded)",
                        "Session ended:".green().bold(),
                        n.cyan(),
                        s.events.len().to_string().yellow());
                }
                None => println!("{}", "No active session.".yellow()),
            }
        }
        SessionAction::Current => {
            match current_session_name()? {
                Some(n) => println!("{}", n.cyan()),
                None => println!("{}", "(no active session)".dimmed()),
            }
        }
        SessionAction::List => {
            let d = sessions_dir()?;
            let mut found = 0usize;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("toml") {
                    let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let s = load_session(&name).ok();
                    let count = s.as_ref().map(|x| x.events.len()).unwrap_or(0);
                    let started = s.as_ref().map(|x| x.started_at.clone()).unwrap_or_default();
                    println!("  {} {} ({} events)", name.cyan(), started.dimmed(), count.to_string().yellow());
                    found += 1;
                }
            }
            if found == 0 { println!("{}", "(no sessions)".dimmed()); }
        }
        SessionAction::Log { name } => {
            let n = match name {
                Some(x) => x,
                None => match current_session_name()? {
                    Some(x) => x,
                    None => { eprintln!("{}", "No session name given and no active session.".red()); std::process::exit(1); }
                },
            };
            let s = load_session(&n)?;
            println!("{} {} ({}, {} events)",
                "Session:".cyan().bold(),
                s.name.cyan(),
                s.started_at.dimmed(),
                s.events.len().to_string().yellow());
            for e in &s.events {
                let kind = match e.kind.as_str() {
                    "backup" => "backup".green(),
                    "delete" => "delete".red(),
                    "note" => "note".magenta(),
                    _ => e.kind.as_str().dimmed(),
                };
                let file = e.file.as_deref().unwrap_or("");
                let msg = e.message.as_deref().unwrap_or("");
                println!("  [{}] {} {} {}",
                    e.timestamp.dimmed(),
                    kind,
                    file.cyan(),
                    msg.dimmed());
            }
        }
        SessionAction::Note { message } => {
            let name = match current_session_name()? {
                Some(x) => x,
                None => { eprintln!("{}", "No active session (start one first).".red()); std::process::exit(1); }
            };
            let mut s = load_session(&name)?;
            s.events.push(SessionEvent {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                kind: "note".to_string(),
                file: None,
                backup: None,
                message: Some(message.clone()),
            });
            save_session(&s)?;
            println!("{} {}", "Note added:".green(), message.dimmed());
        }
        SessionAction::Record { file, backup } => {
            let name = match current_session_name()? {
                Some(x) => x,
                None => { eprintln!("{}", "No active session.".red()); std::process::exit(1); }
            };
            let mut s = load_session(&name)?;
            s.events.push(SessionEvent {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                kind: "backup".to_string(),
                file: Some(file.clone()),
                backup: Some(backup.clone()),
                message: None,
            });
            save_session(&s)?;
            println!("{} {} → {}", "Recorded:".green(), file.cyan(), backup.dimmed());
        }
        SessionAction::Rm { name, yes } => {
            let p = crate::engine::state::session_path(&name)?;
            if !p.exists() { eprintln!("{} '{}' not found", "!".yellow(), name); std::process::exit(1); }
            let ok = crate::engine::confirm::confirm(&format!("Delete session '{}'?", name), yes)?;
            if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            std::fs::remove_file(&p)?;
            if current_session_name()?.as_deref() == Some(name.as_str()) {
                set_current_session(None)?;
            }
            println!("{} {}", "Deleted:".green(), name.cyan());
        }
        SessionAction::Path => println!("{}", sessions_dir()?.display()),
    }
    Ok(())
}
