use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::analysis::{complexity_of, function_bodies, short_path};
use crate::engine::encoding::read_file_smart;
use crate::engine::git::{ensure_repo, git};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct AnalyzeHotspotArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 's', long)]
    since: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: AnalyzeHotspotArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd = vec!["log".to_string(), "--name-only".to_string(), "--pretty=format:__COMMIT__".to_string()];
    if let Some(s) = &args.since { cmd.push(format!("--since={}", s)); }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;
    let mut churn: HashMap<String, usize> = HashMap::new();
    for line in out.lines() {
        let l = line.trim();
        if l.is_empty() || l == "__COMMIT__" { continue; }
        *churn.entry(l.to_string()).or_insert(0) += 1;
    }

    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    // Compute file-level complexity as sum of function complexities
    let cwd = std::env::current_dir()?;
    let mut rows: Vec<(PathBuf, usize, usize, usize)> = Vec::new(); // (file, churn, complexity, hotspot)
    for f in &files {
        // git --name-only reports repo-root-relative paths (relative to cwd when
        // run from the repo root); short_path strips the passed root instead, so
        // try the root-relative key first, then the cwd-relative one.
        let sp = short_path(&args.path, f).replace('\\', "/");
        let sp_cwd = short_path(&cwd, f).replace('\\', "/");
        let ch = *churn.get(&sp).or_else(|| churn.get(&sp_cwd)).unwrap_or(&0);
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let fns = function_bodies(&content);
        let complexity: usize = fns.iter().map(|(_, b, _)| complexity_of(b)).sum::<usize>().max(1);
        let hotspot = ch * complexity;
        if ch > 0 && complexity > 1 {
            rows.push((f.clone(), ch, complexity, hotspot));
        }
    }
    rows.sort_by(|a, b| b.3.cmp(&a.3));

    println!("{}", "Hotspots (churn × complexity):".cyan().bold());
    println!("{:>8} {:>6} {:>6}  {}", "score".dimmed(), "churn".dimmed(), "cmplx".dimmed(), "file".dimmed());
    for (p, ch, cx, hs) in rows.iter().take(args.top) {
        let color = if *hs >= 500 { "red" } else if *hs >= 100 { "yellow" } else { "green" };
        println!("{:>8} {:>6} {:>6}  {}",
            hs.to_string().color(color).bold(),
            ch.to_string().yellow(),
            cx.to_string().yellow(),
            short_path(&args.path, p).cyan());
    }
    Ok(())
}
