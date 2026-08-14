use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::http::{apply_headers, build_client, read_body_bytes};

#[derive(Args)]
pub struct ApiTestArgs {
    spec: PathBuf,

    #[arg(long)]
    fail_fast: bool,

    #[arg(short = 'v', long)]
    verbose: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: ApiTestArgs) -> Result<()> {
    if !args.spec.exists() { anyhow::bail!("Spec file not found: {}", args.spec.display()); }
    let content = read_file_smart(&args.spec)?;
    let tests = parse_spec(&content)?;
    if tests.is_empty() { println!("{}", "No tests in spec.".yellow()); return Ok(()); }

    let client = build_client(args.timeout, true, None)?;
    println!("{} {} tests", "Running:".cyan().bold(), tests.len().to_string().yellow());
    let mut passed = 0usize;
    let mut failed = 0usize;

    for (i, t) in tests.iter().enumerate() {
        print!("  [{}/{}] {} {} ... ",
            (i + 1).to_string().dimmed(), tests.len().to_string().dimmed(),
            t.method.cyan(), t.url.yellow());

        let method = match reqwest::Method::from_bytes(t.method.as_bytes()) {
            Ok(m) => m,
            Err(_) => { failed += 1; println!("{}", "BAD METHOD".red().bold()); continue; }
        };
        let mut req = client.request(method, &t.url);
        let mut hmap = std::collections::HashMap::new();
        for (k, v) in &t.headers { hmap.insert(k.clone(), v.clone()); }
        req = apply_headers(req, &hmap);
        if let Some(b) = &t.body { req = req.body(b.clone()); }

        let start = std::time::Instant::now();
        let resp = match req.send() {
            Ok(r) => r,
            Err(e) => { failed += 1; println!("{}\n    {}", "ERROR".red().bold(), e); if args.fail_fast { break; } continue; }
        };
        let status = resp.status().as_u16();
        let response_headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        let body = read_body_bytes(resp)?;
        let elapsed = start.elapsed().as_millis();

        let mut fails: Vec<String> = Vec::new();
        if let Some(e) = t.expect_status { if status != e { fails.push(format!("status: got {}, expected {}", status, e)); } }
        for needle in &t.expect_contains {
            if !body.windows(needle.len()).any(|w| w == needle.as_bytes()) { fails.push(format!("body should contain '{}'", needle)); }
        }
        for needle in &t.expect_not_contains {
            if body.windows(needle.len()).any(|w| w == needle.as_bytes()) { fails.push(format!("body should NOT contain '{}'", needle)); }
        }
        for (hk, hv) in &t.expect_headers {
            let ok = response_headers.iter().any(|(k, v)| k.eq_ignore_ascii_case(hk) && v.contains(hv));
            if !ok { fails.push(format!("header '{}' should contain '{}'", hk, hv)); }
        }
        if fails.is_empty() {
            passed += 1;
            println!("{} ({}ms)", "PASS".green().bold(), elapsed);
            if args.verbose {
                println!("    {} {}", "→ status".dimmed(), status);
                let s: String = String::from_utf8_lossy(&body).chars().take(200).collect();
                println!("    {} {}", "→ body".dimmed(), s.dimmed());
            }
        } else {
            failed += 1;
            println!("{} ({}ms)", "FAIL".red().bold(), elapsed);
            for r in &fails { println!("    {} {}", "✗".red(), r); }
            let preview: String = String::from_utf8_lossy(&body).chars().take(500).collect();
            println!("    {} {}", "response:".dimmed(), preview.dimmed());
            if args.fail_fast { break; }
        }
    }
    println!("\n{} {} passed, {} failed", "Summary:".bold(), passed.to_string().green(), failed.to_string().red());
    if failed > 0 { std::process::exit(1); }
    Ok(())
}

struct ApiCase {
    method: String, url: String,
    headers: Vec<(String, String)>, body: Option<String>,
    expect_status: Option<u16>, expect_contains: Vec<String>,
    expect_not_contains: Vec<String>, expect_headers: Vec<(String, String)>,
}

fn parse_spec(content: &str) -> Result<Vec<ApiCase>> {
    let mut tests: Vec<ApiCase> = Vec::new();
    let mut cur: Option<ApiCase> = None;
    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') { continue; }
        if line.starts_with("===") {
            if let Some(c) = cur.take() { tests.push(c); }
            cur = Some(ApiCase {
                method: String::new(), url: String::new(),
                headers: Vec::new(), body: None,
                expect_status: None, expect_contains: Vec::new(),
                expect_not_contains: Vec::new(), expect_headers: Vec::new(),
            });
            continue;
        }
        let case = cur.as_mut().ok_or_else(|| anyhow::anyhow!("Line {}: content before ===", idx + 1))?;
        if case.method.is_empty() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 { anyhow::bail!("Line {}: expected 'METHOD URL'", idx + 1); }
            case.method = parts[0].to_uppercase();
            case.url = parts[1].to_string();
            continue;
        }
        let l = line.trim_start();
        if let Some(v) = l.strip_prefix("expect:") { case.expect_status = Some(v.trim().parse().with_context(|| format!("bad status line {}", idx + 1))?); }
        else if let Some(v) = l.strip_prefix("contains:") { case.expect_contains.push(v.trim().to_string()); }
        else if let Some(v) = l.strip_prefix("not-contains:") { case.expect_not_contains.push(v.trim().to_string()); }
        else if let Some(v) = l.strip_prefix("header:") {
            let parts: Vec<&str> = v.splitn(2, ':').collect();
            if parts.len() == 2 { case.expect_headers.push((parts[0].trim().to_string(), parts[1].trim().to_string())); }
        } else if let Some(v) = l.strip_prefix("body:") { case.body = Some(v.trim().to_string()); }
        else if let Some(col) = l.find(':') {
            let (k, v) = l.split_at(col);
            case.headers.push((k.trim().to_string(), v[1..].trim().to_string()));
        }
    }
    if let Some(c) = cur.take() { tests.push(c); }
    Ok(tests)
}
