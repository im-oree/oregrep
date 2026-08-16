use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Persistent project notes — key-value pairs stored in .ore/notes.json
/// for cross-session architecture memory.
#[derive(Args)]
pub struct NotesArgs {
    #[command(subcommand)]
    action: NotesAction,

    /// Working directory (default: current dir)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum NotesAction {
    /// Set a note: ore notes set "key" "value"
    Set {
        key: String,
        #[arg(default_value = "")]
        value: String,
    },
    /// Get a note by key
    Get {
        key: String,
    },
    /// Remove a note
    Rm {
        key: String,
    },
    /// List all notes
    List,
    /// Remove all notes
    Clear,
    /// Search notes by key or value substring
    Search {
        query: String,
    },
}

type NotesMap = BTreeMap<String, String>;

fn notes_path(dir: &Option<PathBuf>) -> PathBuf {
    let base = dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let ore_dir = base.join(".ore");
    ore_dir.join("notes.json")
}

fn ensure_dir(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
    }
    Ok(())
}

fn load_notes(path: &PathBuf) -> Result<NotesMap> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let map: NotesMap = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(map)
}

fn save_notes(path: &PathBuf, notes: &NotesMap) -> Result<()> {
    ensure_dir(path)?;
    let json = serde_json::to_string_pretty(notes)?;
    std::fs::write(path, json)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn run(args: NotesArgs) -> Result<()> {
    let path = notes_path(&args.dir);

    match args.action {
        NotesAction::Set { key, value } => {
            let mut notes = load_notes(&path)?;
            let is_update = notes.contains_key(&key);
            notes.insert(key.clone(), value.clone());
            save_notes(&path, &notes)?;

            if is_update {
                println!(
                    "{} {} = {:?}",
                    "Updated:".yellow().bold(),
                    key.cyan(),
                    value
                );
            } else {
                println!(
                    "{} {} = {:?}",
                    "Set:".green().bold(),
                    key.cyan(),
                    value
                );
            }
        }

        NotesAction::Get { key } => {
            let notes = load_notes(&path)?;
            match notes.get(&key) {
                Some(value) => println!("{}", value),
                None => {
                    eprintln!("{} key {:?} not found", "Not found:".red().bold(), key);
                    std::process::exit(1);
                }
            }
        }

        NotesAction::Rm { key } => {
            let mut notes = load_notes(&path)?;
            if notes.remove(&key).is_some() {
                save_notes(&path, &notes)?;
                println!("{} {:?}", "Removed:".green().bold(), key);
            } else {
                eprintln!("{} key {:?} not found", "Not found:".red().bold(), key);
                std::process::exit(1);
            }
        }

        NotesAction::List => {
            let notes = load_notes(&path)?;
            if notes.is_empty() {
                println!("{}", "(no notes)".dimmed());
                return Ok(());
            }
            // Find max key width for alignment
            let max_w = notes.keys().map(|k| k.len()).max().unwrap_or(0).min(40);
            for (key, value) in &notes {
                println!(
                    "  {:<width$}  {}",
                    key.cyan(),
                    value,
                    width = max_w
                );
            }
            println!(
                "\n{} {} note{}",
                "Total:".dimmed(),
                notes.len().to_string().yellow(),
                if notes.len() == 1 { "" } else { "s" }
            );
        }

        NotesAction::Clear => {
            if path.exists() {
                std::fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove {}", path.display()))?;
            }
            println!("{}", "All notes cleared.".green().bold());
        }

        NotesAction::Search { query } => {
            let notes = load_notes(&path)?;
            let query_lower = query.to_lowercase();
            let matches: Vec<(&String, &String)> = notes
                .iter()
                .filter(|(k, v)| {
                    k.to_lowercase().contains(&query_lower)
                        || v.to_lowercase().contains(&query_lower)
                })
                .collect();

            if matches.is_empty() {
                println!("{} no notes matching {:?}", "No results:".yellow(), query);
                std::process::exit(1);
            }

            let max_w = matches.iter().map(|(k, _)| k.len()).max().unwrap_or(0).min(40);
            for (key, value) in &matches {
                println!(
                    "  {:<width$}  {}",
                    key.cyan(),
                    value,
                    width = max_w
                );
            }
            println!(
                "\n{} {} match{}",
                "Found:".dimmed(),
                matches.len().to_string().yellow(),
                if matches.len() == 1 { "" } else { "es" }
            );
        }
    }

    Ok(())
}
