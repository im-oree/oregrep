use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::git::{ensure_repo, git};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportContributorsArgs {
    #[arg(short = 's', long)]
    since: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportContributorsArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd = vec!["log".to_string(), "--pretty=format:%an\t%ae".to_string()];
    if let Some(s) = &args.since { cmd.push(format!("--since={}", s)); }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;

    let mut counts: HashMap<String, (String, usize)> = HashMap::new();
    let mut total = 0usize;
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.is_empty() { continue; }
        let name = parts[0].to_string();
        let email = parts.get(1).unwrap_or(&"").to_string();
        let entry = counts.entry(name).or_insert((email, 0));
        entry.1 += 1;
        total += 1;
    }
    let mut rows: Vec<(String, String, usize)> = counts.into_iter().map(|(n, (e, c))| (n, e, c)).collect();
    rows.sort_by(|a, b| b.2.cmp(&a.2));

    let mut md = String::new();
    md.push_str("# Contributors\n\n");
    md.push_str(&format!("_{} commits total_\n\n", total));
    md.push_str("| # | Name | Email | Commits | % |\n|---:|---|---|---:|---:|\n");
    for (i, (name, email, count)) in rows.iter().enumerate() {
        let pct = if total > 0 { (*count as f64 / total as f64) * 100.0 } else { 0.0 };
        md.push_str(&format!("| {} | {} | `{}` | {} | {:.1}% |\n", i + 1, name, email, count, pct));
    }
    write_out(&md, args.output)
}
