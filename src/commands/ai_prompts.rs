use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::ai::prompts::{default_for, get, list, prompts_dir, reset};

#[derive(Args)]
pub struct AiPromptsArgs {
    #[command(subcommand)]
    pub action: PromptsAction,
}

#[derive(Subcommand)]
pub enum PromptsAction {
    List,
    Show { name: String },
    Path { name: Option<String> },
    Edit { name: String },
    Reset { name: String, #[arg(short = 'y', long)] yes: bool },
    Diff { name: String },
}

pub fn run(args: AiPromptsArgs) -> Result<()> {
    match args.action {
        PromptsAction::List => {
            for name in list()? { println!("  {}", name.cyan()); }
        }
        PromptsAction::Show { name } => {
            let text = get(&name)?;
            print!("{}", text);
        }
        PromptsAction::Path { name } => {
            let dir = prompts_dir()?;
            match name {
                Some(n) => println!("{}", dir.join(format!("{}.md", n)).display()),
                None => println!("{}", dir.display()),
            }
        }
        PromptsAction::Edit { name } => {
            let dir = prompts_dir()?;
            let path = dir.join(format!("{}.md", name));
            if !path.exists() { anyhow::bail!("Prompt not found: {}", name); }
            let editor = std::env::var("EDITOR").ok()
                .or_else(|| std::env::var("VISUAL").ok())
                .unwrap_or_else(|| if cfg!(windows) { "notepad".to_string() } else { "vi".to_string() });
            std::process::Command::new(&editor).arg(&path).status().ok();
        }
        PromptsAction::Reset { name, yes } => {
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Reset prompt '{}' to bundled default?", name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            reset(&name)?;
            println!("{} {}", "Reset:".green().bold(), name.cyan());
        }
        PromptsAction::Diff { name } => {
            let current = get(&name)?;
            let default = default_for(&name).ok_or_else(|| anyhow::anyhow!("No bundled default for '{}'", name))?;
            if current == default {
                println!("{}", "(no changes from default)".dimmed());
                return Ok(());
            }
            // Simple line diff
            use similar::{ChangeTag, TextDiff};
            let diff = TextDiff::from_lines(default, &current);
            for change in diff.iter_all_changes() {
                let tag = match change.tag() {
                    ChangeTag::Delete => "-".red().to_string(),
                    ChangeTag::Insert => "+".green().to_string(),
                    ChangeTag::Equal => " ".dimmed().to_string(),
                };
                print!("{} {}", tag, change.value());
            }
        }
    }
    Ok(())
}
