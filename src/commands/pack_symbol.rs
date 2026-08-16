use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Pack every file containing a symbol, with N lines context around each usage.
/// Great for "show me all uses of useStore" without dumping whole files.
#[derive(Args)]
pub struct PackSymbolArgs {
    symbol: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Lines of context around each usage
    #[arg(short = 'C', long, default_value = "5")]
    context: usize,

    /// Output format: tag (default), md, plain
    #[arg(short = 'f', long, default_value = "tag")]
    format: String,

    /// Show line numbers
    #[arg(short = 'n', long)]
    numbers: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,
}

pub fn run(args: PackSymbolArgs) -> Result<()> {
    let p = format!(r"\b{}\b", regex::escape(&args.symbol));
    let re = RegexBuilder::new(&p)
        .case_insensitive(args.ignore_case)
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

    let mut total_files = 0usize;
    let mut total_hits = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        let hits: Vec<usize> = lines.iter().enumerate()
            .filter_map(|(i, l)| if re.is_match(l) { Some(i) } else { None })
            .collect();

        if hits.is_empty() { continue; }
        total_files += 1;
        total_hits += hits.len();

        // Merge overlapping context windows
        let mut blocks: Vec<(usize, usize)> = Vec::new();
        for &h in &hits {
            let s = h.saturating_sub(args.context);
            let e = (h + args.context + 1).min(lines.len());
            if let Some(last) = blocks.last_mut() {
                if s <= last.1 {
                    last.1 = last.1.max(e);
                    continue;
                }
            }
            blocks.push((s, e));
        }

        // Render
        let file_str = f.display().to_string();
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("");

        match args.format.as_str() {
            "tag" => {
                println!("<file path=\"{}\" hits=\"{}\">", file_str, hits.len());
                for (bi, (s, e)) in blocks.iter().enumerate() {
                    if bi > 0 { println!("---"); }
                    for i in *s..*e {
                        if args.numbers {
                            println!("{:>5} │ {}", i + 1, lines[i]);
                        } else {
                            println!("{}", lines[i]);
                        }
                    }
                }
                println!("</file>");
            }
            "md" => {
                println!("### `{}` ({} hits)\n", file_str, hits.len());
                println!("```{}", ext);
                for (bi, (s, e)) in blocks.iter().enumerate() {
                    if bi > 0 { println!("// ---"); }
                    for i in *s..*e {
                        if args.numbers {
                            println!("{:>5} │ {}", i + 1, lines[i]);
                        } else {
                            println!("{}", lines[i]);
                        }
                    }
                }
                println!("```\n");
            }
            "plain" => {
                println!("=== {} ({} hits) ===", file_str, hits.len());
                for (bi, (s, e)) in blocks.iter().enumerate() {
                    if bi > 0 { println!("---"); }
                    for i in *s..*e {
                        if args.numbers {
                            println!("{:>5} │ {}", i + 1, lines[i]);
                        } else {
                            println!("{}", lines[i]);
                        }
                    }
                }
                println!();
            }
            _ => anyhow::bail!("Unknown format: {}", args.format),
        }
    }

    eprintln!("\n{} '{}' in {} files ({} total hits)",
        "pack-symbol:".bold(),
        args.symbol.cyan(),
        total_files.to_string().yellow(),
        total_hits.to_string().yellow()
    );

    Ok(())
}
