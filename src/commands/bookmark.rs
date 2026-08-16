use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Bookmarks: named references to file:line locations for quick navigation.
/// Stored in .ore/bookmarks.json per workspace.
#[derive(Args)]
pub struct BookmarkArgs {
    #[command(subcommand)]
    action: BookmarkAction,

    /// Working directory (default: current dir)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum BookmarkAction {
    /// Set a bookmark: ore bookmark set <name> <file:line> [-m "description"]
    Set {
        name: String,
        /// Location as file:line (e.g. src/foo.ts:42)
        location: String,
        /// Optional description/memo
        #[arg(short = 'm', long)]
        memo: Option<String>,
    },
    /// Get a bookmark by name (prints file:line)
    Get {
        name: String,
    },
    /// Remove a bookmark
    Rm {
        name: String,
    },
    /// List all bookmarks
    List,
    /// Jump: print file content around the bookmarked line
    Jump {
        name: String,
        /// Lines of context (default: 5)
        #[arg(short = 'C', long, default_value = "5")]
        context: usize,
    },
    /// Clear all bookmarks
    Clear,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Bookmark {
    file: String,
    line: usize,
    memo: Option<String>,
}

type BookmarkMap = BTreeMap<String, Bookmark>;

fn bookmarks_path(dir: &Option<PathBuf>) -> PathBuf {
    let base = dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join(".ore").join("bookmarks.json")
}

fn ensure_dir(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn load_bookmarks(path: &PathBuf) -> Result<BookmarkMap> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(path)?;
    let map: BookmarkMap = serde_json::from_str(&content)?;
    Ok(map)
}

fn save_bookmarks(path: &PathBuf, bookmarks: &BookmarkMap) -> Result<()> {
    ensure_dir(path)?;
    let json = serde_json::to_string_pretty(bookmarks)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn parse_location(s: &str) -> Result<(String, usize)> {
    // Handle file:line format, being careful with Windows paths (C:\foo:42)
    // Strategy: find last colon where remainder is all digits
    let bytes = s.as_bytes();
    for i in (0..s.len()).rev() {
        if bytes[i] == b':' {
            let after = &s[i + 1..];
            if after.chars().all(|c| c.is_ascii_digit()) && !after.is_empty() {
                let line: usize = after.parse()?;
                return Ok((s[..i].to_string(), line));
            }
        }
    }
    anyhow::bail!("Invalid location format: {}. Use file:line (e.g. src/foo.ts:42)", s);
}

pub fn run(args: BookmarkArgs) -> Result<()> {
    let path = bookmarks_path(&args.dir);
    let cwd = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match args.action {
        BookmarkAction::Set { name, location, memo } => {
            let (file, line) = parse_location(&location)?;
            let mut bookmarks = load_bookmarks(&path)?;
            let is_update = bookmarks.contains_key(&name);
            bookmarks.insert(name.clone(), Bookmark { file: file.clone(), line, memo: memo.clone() });
            save_bookmarks(&path, &bookmarks)?;

            let action = if is_update { "Updated:" } else { "Set:" };
            println!(
                "{} {} → {}:{} {}",
                if is_update { action.yellow().bold() } else { action.green().bold() },
                name.cyan(),
                file,
                line.to_string().yellow(),
                memo.as_deref().unwrap_or("").dimmed()
            );
        }

        BookmarkAction::Get { name } => {
            let bookmarks = load_bookmarks(&path)?;
            match bookmarks.get(&name) {
                Some(b) => println!("{}:{}", b.file, b.line),
                None => {
                    eprintln!("{} bookmark {:?} not found", "Not found:".red().bold(), name);
                    std::process::exit(1);
                }
            }
        }

        BookmarkAction::Rm { name } => {
            let mut bookmarks = load_bookmarks(&path)?;
            if bookmarks.remove(&name).is_some() {
                save_bookmarks(&path, &bookmarks)?;
                println!("{} {:?}", "Removed:".green().bold(), name);
            } else {
                eprintln!("{} bookmark {:?} not found", "Not found:".red().bold(), name);
                std::process::exit(1);
            }
        }

        BookmarkAction::List => {
            let bookmarks = load_bookmarks(&path)?;
            if bookmarks.is_empty() {
                println!("{}", "(no bookmarks)".dimmed());
                return Ok(());
            }
            let max_name = bookmarks.keys().map(|k| k.len()).max().unwrap_or(0).min(30);
            for (name, b) in &bookmarks {
                let memo_str = b.memo.as_deref().unwrap_or("");
                println!(
                    "  {:<width$}  {}:{:<5}  {}",
                    name.cyan(),
                    b.file,
                    b.line.to_string().yellow(),
                    memo_str.dimmed(),
                    width = max_name
                );
            }
            println!("\n{} {} bookmark{}", "Total:".dimmed(), bookmarks.len().to_string().yellow(), if bookmarks.len() == 1 { "" } else { "s" });
        }

        BookmarkAction::Jump { name, context } => {
            let bookmarks = load_bookmarks(&path)?;
            let b = match bookmarks.get(&name) {
                Some(b) => b,
                None => {
                    eprintln!("{} bookmark {:?} not found", "Not found:".red().bold(), name);
                    std::process::exit(1);
                }
            };

            let file_path = cwd.join(&b.file);
            if !file_path.exists() {
                anyhow::bail!("Bookmarked file not found: {}", b.file);
            }

            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", b.file))?;
            let lines: Vec<&str> = content.lines().collect();
            let total = lines.len();

            if b.line == 0 || b.line > total {
                anyhow::bail!("Bookmarked line {} out of range (file has {} lines)", b.line, total);
            }

            let from = b.line.saturating_sub(context);
            let to = (b.line + context).min(total);

            println!("{} {} → {}:{}", "→".cyan(), name.cyan().bold(), b.file, b.line.to_string().yellow());
            if let Some(ref m) = b.memo {
                println!("  {}", m.dimmed());
            }
            println!();

            if from > 1 {
                println!("{}", "  ...".dimmed());
            }

            for i in from..=to {
                let line_num = i;
                let line_idx = i - 1;
                let line = lines.get(line_idx).unwrap_or(&"");
                let is_target = i == b.line;

                if is_target {
                    println!("{:>5} │ {}", line_num.to_string().yellow().bold(), line.yellow().bold());
                } else {
                    println!("{:>5} │ {}", line_num.to_string().dimmed(), line);
                }
            }

            if to < total {
                println!("{}", "  ...".dimmed());
            }
        }

        BookmarkAction::Clear => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("{}", "All bookmarks cleared.".green().bold());
        }
    }

    Ok(())
}
