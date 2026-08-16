use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Tag files with labels for session tracking.
/// Stored in .ore/tags.json per workspace.
#[derive(Args)]
pub struct TagArgs {
    #[command(subcommand)]
    action: TagAction,

    /// Working directory (default: current dir)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum TagAction {
    /// Add tag(s) to a file: ore tag add <file> <tags...>
    Add {
        file: String,
        /// Tags to add (e.g. read patched reviewed)
        tags: Vec<String>,
    },
    /// Remove tag(s) from a file
    Rm {
        file: String,
        /// Tags to remove
        tags: Vec<String>,
    },
    /// List all tags for a file
    Get {
        file: String,
    },
    /// List all files with a specific tag
    Files {
        tag: String,
    },
    /// List all files and their tags
    List,
    /// Clear all tags from a file
    ClearFile {
        file: String,
    },
    /// Clear all tags
    ClearAll,
    /// Show tag summary (counts per tag)
    Summary,
}

// file -> set of tags
type TagMap = BTreeMap<String, BTreeSet<String>>;

fn tags_path(dir: &Option<PathBuf>) -> PathBuf {
    let base = dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join(".ore").join("tags.json")
}

fn ensure_dir(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn load_tags(path: &PathBuf) -> Result<TagMap> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(path)?;
    let map: TagMap = serde_json::from_str(&content)?;
    Ok(map)
}

fn save_tags(path: &PathBuf, tags: &TagMap) -> Result<()> {
    ensure_dir(path)?;
    // Remove empty entries before saving
    let cleaned: TagMap = tags
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let json = serde_json::to_string_pretty(&cleaned)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn run(args: TagArgs) -> Result<()> {
    let path = tags_path(&args.dir);

    match args.action {
        TagAction::Add { file, tags } => {
            if tags.is_empty() {
                anyhow::bail!("Provide at least one tag");
            }
            let mut tag_map = load_tags(&path)?;
            let entry = tag_map.entry(file.clone()).or_insert_with(BTreeSet::new);
            for t in &tags {
                entry.insert(t.clone());
            }
            save_tags(&path, &tag_map)?;
            println!(
                "{} {} → {}",
                "Tagged:".green().bold(),
                file.cyan(),
                tags.join(", ").yellow()
            );
        }

        TagAction::Rm { file, tags } => {
            let mut tag_map = load_tags(&path)?;
            if let Some(entry) = tag_map.get_mut(&file) {
                for t in &tags {
                    entry.remove(t);
                }
                save_tags(&path, &tag_map)?;
                println!("{} removed {} from {}", "Tags:".green().bold(), tags.join(", ").yellow(), file.cyan());
            } else {
                eprintln!("{} file {:?} has no tags", "Not found:".yellow(), file);
            }
        }

        TagAction::Get { file } => {
            let tag_map = load_tags(&path)?;
            match tag_map.get(&file) {
                Some(tags) if !tags.is_empty() => {
                    println!("{}", tags.iter().cloned().collect::<Vec<_>>().join(" "));
                }
                _ => {
                    println!("{}", "(no tags)".dimmed());
                }
            }
        }

        TagAction::Files { tag } => {
            let tag_map = load_tags(&path)?;
            let files: Vec<&String> = tag_map
                .iter()
                .filter(|(_, tags)| tags.contains(&tag))
                .map(|(f, _)| f)
                .collect();

            if files.is_empty() {
                println!("{} no files with tag {:?}", "None:".yellow(), tag);
                return Ok(());
            }

            for f in &files {
                println!("  {}", f.cyan());
            }
            println!("\n{} {} file{}", "Total:".dimmed(), files.len().to_string().yellow(), if files.len() == 1 { "" } else { "s" });
        }

        TagAction::List => {
            let tag_map = load_tags(&path)?;
            if tag_map.is_empty() {
                println!("{}", "(no tags)".dimmed());
                return Ok(());
            }

            let max_file = tag_map.keys().map(|k| k.len()).max().unwrap_or(0).min(60);
            for (file, tags) in &tag_map {
                let tags_str: String = tags.iter().cloned().collect::<Vec<_>>().join(", ");
                println!("  {:<width$}  {}", file.cyan(), tags_str.yellow(), width = max_file);
            }
            println!("\n{} {} file{}", "Total:".dimmed(), tag_map.len().to_string().yellow(), if tag_map.len() == 1 { "" } else { "s" });
        }

        TagAction::ClearFile { file } => {
            let mut tag_map = load_tags(&path)?;
            if tag_map.remove(&file).is_some() {
                save_tags(&path, &tag_map)?;
                println!("{} cleared tags from {}", "Cleared:".green().bold(), file.cyan());
            } else {
                println!("{} file {:?} had no tags", "None:".yellow(), file);
            }
        }

        TagAction::ClearAll => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("{}", "All tags cleared.".green().bold());
        }

        TagAction::Summary => {
            let tag_map = load_tags(&path)?;
            let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
            for tags in tag_map.values() {
                for t in tags {
                    *tag_counts.entry(t.clone()).or_insert(0) += 1;
                }
            }

            if tag_counts.is_empty() {
                println!("{}", "(no tags)".dimmed());
                return Ok(());
            }

            let max_tag = tag_counts.keys().map(|k| k.len()).max().unwrap_or(0).min(30);
            for (tag, count) in &tag_counts {
                println!("  {:<width$}  {} file{}", tag.yellow(), count.to_string().cyan(), if *count == 1 { "" } else { "s" }, width = max_tag);
            }
            println!("\n{} {} unique tag{}", "Total:".dimmed(), tag_counts.len().to_string().yellow(), if tag_counts.len() == 1 { "" } else { "s" });
        }
    }

    Ok(())
}
