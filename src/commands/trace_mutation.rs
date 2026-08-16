use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Find every place a property/variable gets WRITTEN to.
/// Detects: `.X =`, `X:` in object literals, `X =`, `set X(`,
/// `updateX(`, `setX(`, `mutate(X:`, function params like `{X}`.
#[derive(Args)]
pub struct TraceMutationArgs {
    /// Property/variable name to trace
    pub property: String,

    /// Path to search (default .)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(short = 'e', long)]
    pub ext: Option<String>,

    #[arg(short = 'x', long)]
    pub exclude: Option<String>,

    /// Show context lines
    #[arg(short = 'C', long, default_value = "1")]
    pub context: usize,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Only show file:line summary (no code)
    #[arg(short = 'l', long)]
    pub lines_only: bool,
}

pub fn run(args: TraceMutationArgs) -> Result<()> {
    let p = regex::escape(&args.property);

    // Patterns that indicate WRITE (not read):
    //  X =                      direct assignment (but not == or ===)
    //  .X =                     property assignment
    //  X:                       object literal key or type annotation
    //  set X(                   setter
    //  setX(                    setter method
    //  updateX(                 update pattern
    //  X++, X--                 increment
    //  X +=, -=, *=, /=         compound assignment
    let patterns = vec![
        format!(r"\b{}\s*(=[^=]|=$)", p),                // X = ... (not ==)
        format!(r"\.\s*{}\s*(=[^=]|=$)", p),            // .X =
        format!(r"^[ \t]*{}\s*:", p),                    // X: (object literal / interface)
        format!(r"[,{{(]\s*{}\s*:", p),                  // { X:
        format!(r"\bset\s+{}\s*\(", p),                  // set X(
        format!(r"\bset{}[A-Z]?\w*\s*\(", capitalize(&args.property)),  // setX(
        format!(r"\bupdate{}[A-Z]?\w*\s*\(", capitalize(&args.property)),  // updateX(
        format!(r"\b{}\s*\+\+", p),
        format!(r"\b{}\s*--", p),
        format!(r"\b{}\s*[+\-*/]=", p),                  // X += etc
    ];

    let combined = patterns.join("|");
    let re = RegexBuilder::new(&combined)
        .case_insensitive(args.ignore_case)
        .multi_line(true)
        .build()?;

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: false,
        respect_gitignore: true,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;

    let mut total = 0usize;
    let mut files_hit = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        let mut hits: Vec<usize> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            // Skip comment-only lines (heuristic)
            let stripped = line.trim_start();
            if stripped.starts_with("//") || stripped.starts_with("*") || stripped.starts_with("#") {
                continue;
            }
            if re.is_match(line) {
                hits.push(i);
            }
        }

        if hits.is_empty() { continue; }
        files_hit += 1;
        total += hits.len();

        if args.lines_only {
            for i in &hits {
                println!("{}:{}", f.display(), i + 1);
            }
            continue;
        }

        println!("\n{}", f.display().to_string().cyan().bold());
        let mut printed = std::collections::HashSet::new();
        for &h in &hits {
            let s = h.saturating_sub(args.context);
            let e = (h + args.context + 1).min(lines.len());
            for i in s..e {
                if printed.contains(&i) { continue; }
                printed.insert(i);
                let ln = i + 1;
                let text = lines[i];
                if i == h {
                    let hl = re.replace_all(text, |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("  {}: {}", ln.to_string().green(), hl);
                } else {
                    println!("  {}| {}", ln.to_string().dimmed(), text.dimmed());
                }
            }
        }
    }

    eprintln!("\n{} {} write sites for {:?} in {} files",
        "trace-mutation:".bold(),
        total.to_string().yellow(),
        args.property,
        files_hit.to_string().yellow()
    );

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
