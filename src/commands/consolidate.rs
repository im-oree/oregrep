use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::analysis::{function_bodies, short_path};
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct ConsolidateArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Minimum body length in chars to consider
    #[arg(short = 'm', long, default_value = "80")]
    min_len: usize,
    /// Similarity threshold (0.0-1.0)
    #[arg(short = 's', long, default_value = "0.85")]
    similarity: f64,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: ConsolidateArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    // Collect all function bodies with location
    let mut fns: Vec<(PathBuf, String, String, usize)> = Vec::new(); // (file, name, normalized-body, line)
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        for (name, body, line) in function_bodies(&content) {
            if body.len() < args.min_len { continue; }
            let normalized = normalize(&body);
            fns.push((f.clone(), name, normalized, line));
        }
    }

    // Fast candidate grouping by length bucket
    let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, (_, _, body, _)) in fns.iter().enumerate() {
        buckets.entry(body.len() / 50).or_default().push(i);
    }

    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for indexes in buckets.values() {
        for i in 0..indexes.len() {
            for j in (i + 1)..indexes.len() {
                let a = indexes[i];
                let b = indexes[j];
                let sim = jaccard_lines(&fns[a].2, &fns[b].2);
                if sim >= args.similarity {
                    pairs.push((a, b, sim));
                }
            }
        }
    }
    pairs.sort_by(|x, y| y.2.partial_cmp(&x.2).unwrap_or(std::cmp::Ordering::Equal));

    println!("{} {} near-duplicate function pairs (threshold {:.2})",
        "Consolidate opportunities:".cyan().bold(),
        pairs.len().to_string().yellow(),
        args.similarity);
    for (a, b, sim) in pairs.iter().take(args.top) {
        let (fa, na, _, la) = &fns[*a];
        let (fb, nb, _, lb) = &fns[*b];
        println!("\n  {:.2}  {}::{} (L{})  ↔  {}::{} (L{})",
            sim,
            short_path(&args.path, fa).cyan(), na.yellow(), la,
            short_path(&args.path, fb).cyan(), nb.yellow(), lb);
    }
    Ok(())
}

fn normalize(text: &str) -> String {
    // Strip whitespace, collapse tokens
    text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

fn jaccard_lines(a: &str, b: &str) -> f64 {
    use std::collections::HashSet;
    let sa: HashSet<&str> = a.lines().collect();
    let sb: HashSet<&str> = b.lines().collect();
    let inter = sa.intersection(&sb).count() as f64;
    let uni = sa.union(&sb).count() as f64;
    if uni == 0.0 { 0.0 } else { inter / uni }
}
