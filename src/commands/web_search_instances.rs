use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::{Duration, Instant};

use crate::engine::ai::config::load;

#[derive(Args)]
pub struct WebSearchInstancesArgs {
    /// Show latencies (probes each instance with a 3s HEAD)
    #[arg(short = 't', long, default_value = "true")]
    test: bool,
}

pub fn run(args: WebSearchInstancesArgs) -> Result<()> {
    let cfg = load()?;
    let mut instances: Vec<String> = vec![cfg.search_searxng_url.trim().trim_end_matches('/').to_string()];
    for e in cfg.search_fallback_instances.split(',') {
        let s = e.trim().trim_end_matches('/').to_string();
        if !s.is_empty() && !instances.contains(&s) { instances.push(s); }
    }

    println!("{} {} instances", "Search instances:".cyan().bold(), instances.len().to_string().yellow());
    if !args.test {
        for (i, url) in instances.iter().enumerate() {
            println!("  {}. {}", i + 1, url.cyan());
        }
        return Ok(());
    }

    let rt = crate::engine::ai::runtime::build_runtime()?;
    let results: Vec<(String, Option<Duration>, Option<String>)> = rt.block_on(async {
        let c = reqwest::Client::builder()
            .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(3))
            .build().unwrap();
        let mut out = Vec::new();
        for url in instances.iter() {
            let start = Instant::now();
            let full = format!("{}/search?q=test&format=json", url);
            match c.get(&full).send().await {
                Ok(r) if r.status().is_success() => out.push((url.clone(), Some(start.elapsed()), None)),
                Ok(r) => out.push((url.clone(), Some(start.elapsed()), Some(format!("HTTP {}", r.status())))),
                Err(e) => out.push((url.clone(), None, Some(short(&e.to_string())))),
            }
        }
        out
    });

    for (i, (url, latency, err)) in results.iter().enumerate() {
        let idx = format!("{}.", i + 1);
        match (latency, err) {
            (Some(d), None) => println!("  {} {}  {}ms", idx.dimmed(), url.cyan(), d.as_millis().to_string().green()),
            (Some(d), Some(msg)) => println!("  {} {}  {}ms  {}", idx.dimmed(), url.cyan(), d.as_millis().to_string().yellow(), msg.red()),
            (None, Some(msg)) => println!("  {} {}  {}", idx.dimmed(), url.cyan(), msg.red()),
            _ => println!("  {} {}  ?", idx.dimmed(), url.cyan()),
        }
    }
    Ok(())
}

fn short(s: &str) -> String {
    if s.len() > 100 { format!("{}…", &s[..100]) } else { s.to_string() }
}
