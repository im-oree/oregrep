use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportImportsArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(short = 'n', long, default_value = "30")]
    top: usize,
}

pub fn run(args: ReportImportsArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;

    let mut md = String::new();
    md.push_str("# Import Graph Report\n\n");
    md.push_str(&format!("**Files:** {}\n\n", g.deps.len()));

    let mut fanout: Vec<(&PathBuf, usize)> = g.deps.iter().map(|(k, v)| (k, v.len())).collect();
    fanout.sort_by(|a, b| b.1.cmp(&a.1));
    let mut fanin: Vec<(&PathBuf, usize)> = g.deps_reverse.iter().map(|(k, v)| (k, v.len())).collect();
    fanin.sort_by(|a, b| b.1.cmp(&a.1));

    md.push_str(&format!("## Top {} — Fanout (imports many things)\n\n", args.top));
    md.push_str("| # | File | Imports |\n|---:|---|---:|\n");
    for (i, (p, n)) in fanout.iter().take(args.top).enumerate() {
        md.push_str(&format!("| {} | `{}` | {} |\n", i + 1, short_path(&args.path, p), n));
    }

    md.push_str(&format!("\n## Top {} — Fanin (imported by many)\n\n", args.top));
    md.push_str("| # | File | Importers |\n|---:|---|---:|\n");
    for (i, (p, n)) in fanin.iter().take(args.top).enumerate() {
        md.push_str(&format!("| {} | `{}` | {} |\n", i + 1, short_path(&args.path, p), n));
    }
    write_out(&md, args.output)
}
