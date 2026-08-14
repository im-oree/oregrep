use anyhow::Result;
use clap::Args;
use colored::*;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use url::Url;

use crate::engine::http::{build_client, read_body_bytes, status_color};

#[derive(Args)]
pub struct CrawlArgs {
    /// Starting URL
    url: String,

    /// Max pages to fetch
    #[arg(short = 'n', long, default_value = "50")]
    max: usize,

    /// Max link-follow depth
    #[arg(short = 'd', long, default_value = "2")]
    depth: usize,

    /// Only follow links on the SAME domain as start URL
    #[arg(long, default_value = "true")]
    same_domain: bool,

    /// Save fetched pages here (one file per URL)
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,

    #[arg(short = 't', long, default_value = "20")]
    timeout: u64,

    /// Verbose (show link extraction per page)
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: CrawlArgs) -> Result<()> {
    let start_url = Url::parse(&args.url)?;
    let start_domain = start_url.host_str().unwrap_or("").to_string();

    let client = build_client(args.timeout, true, None)?;
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((args.url.clone(), 0));

    if let Some(d) = &args.output_dir {
        std::fs::create_dir_all(d)?;
    }

    let selector = Selector::parse("a[href]").map_err(|e| anyhow::anyhow!("selector: {}", e))?;
    let mut fetched = 0usize;

    while let Some((current, depth)) = queue.pop_front() {
        if fetched >= args.max { break; }
        if visited.contains(&current) { continue; }
        visited.insert(current.clone());

        let start = std::time::Instant::now();
        let resp = match client.get(&current).send() {
            Ok(r) => r,
            Err(e) => {
                println!("  {} {} {}", "ERR".red(), current.cyan(), e.to_string().dimmed());
                continue;
            }
        };
        let status = resp.status().as_u16();
        let ctype = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let final_url = resp.url().clone();
        let body = read_body_bytes(resp).unwrap_or_default();
        let ms = start.elapsed().as_millis();
        fetched += 1;

        let color = status_color(status);
        println!("  [d={}] {} {}  ({} bytes, {}ms)",
            depth.to_string().dimmed(),
            format!("{}", status).color(color).bold(),
            current.cyan(),
            body.len().to_string().yellow(),
            ms.to_string().dimmed()
        );

        if let Some(dir) = &args.output_dir {
            let name = url_to_filename(&current);
            let _ = std::fs::write(dir.join(name), &body);
        }

        if !ctype.contains("html") || depth >= args.depth { continue; }
        let text = String::from_utf8_lossy(&body);
        let doc = Html::parse_document(&text);
        let mut added = 0usize;
        for el in doc.select(&selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(resolved) = final_url.join(href) {
                    if args.same_domain && resolved.host_str() != Some(&start_domain) { continue; }
                    let s = resolved.to_string();
                    if !visited.contains(&s) {
                        queue.push_back((s, depth + 1));
                        added += 1;
                    }
                }
            }
        }
        if args.verbose {
            println!("    {} {} new links queued", "+".dimmed(), added.to_string().dimmed());
        }
    }

    println!("\n{} {} pages fetched, {} URLs visited, {} still queued",
        "Summary:".bold(),
        fetched.to_string().green(),
        visited.len().to_string().yellow(),
        queue.len().to_string().dimmed()
    );
    Ok(())
}

fn url_to_filename(url: &str) -> String {
    let sanitized: String = url.chars().map(|c| {
        if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' }
    }).collect();
    let cut: String = sanitized.chars().take(120).collect();
    format!("{}.html", cut)
}
