use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::short_path;
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, WalkConfig};

#[derive(Args)]
pub struct AnalyzeTypeCoverageArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: AnalyzeTypeCoverageArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: vec!["ts".into(),"tsx".into()],
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;
    let any_re = regex::Regex::new(r":\s*any\b|<any>|\bas\s+any\b").unwrap();

    let mut total_any = 0usize;
    let mut total_lines = 0usize;
    let mut rows: Vec<(PathBuf, usize, usize)> = Vec::new();
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines = content.lines().count();
        let anys = any_re.find_iter(&content).count();
        total_any += anys;
        total_lines += lines;
        if anys > 0 { rows.push((f.clone(), anys, lines)); }
    }
    rows.sort_by(|a, b| b.1.cmp(&a.1));

    let pct = if total_lines == 0 { 0.0 } else { 100.0 * (1.0 - total_any as f64 / total_lines as f64) };
    println!("{} {} `any` usages in {} lines ({:.2}% strict)",
        "Type coverage:".cyan().bold(),
        total_any.to_string().yellow(),
        total_lines.to_string().dimmed(),
        pct);
    for (f, anys, _lines) in rows.iter().take(args.top) {
        println!("  {:>4}  {}", anys.to_string().yellow(), short_path(&args.path, f).cyan());
    }
    Ok(())
}
