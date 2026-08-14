use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct SearchFuzzyArgs {
    /// Query (case-insensitive, typo-tolerant)
    query: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    /// Max edit distance (default 2). Higher = more permissive.
    #[arg(short = 'd', long, default_value = "2")]
    distance: usize,

    /// Search filenames only (not content)
    #[arg(short = 'f', long)]
    filenames_only: bool,

    /// Min token length to bother matching (default 3)
    #[arg(long, default_value = "3")]
    min_token: usize,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    /// Max results
    #[arg(short = 'n', long, default_value = "50")]
    limit: usize,
}

pub fn run(args: SearchFuzzyArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;
    let q = args.query.to_lowercase();

    // Score = min edit distance to any token in file (or filename)
    let mut hits: Vec<(usize, PathBuf, Option<(usize, String, String)>)> = Vec::new();

    for f in &files {
        let fname = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();

        // Try filename match first
        let fname_dist = strsim::levenshtein(&q, &fname);
        if fname_dist <= args.distance {
            hits.push((fname_dist, f.clone(), None));
            continue;
        }
        // Substring in filename
        if fname.contains(&q) {
            hits.push((0, f.clone(), None));
            continue;
        }

        if args.filenames_only { continue; }

        // Content search
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let mut best: Option<(usize, String, String)> = None; // (dist, token, line)

        for (lineno, line) in content.lines().enumerate() {
            for token in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
                if token.len() < args.min_token { continue; }
                let tl = token.to_lowercase();
                let d = strsim::levenshtein(&q, &tl);
                if d <= args.distance {
                    if best.as_ref().map(|(bd, _, _)| d < *bd).unwrap_or(true) {
                        best = Some((d, token.to_string(), format!("{}: {}", lineno + 1, line.trim())));
                    }
                }
            }
        }
        if let Some(b) = best {
            hits.push((b.0, f.clone(), Some(b)));
        }
    }

    hits.sort_by_key(|(d, _, _)| *d);
    hits.truncate(args.limit);

    if hits.is_empty() {
        println!("{} No fuzzy matches for '{}' (distance <= {})", "!".yellow(), args.query, args.distance);
        return Ok(());
    }

    for (dist, path, ctx) in &hits {
        let d = if *dist == 0 { "exact".green().to_string() } else { format!("d={}", dist).yellow().to_string() };
        println!("{} {}", d, path.display().to_string().cyan());
        if let Some((_, token, line)) = ctx {
            println!("  {} `{}`  {}", "→".dimmed(), token.magenta(), line.dimmed());
        }
    }
    eprintln!("\n{} fuzzy matches", hits.len().to_string().yellow());
    Ok(())
}
