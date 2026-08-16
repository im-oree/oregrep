use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::io::Read;
use std::path::PathBuf;

use crate::engine::proc::run_cmd;
use crate::engine::storage::{macro_path, macros_dir};

#[derive(Args)]
pub struct MacroArgs {
    #[command(subcommand)]
    pub action: MacroAction,
}

#[derive(Subcommand)]
pub enum MacroAction {
    /// Save a macro (sequence of commands, one per line) from stdin or --file
    Save {
        name: String,
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Run a macro (executes each command sequentially)
    Run {
        name: String,
        /// Continue if a command fails
        #[arg(short = 'c', long)]
        continue_on_error: bool,
        /// Stream output live
        #[arg(short = 's', long)]
        stream: bool,
        /// Show the commands but don't execute
        #[arg(long)]
        dry_run: bool,
    },
    /// List saved macros
    List,
    /// Show a macro's content
    Show { name: String },
    /// Delete a macro
    Rm {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show macro file path
    Path { name: String },
    /// Export a macro to a file
    Export { name: String, output: PathBuf },
}

pub fn run(args: MacroArgs) -> Result<()> {
    match args.action {
        MacroAction::Save { name, file, force } => {
            let path = macro_path(&name)?;
            if path.exists() && !force { anyhow::bail!("Macro exists: {} (use --force)", name); }
            let content = if let Some(f) = file {
                std::fs::read_to_string(f)?
            } else {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            };
            let n_cmds = content.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#')).count();
            std::fs::write(&path, &content)?;
            println!("{} {} ({} commands)", "Saved:".green().bold(), name.cyan(), n_cmds.to_string().yellow());
        }
        MacroAction::Run { name, continue_on_error, stream, dry_run } => {
            let path = macro_path(&name)?;
            if !path.exists() { anyhow::bail!("Macro not found: {}", name); }
            let content = std::fs::read_to_string(&path)?;
            let cmds: Vec<&str> = content.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();

            println!("{} {} ({} commands)", "Running:".cyan().bold(), name.cyan(), cmds.len().to_string().yellow());
            let mut ok = 0usize;
            let mut fail = 0usize;
            for (i, cmd) in cmds.iter().enumerate() {
                println!("\n{} [{}/{}] {}", "▶".cyan(), (i + 1).to_string().yellow(), cmds.len().to_string().yellow(), cmd.dimmed());
                if dry_run { continue; }
                let r = run_cmd(cmd, stream, false)?;
                if !stream {
                    if !r.stdout.is_empty() { print!("{}", r.stdout); }
                    if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
                }
                if r.success() {
                    ok += 1;
                    println!("{} ({}ms)", "OK".green(), r.duration_ms.to_string().dimmed());
                } else {
                    fail += 1;
                    eprintln!("{} exit {} ({}ms)", "FAIL".red().bold(), r.exit_code, r.duration_ms.to_string().dimmed());
                    if !continue_on_error {
                        eprintln!("{} macro halted at step {}", "!".red(), i + 1);
                        std::process::exit(1);
                    }
                }
            }
            println!("\n{} {} ok, {} failed", "Summary:".bold(), ok.to_string().green(), fail.to_string().red());
            if fail > 0 { std::process::exit(1); }
        }
        MacroAction::List => {
            let d = macros_dir()?;
            let mut count = 0;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("macro") {
                    let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                    let ncmds = content.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#')).count();
                    println!("  {} ({} commands)", name.cyan(), ncmds.to_string().yellow());
                    count += 1;
                }
            }
            if count == 0 { println!("{}", "(no macros)".dimmed()); }
        }
        MacroAction::Show { name } => {
            let path = macro_path(&name)?;
            if !path.exists() { anyhow::bail!("Macro not found: {}", name); }
            print!("{}", std::fs::read_to_string(&path)?);
        }
        MacroAction::Rm { name, yes } => {
            let path = macro_path(&name)?;
            if !path.exists() { anyhow::bail!("Macro not found: {}", name); }
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Delete macro '{}'?", name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            std::fs::remove_file(&path)?;
            println!("{} {}", "Deleted:".green(), name.cyan());
        }
        MacroAction::Path { name } => {
            println!("{}", macro_path(&name)?.display());
        }
        MacroAction::Export { name, output } => {
            let path = macro_path(&name)?;
            if !path.exists() { anyhow::bail!("Macro not found: {}", name); }
            std::fs::copy(&path, &output)?;
            println!("{} {} → {}", "Exported:".green(), name.cyan(), output.display().to_string().cyan());
        }
    }
    Ok(())
}
