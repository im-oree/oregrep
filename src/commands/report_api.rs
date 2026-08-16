use anyhow::Result;
use clap::Args;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportApiArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportApiArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;

    let mut md = String::new();
    md.push_str("# Public API Surface\n\n");
    md.push_str(&format!("_Generated: {}_\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    let mut sorted: Vec<(&PathBuf, &Vec<crate::engine::symbols::Symbol>)> = g.symbols.iter().collect();
    sorted.sort_by_key(|(p, _)| (*p).clone());

    let mut kind_totals: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_exported = 0usize;

    for (p, syms) in &sorted {
        let exports: Vec<&crate::engine::symbols::Symbol> = syms.iter().filter(|s| s.exported).collect();
        if exports.is_empty() { continue; }
        md.push_str(&format!("## `{}`\n\n", short_path(&args.path, p)));
        for s in &exports {
            let kind = format!("{:?}", s.kind).to_lowercase();
            *kind_totals.entry(kind.clone()).or_insert(0) += 1;
            total_exported += 1;
            md.push_str(&format!("- **{}** _{}_  L{}\n", s.name, kind, s.line));
        }
        md.push('\n');
    }

    md.insert_str(0, &format!("**Total exports:** {}\n\n", total_exported));
    if !kind_totals.is_empty() {
        let mut counts = String::from("**By kind:** ");
        let parts: Vec<String> = kind_totals.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
        counts.push_str(&parts.join(", "));
        counts.push_str("\n\n");
        md.insert_str(0, &counts);
    }
    md.insert_str(0, "# Public API Surface\n\n");
    write_out(&md, args.output)
}
