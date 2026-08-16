use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Find every place a property is assigned a falsy/zero/reset value.
/// Detects: X = 0, X = null, X = undefined, X = false, X = "", X = []
#[derive(Args)]
pub struct WhyResetArgs {
    property: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    #[arg(short = 'C', long, default_value = "2")]
    context: usize,

    #[arg(short = 'i', long)]
    ignore_case: bool,
}

pub fn run(args: WhyResetArgs) -> Result<()> {
    let p = regex::escape(&args.property);
    // Match: X = 0 | X = null | X = undefined | X = false | X = "" | X = '' | X = [] | X = {}
    // (raw string with ## delimiters so the literal "" and '' are legal)
    let pattern = format!(
        r##"\b{}\s*=\s*(0([^\d.]|$)|null|undefined|false|""|''|\[\s*\]|\{{\s*\}}|None|nil)"##,
        p
    );
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(args.ignore_case)
        .multi_line(true)
        .build()?;

    // Also object-literal form: X: 0 / X: null etc
    let pattern2 = format!(
        r##"\b{}\s*:\s*(0([^\d.]|$)|null|undefined|false|""|''|\[\s*\]|\{{\s*\}}|None|nil)"##,
        p
    );
    let re2 = RegexBuilder::new(&pattern2)
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

    let mut total = 0usize;
    let mut files_hit = 0usize;

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        let hits: Vec<usize> = lines.iter().enumerate()
            .filter_map(|(i, l)| {
                let stripped = l.trim_start();
                if stripped.starts_with("//") || stripped.starts_with("*") { return None; }
                if re.is_match(l) || re2.is_match(l) { Some(i) } else { None }
            })
            .collect();

        if hits.is_empty() { continue; }
        files_hit += 1;
        total += hits.len();

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
                    let hl1 = re.replace_all(text, |c: &regex::Captures| c[0].red().bold().to_string());
                    let hl2 = re2.replace_all(&hl1, |c: &regex::Captures| c[0].red().bold().to_string());
                    println!("  {}: {}", ln.to_string().green(), hl2);
                } else {
                    println!("  {}| {}", ln.to_string().dimmed(), text.dimmed());
                }
            }
        }
    }

    eprintln!("\n{} {} reset sites for {:?} in {} files",
        "why-reset:".bold(),
        total.to_string().yellow(),
        args.property,
        files_hit.to_string().yellow()
    );

    if total == 0 {
        std::process::exit(1);
    }

    Ok(())
}
