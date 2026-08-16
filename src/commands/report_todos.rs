use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::analysis::short_path;
use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportTodosArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportTodosArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;
    let re = regex::Regex::new(r"(TODO|FIXME|HACK|XXX|NOTE)\b[:\s]*(.*)").unwrap();

    let mut md = String::new();
    md.push_str("# TODO / FIXME / HACK Report\n\n");
    md.push_str(&format!("_Generated: {}_\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    let mut by_kind: std::collections::HashMap<String, Vec<(PathBuf, usize, String)>> = std::collections::HashMap::new();
    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = re.captures(line) {
                let kind = cap[1].to_string();
                let text = cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                by_kind.entry(kind).or_default().push((f.clone(), i + 1, text));
            }
        }
    }
    let total: usize = by_kind.values().map(|v| v.len()).sum();
    md.push_str(&format!("**Total:** {} items across {} categories.\n\n", total, by_kind.len()));

    let mut keys: Vec<&String> = by_kind.keys().collect();
    keys.sort();
    for k in keys {
        let items = &by_kind[k];
        md.push_str(&format!("## {} ({})\n\n", k, items.len()));
        for (p, ln, text) in items {
            md.push_str(&format!("- `{}:{}` — {}\n", short_path(&args.path, p), ln, text));
        }
        md.push('\n');
    }
    write_out(&md, args.output)
}
