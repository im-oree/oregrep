use anyhow::Result;
use clap::Args;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::analysis::short_path;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportCoverageArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportCoverageArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let all = collect_files(&cfg)?;

    let mut test_stems: HashSet<String> = HashSet::new();
    for f in &all {
        let name = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if name.ends_with(".test") || name.ends_with(".spec") {
            let base = name.trim_end_matches(".test").trim_end_matches(".spec").to_string();
            test_stems.insert(base);
        }
        if f.to_string_lossy().contains("__tests__") {
            test_stems.insert(name.to_string());
        }
    }

    let mut with_test = 0usize;
    let mut without_test: Vec<PathBuf> = Vec::new();
    for f in &all {
        let name = f.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if name.ends_with(".test") || name.ends_with(".spec") { continue; }
        if f.to_string_lossy().contains("__tests__") { continue; }
        if test_stems.contains(&name) { with_test += 1; }
        else { without_test.push(f.clone()); }
    }
    let source_count = with_test + without_test.len();
    let pct = if source_count == 0 { 0.0 } else { 100.0 * with_test as f64 / source_count as f64 };

    let mut md = String::new();
    md.push_str("# Test Coverage (structural)\n\n");
    md.push_str(&format!("_{} source files, {} with matching test file ({:.1}%)_\n\n", source_count, with_test, pct));
    md.push_str("## Files WITHOUT tests\n\n");
    for f in without_test.iter().take(100) {
        md.push_str(&format!("- `{}`\n", short_path(&args.path, f)));
    }
    if without_test.len() > 100 {
        md.push_str(&format!("\n_(+{} more)_\n", without_test.len() - 100));
    }
    write_out(&md, args.output)
}
