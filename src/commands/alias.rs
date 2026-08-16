use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::state::{aliases_path, Aliases};

#[derive(Args)]
pub struct AliasArgs {
    #[command(subcommand)]
    pub action: AliasAction,
}

#[derive(Subcommand)]
pub enum AliasAction {
    /// List all aliases
    List,
    /// Add an alias: ore alias add <name> "<commands...>"
    Add { name: String, command: String },
    /// Remove an alias
    Rm { name: String },
    /// Show the aliases file path
    Path,
    /// Run an alias by name (mainly for scripting; usually invoked as `ore <name>`)
    Run { name: String, extra: Vec<String> },
    /// Show what an alias would expand to
    Show { name: String },
}

pub fn run(args: AliasArgs) -> Result<()> {
    match args.action {
        AliasAction::List => {
            let a = Aliases::load()?;
            if a.map.is_empty() { println!("{}", "(no aliases defined)".dimmed()); return Ok(()); }
            let mut entries: Vec<(&String, &String)> = a.map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (name, cmd) in entries {
                println!("  {} → {}", name.cyan().bold(), cmd.yellow());
            }
        }
        AliasAction::Add { name, command } => {
            let mut a = Aliases::load()?;
            a.map.insert(name.clone(), command.clone());
            a.save()?;
            println!("{} {} → {}", "Added:".green().bold(), name.cyan(), command.yellow());
        }
        AliasAction::Rm { name } => {
            let mut a = Aliases::load()?;
            match a.map.remove(&name) {
                Some(v) => {
                    a.save()?;
                    println!("{} {} (was {})", "Removed:".green(), name.cyan(), v.dimmed());
                }
                None => { eprintln!("{} alias '{}' not defined", "!".yellow(), name); std::process::exit(1); }
            }
        }
        AliasAction::Path => println!("{}", aliases_path()?.display()),
        AliasAction::Run { name, extra } => {
            let a = Aliases::load()?;
            match a.map.get(&name) {
                Some(cmd) => {
                    // Build final command: cmd + extra args, then run via `ore <...>`
                    let extra_joined = extra.iter().map(|s| shell_quote(s)).collect::<Vec<_>>().join(" ");
                    let final_cmd = if extra_joined.is_empty() { format!("ore {}", cmd) } else { format!("ore {} {}", cmd, extra_joined) };
                    let result = crate::engine::proc::run_cmd(&final_cmd, true, false)?;
                    if !result.success() { std::process::exit(result.exit_code); }
                }
                None => { eprintln!("{} alias '{}' not defined", "!".red(), name); std::process::exit(1); }
            }
        }
        AliasAction::Show { name } => {
            let a = Aliases::load()?;
            match a.map.get(&name) {
                Some(cmd) => println!("ore {}", cmd),
                None => { eprintln!("{} alias '{}' not defined", "!".yellow(), name); std::process::exit(1); }
            }
        }
    }
    Ok(())
}

fn shell_quote(s: &str) -> String {
    if s.contains(' ') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}
