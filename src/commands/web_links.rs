use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::BTreeSet;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebLinksArgs {
    url: String,

    /// Only include links matching this substring
    #[arg(short = 'f', long)]
    filter: Option<String>,

    /// Only same-domain links
    #[arg(short = 's', long)]
    same_domain: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,

    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: WebLinksArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let obj = tab.evaluate(
        "JSON.stringify(Array.from(document.querySelectorAll('a[href]')).map(a => ({ href: a.href, text: (a.innerText||'').trim().slice(0,120) })))",
        true
    )?;
    let raw = obj.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap_or_default();

    let base_domain = url::Url::parse(&args.url).ok().and_then(|u| u.host_str().map(|s| s.to_string()));

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    for item in &parsed {
        let href = item.get("href").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if href.is_empty() { continue; }
        if let Some(f) = &args.filter { if !href.contains(f) { continue; } }
        if args.same_domain {
            if let Some(base) = &base_domain {
                let host = url::Url::parse(&href).ok().and_then(|u| u.host_str().map(|s| s.to_string())).unwrap_or_default();
                if &host != base { continue; }
            }
        }
        if seen.insert(href.clone()) { rows.push((href, text)); }
    }

    if args.json {
        let arr: Vec<_> = rows.iter().map(|(h, t)| serde_json::json!({ "href": h, "text": t })).collect();
        let text = serde_json::to_string_pretty(&arr)?;
        match args.output {
            Some(p) => { std::fs::write(&p, &text)?; eprintln!("Wrote: {}", p.display()); }
            None => println!("{}", text),
        }
        return Ok(());
    }

    let mut out = String::new();
    for (href, text) in &rows {
        out.push_str(&format!("{}", href));
        if !text.is_empty() { out.push_str(&format!("  # {}", text)); }
        out.push('\n');
    }
    match args.output {
        Some(p) => { std::fs::write(&p, &out)?; eprintln!("Wrote: {}", p.display()); }
        None => print!("{}", out),
    }
    eprintln!("\n{} {} links", "Total:".bold(), rows.len().to_string().yellow());
    Ok(())
}
