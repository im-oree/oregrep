use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::git::{ensure_repo, git};

use crate::commands::report_health::write_out;

#[derive(Args)]
pub struct ReportChangesArgs {
    #[arg(short = 's', long)]
    since: Option<String>,
    #[arg(short = 'u', long)]
    until: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: ReportChangesArgs) -> Result<()> {
    ensure_repo()?;
    let since = args.since.clone().unwrap_or_else(|| "1 week ago".to_string());
    let mut cmd = vec![
        "log".to_string(),
        "--pretty=format:%h|%an|%ad|%s".to_string(),
        "--date=short".to_string(),
        "--shortstat".to_string(),
    ];
    cmd.push(format!("--since={}", since));
    if let Some(u) = &args.until { cmd.push(format!("--until={}", u)); }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;

    let mut md = String::new();
    md.push_str("# Change Report\n\n");
    md.push_str(&format!("_since **{}**_\n\n", since));
    md.push_str("| Hash | Author | Date | Message |\n|---|---|---|---|\n");
    let lines: Vec<&str> = out.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() { i += 1; continue; }
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            md.push_str(&format!("| `{}` | {} | {} | {} |\n", parts[0], parts[1], parts[2], parts[3]));
        }
        i += 1;
    }
    write_out(&md, args.output)
}
