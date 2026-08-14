use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct MapArgs {
    /// Path to map
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,

    /// Sort by: name, lines, size, exports, imports
    #[arg(short = 's', long, default_value = "name")]
    sort: String,

    /// Reverse sort
    #[arg(short = 'r', long)]
    reverse: bool,

    /// Top N files only
    #[arg(short = 'n', long, default_value = "0")]
    top: usize,
}

struct FileStat {
    path: PathBuf,
    lines: usize,
    size: u64,
    exports: usize,
    imports: usize,
}

pub fn run(args: MapArgs) -> Result<()> {
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

    let export_re = regex::Regex::new(r"(?m)^(?:export\s+(?:default\s+)?(?:const|let|var|function|class|interface|type|enum|async\s+function)|pub\s+(?:fn|struct|enum|trait|const|static|mod)|def\s+\w+)")?;
    let import_re = regex::Regex::new(r#"(?m)^(?:import\s|from\s+['"]|use\s+\w|require\(['"])"#)?;

    let mut stats: Vec<FileStat> = Vec::new();
    let mut by_ext: HashMap<String, (usize, usize)> = HashMap::new(); // (files, lines)

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines = content.lines().count();
        let size = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        let exports = export_re.find_iter(&content).count();
        let imports = import_re.find_iter(&content).count();

        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let e = by_ext.entry(ext).or_insert((0, 0));
        e.0 += 1;
        e.1 += lines;

        stats.push(FileStat { path: f.clone(), lines, size, exports, imports });
    }

    match args.sort.as_str() {
        "lines" => stats.sort_by_key(|s| s.lines),
        "size" => stats.sort_by_key(|s| s.size),
        "exports" => stats.sort_by_key(|s| s.exports),
        "imports" => stats.sort_by_key(|s| s.imports),
        _ => stats.sort_by(|a, b| a.path.cmp(&b.path)),
    }
    if args.reverse { stats.reverse(); }
    if args.top > 0 { stats.truncate(args.top); }

    println!("{}", format!("Map of {}", args.path.display()).cyan().bold());
    println!("{} files, {} total lines", files.len().to_string().yellow(), stats.iter().map(|s| s.lines).sum::<usize>().to_string().yellow());
    println!();
    println!("{:>8} {:>8} {:>4} {:>4}  {}",
        "lines".dimmed(), "size".dimmed(),
        "exp".dimmed(), "imp".dimmed(),
        "path".dimmed());
    for s in &stats {
        println!("{:>8} {:>8} {:>4} {:>4}  {}",
            s.lines.to_string().yellow(),
            format_size(s.size).green(),
            s.exports.to_string().cyan(),
            s.imports.to_string().magenta(),
            s.path.display().to_string()
        );
    }

    println!("\n{}", "By extension:".bold());
    let mut ext_entries: Vec<_> = by_ext.into_iter().collect();
    ext_entries.sort_by(|a, b| b.1.1.cmp(&a.1.1));
    for (ext, (files, lines)) in ext_entries {
        println!("  {:<10} {:>6} files  {:>10} lines",
            ext.cyan(), files.to_string().yellow(), lines.to_string().green());
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
}
