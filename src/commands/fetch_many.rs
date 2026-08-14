use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::engine::encoding::read_file_smart;
use crate::engine::http::{apply_headers, build_client, parse_headers_from_flags, read_body_bytes, status_color};

#[derive(Args)]
pub struct FetchManyArgs {
    urls: Vec<String>,

    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    #[arg(short = 'l', long, default_value = "5")]
    limit: usize,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,

    #[arg(short = 'v', long)]
    verbose: bool,

    /// Rate limit (requests per second, 0 = no limit)
    #[arg(short = 'r', long, default_value = "0")]
    rate: f64,
}

pub fn run(args: FetchManyArgs) -> Result<()> {
    let mut urls: Vec<String> = args.urls.clone();
    if let Some(f) = &args.file {
        let content = read_file_smart(f)?;
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') { urls.push(l.to_string()); }
        }
    }
    if urls.is_empty() { anyhow::bail!("Provide URLs or --file"); }

    if let Some(d) = &args.output_dir { std::fs::create_dir_all(d)?; }

    let hdrs = Arc::new(parse_headers_from_flags(&args.headers)?);
    let client = Arc::new(build_client(args.timeout, true, None)?);
    let output_dir = args.output_dir.clone();
    let _verbose = args.verbose;

    // Rate limiter: main thread doles out "tickets" at rate; workers pull tickets
    let (ticket_tx, ticket_rx) = std::sync::mpsc::channel::<()>();
    let ticket_rx = Arc::new(Mutex::new(ticket_rx));
    let rate_delay = if args.rate > 0.0 { Some(Duration::from_secs_f64(1.0 / args.rate)) } else { None };
    let n_urls = urls.len();
    let dispatcher = thread::spawn(move || {
        for i in 0..n_urls {
            if i > 0 {
                if let Some(d) = rate_delay { thread::sleep(d); }
            }
            if ticket_tx.send(()).is_err() { break; }
        }
    });

    // Concurrency limiter
    let (perm_tx, perm_rx) = std::sync::mpsc::sync_channel(args.limit);
    for _ in 0..args.limit { perm_tx.send(()).ok(); }
    let perm_rx = Arc::new(Mutex::new(perm_rx));

    let results: Arc<Mutex<Vec<(String, u16, usize, u128)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for url in urls.iter().cloned() {
        let client = Arc::clone(&client);
        let hdrs = Arc::clone(&hdrs);
        let perm_rx = Arc::clone(&perm_rx);
        let ticket_rx = Arc::clone(&ticket_rx);
        let perm_tx = perm_tx.clone();
        let output_dir = output_dir.clone();
        let results = Arc::clone(&results);

        let h = thread::spawn(move || {
            let _permit = perm_rx.lock().unwrap().recv().ok();
            // Wait for a rate-limit ticket
            let _ = ticket_rx.lock().unwrap().recv();

            let start = Instant::now();
            let req = apply_headers(client.get(&url), &hdrs);
            let (status, bytes) = match req.send() {
                Ok(r) => {
                    let s = r.status().as_u16();
                    let body = read_body_bytes(r).unwrap_or_default();
                    if let Some(dir) = &output_dir {
                        if let Ok(fname) = crate::engine::http::filename_from_url(&url) {
                            let target = dir.join(fname);
                            let _ = std::fs::write(&target, &body);
                        }
                    }
                    (s, body.len())
                }
                Err(_) => (0, 0),
            };
            let ms = start.elapsed().as_millis();
            let color = status_color(status);
            let label = if status == 0 { "ERR".red().bold().to_string() } else { format!("{}", status).color(color).bold().to_string() };
            println!("  {} {} {}  ({} bytes, {}ms)",
                label, "→".dimmed(), url.cyan(),
                bytes.to_string().yellow(), ms.to_string().dimmed());
            results.lock().unwrap().push((url.clone(), status, bytes, ms));
            perm_tx.send(()).ok();
        });
        handles.push(h);
    }
    for h in handles { h.join().ok(); }
    dispatcher.join().ok();

    let rs = results.lock().unwrap().clone();
    let ok = rs.iter().filter(|(_, s, _, _)| *s >= 200 && *s < 400).count();
    let failed = rs.len() - ok;
    let total_bytes: usize = rs.iter().map(|(_, _, b, _)| *b).sum();
    println!("\n{} {} ok, {} failed, {} bytes total",
        "Summary:".bold(),
        ok.to_string().green(),
        failed.to_string().red(),
        crate::engine::http::fmt_bytes(total_bytes as u64).yellow()
    );
    Ok(())
}
