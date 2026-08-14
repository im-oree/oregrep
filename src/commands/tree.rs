use anyhow::Result;
use clap::Args;
use colored::*;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct TreeArgs {
    /// Directory to display
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Max depth
    #[arg(short = 'd', long)]
    depth: Option<usize>,

    /// Include hidden files
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    no_ignore: bool,

    /// Show file sizes
    #[arg(short = 's', long)]
    size: bool,

    /// Filter by extension (comma-separated, e.g. "ts,rs")
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Show only directories
    #[arg(short = 'D', long)]
    dirs_only: bool,
}

pub fn run(args: TreeArgs) -> Result<()> {
    if !args.path.exists() {
        anyhow::bail!("Path not found: {}", args.path.display());
    }

    let ext_filter: Option<Vec<String>> = args.ext.as_ref().map(|s| {
        s.split(',')
            .map(|e| e.trim().trim_start_matches('.').to_lowercase())
            .collect()
    });

    let mut builder = WalkBuilder::new(&args.path);
    builder
        .hidden(!args.hidden)
        .git_ignore(!args.no_ignore)
        .git_global(!args.no_ignore)
        .git_exclude(!args.no_ignore);
    if let Some(d) = args.depth {
        builder.max_depth(Some(d + 1));
    }

    let root = args.path.canonicalize().unwrap_or_else(|_| args.path.clone());
    let root_str = root.display().to_string();
    // Strip Windows extended-length path prefix (e.g. \\?\C:\...)
    let root_str = root_str.strip_prefix("\\\\?\\").unwrap_or(&root_str);
    println!("{}", root_str.cyan().bold());

    let mut file_count: usize = 0;
    let mut dir_count: usize = 0;

    for entry in builder.build().flatten() {
        let path = entry.path();
        if path == args.path.as_path() {
            continue;
        }

        let is_dir = path.is_dir();
        if args.dirs_only && !is_dir {
            continue;
        }

        // Extension filter (files only)
        if !is_dir {
            if let Some(filters) = &ext_filter {
                let matches_ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| filters.iter().any(|f| f == &e.to_lowercase()))
                    .unwrap_or(false);
                if !matches_ext {
                    continue;
                }
            }
        }

        // Depth for indent
        let depth = path
            .strip_prefix(&args.path)
            .map(|p| p.components().count())
            .unwrap_or(1);

        let indent = "  ".repeat(depth.saturating_sub(1));
        let branch = "├─ ";
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let display_name = if is_dir {
            format!("{}/", name).blue().bold().to_string()
        } else {
            colorize_file(&name, path)
        };

        if args.size && !is_dir {
            let sz = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let sz_str = format_size(sz);
            println!("{}{}{}  {}", indent, branch.dimmed(), display_name, sz_str.dimmed());
        } else {
            println!("{}{}{}", indent, branch.dimmed(), display_name);
        }

        if is_dir {
            dir_count += 1;
        } else {
            file_count += 1;
        }
    }

    eprintln!(
        "\n{} dirs, {} files",
        dir_count.to_string().yellow(),
        file_count.to_string().yellow()
    );

    Ok(())
}

fn colorize_file(name: &str, path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "rs" => name.truecolor(222, 165, 132).to_string(),
        "ts" | "tsx" => name.cyan().to_string(),
        "js" | "jsx" | "mjs" => name.yellow().to_string(),
        "json" | "toml" | "yaml" | "yml" => name.magenta().to_string(),
        "md" => name.white().bold().to_string(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => name.purple().to_string(),
        "exe" | "dll" | "bin" => name.red().to_string(),
        _ => name.normal().to_string(),
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}
