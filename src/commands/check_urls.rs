use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::engine::encoding::read_file_smart;
use crate::engine::http::{build_client, status_color};

#[derive(Args)]
pub struct CheckUrlsArgs {
    urls: Vec<String>,

    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    #[arg(short = 'l', long, default_value = "10")]
    limit: usize,

    #[arg(short = 't', long, default_value = "10")]
    timeout: u64,

    /// Also try GET on failures (some servers block HEAD)
    #[arg(long)]
    fallback_get: bool,

    /// Only show non-OK results
    #[arg(short = 'F', long)]
    failures_only: bool,
}

pub fn run(args: CheckUrlsArgs) -> Result<()> {
    let mut urls: Vec<String> = args.urls.clone();
    if let Some(f) = &args.file {
        let content = read_file_smart(f)?;
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') { urls.push(l.to_string()); }
        }
    }
    if urls.is_empty() { anyhow::bail!("Provide URLs or --file"); }

    let client = Arc::new(build_client(args.timeout, true, None)?);
    let (perm_tx, perm_rx) = std::sync::mpsc::sync_channel(args.limit);
    for _ in 0..args.limit { perm_tx.send(()).ok(); }
    let perm_rx = Arc::new(Mutex::new(perm_rx));
    let counts: Arc<Mutex<(usize, usize, usize, usize)>> = Arc::new(Mutex::new((0, 0, 0, 0))); // ok, redir, client, server

    let mut handles = Vec::new();
    for url in urls.iter().cloned() {
        let client = Arc::clone(&client);
        let perm_rx = Arc::clone(&perm_rx);
        let perm_tx = perm_tx.clone();
        let counts = Arc::clone(&counts);
        let fallback_get = args.fallback_get;
        let failures_only = args.failures_only;

        let h = thread::spawn(move || {
            let _permit = perm_rx.lock().unwrap().recv().ok();
            let start = std::time::Instant::now();
            let mut resp = client.head(&url).send();
            if resp.is_err() && fallback_get {
                resp = client.get(&url).send();
            }
            let ms = start.elapsed().as_millis();
            match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let final_url = r.url().to_string();
                    let redirected = final_url != url;
                    match status {
                        200..=299 => counts.lock().unwrap().0 += 1,
                        300..=399 => counts.lock().unwrap().1 += 1,
                        400..=499 => counts.lock().unwrap().2 += 1,
                        500..=599 => counts.lock().unwrap().3 += 1,
                        _ => {}
                    }
                    let is_ok = (200..400).contains(&status);
                    if failures_only && is_ok { perm_tx.send(()).ok(); return; }
                    let color = status_color(status);
                    let redir_note = if redirected { format!(" → {}", final_url).dimmed().to_string() } else { String::new() };
                    println!("  {} {} {}  ({}ms){}",
                        status.to_string().color(color).bold(),
                        "•".dimmed(), url.cyan(),
                        ms.to_string().dimmed(), redir_note);
                }
                Err(e) => {
                    counts.lock().unwrap().3 += 1;
                    println!("  {} {} {}: {}", "ERR".red().bold(), "•".dimmed(), url.cyan(), e.to_string().dimmed());
                }
            }
            perm_tx.send(()).ok();
        });
        handles.push(h);
    }
    for h in handles { h.join().ok(); }

    let (ok, r, c, s) = *counts.lock().unwrap();
    println!("\n{} 2xx: {}, 3xx: {}, 4xx: {}, 5xx/err: {}",
        "Summary:".bold(),
        ok.to_string().green(),
        r.to_string().cyan(),
        c.to_string().yellow(),
        s.to_string().red()
    );
    if c > 0 || s > 0 { std::process::exit(1); }
    Ok(())
}
