use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::commit_msg::{compose_message, DiffAnalysis};
use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct DiffSummaryArgs {
    /// First ref (default: HEAD~5)
    #[arg(short = 'f', long, default_value = "HEAD~5")]
    from: String,
    /// Second ref (default: HEAD)
    #[arg(short = 't', long, default_value = "HEAD")]
    to: String,
    /// English summary style: simple | conventional (default simple)
    #[arg(short = 's', long, default_value = "simple")]
    style: String,
}

pub fn run(args: DiffSummaryArgs) -> Result<()> {
    ensure_repo()?;
    // Use `git diff <from>..<to> --numstat` and `--name-status`
    let numstat = git(&["diff", &format!("{}..{}", args.from, args.to), "--numstat"])?;
    let namestat = git(&["diff", &format!("{}..{}", args.from, args.to), "--name-status"])?;

    let mut files: std::collections::HashMap<String, crate::engine::commit_msg::FileChange> = std::collections::HashMap::new();
    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 { continue; }
        let added: usize = parts[0].parse().unwrap_or(0);
        let removed: usize = parts[1].parse().unwrap_or(0);
        let path = parts[2].to_string();
        files.insert(path.clone(), crate::engine::commit_msg::FileChange { status: "M".to_string(), path, added, removed });
    }
    for line in namestat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() { continue; }
        let status = parts[0].to_string();
        let path = if status.starts_with('R') && parts.len() >= 3 {
            parts[2].to_string()
        } else if parts.len() >= 2 {
            parts[1].to_string()
        } else { continue };
        if let Some(fc) = files.get_mut(&path) {
            fc.status = status.chars().next().unwrap_or('M').to_string();
        } else {
            files.insert(path.clone(), crate::engine::commit_msg::FileChange {
                status: status.chars().next().unwrap_or('M').to_string(),
                path: path.clone(),
                added: 0,
                removed: 0,
            });
        }
    }
    let all: Vec<crate::engine::commit_msg::FileChange> = files.into_values().collect();
    if all.is_empty() {
        println!("{}", "No changes between refs.".yellow());
        return Ok(());
    }

    // Build a minimal DiffAnalysis from the ref range (the commit_msg analyzer
    // only knows the working tree, so populate the fields directly here).
    let mut ana = DiffAnalysis {
        files: all.clone(),
        total_added: all.iter().map(|f| f.added).sum(),
        total_removed: all.iter().map(|f| f.removed).sum(),
        buckets: std::collections::HashMap::new(),
        new_symbols: vec![],
        removed_symbols: vec![],
        new_files: all.iter().filter(|f| f.status == "A").map(|f| f.path.clone()).collect(),
        deleted_files: all.iter().filter(|f| f.status == "D").map(|f| f.path.clone()).collect(),
        renamed_files: vec![],
        is_config_only: false,
        is_test_only: false,
        is_docs_only: false,
        is_deps_change: false,
        touches_readme: false,
    };
    // Symbols from patch
    let patch = git(&["diff", &format!("{}..{}", args.from, args.to), "-U0"]).unwrap_or_default();
    let (news, olds) = extract_symbols(&patch);
    ana.new_symbols = news;
    ana.removed_symbols = olds;
    // Buckets
    for f in &ana.files {
        let cat = simple_bucket(&f.path);
        ana.buckets.entry(cat).or_default().push(f.path.clone());
    }

    let msg = compose_message(&ana, &args.style, true);
    println!("{}", "Change summary:".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", msg);
    println!("{}", "─".repeat(60).dimmed());
    Ok(())
}

fn simple_bucket(path: &str) -> String {
    let p = path.to_lowercase();
    if p.contains("test") || p.contains("spec") { "test".to_string() }
    else if p.ends_with(".md") { "docs".to_string() }
    else if p.contains("component") { "components".to_string() }
    else if p.contains("hook") { "hooks".to_string() }
    else if p.ends_with(".json") || p.ends_with(".toml") || p.ends_with(".yml") || p.ends_with(".yaml") { "config".to_string() }
    else { "code".to_string() }
}

fn extract_symbols(patch: &str) -> (Vec<String>, Vec<String>) {
    let add_re = regex::Regex::new(r"^\+\s*(?:export\s+(?:default\s+)?(?:async\s+)?)?(?:function|const|class|interface|type|enum)\s+(\w+)").unwrap();
    let rm_re = regex::Regex::new(r"^-\s*(?:export\s+(?:default\s+)?(?:async\s+)?)?(?:function|const|class|interface|type|enum)\s+(\w+)").unwrap();
    let rs_add = regex::Regex::new(r"^\+\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type|mod)\s+(\w+)").unwrap();
    let rs_rm = regex::Regex::new(r"^-\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type|mod)\s+(\w+)").unwrap();
    let mut a = Vec::new();
    let mut r = Vec::new();
    for line in patch.lines() {
        if let Some(cap) = add_re.captures(line).or_else(|| rs_add.captures(line)) {
            let n = cap[1].to_string();
            if !a.contains(&n) { a.push(n); }
        }
        if let Some(cap) = rm_re.captures(line).or_else(|| rs_rm.captures(line)) {
            let n = cap[1].to_string();
            if !r.contains(&n) { r.push(n); }
        }
    }
    (a, r)
}
