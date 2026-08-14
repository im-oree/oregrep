use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::engine::encoding::read_file_smart;
use crate::engine::http::{apply_headers, build_client, filename_from_url, fmt_bytes, parse_headers_from_flags, read_body_bytes};

#[derive(Args)]
pub struct DownloadManyArgs {
    urls: Vec<String>,

    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    #[arg(short = 'o', long, default_value = ".")]
    output_dir: PathBuf,

    #[arg(long)]
    force: bool,

    #[arg(short = 'l', long, default_value = "4")]
    limit: usize,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "300")]
    timeout: u64,

    #[arg(short = 'r', long, default_value = "0")]
    rate: f64,
}

pub fn run(args: DownloadManyArgs) -> Result<()> {
    let mut urls: Vec<String> = args.urls.clone();
    if let Some(f) = &args.file {
        let content = read_file_smart(f)?;
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') { urls.push(l.to_string()); }
        }
    }
    if urls.is_empty() { anyhow::bail!("Provide URLs or --file"); }
    std::fs::create_dir_all(&args.output_dir)?;

    let client = Arc::new(build_client(args.timeout, true, None)?);
    let hdrs = Arc::new(parse_headers_from_flags(&args.headers)?);
    let output_dir = Arc::new(args.output_dir.clone());
    let force = args.force;

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

    let (perm_tx, perm_rx) = std::sync::mpsc::sync_channel(args.limit);
    for _ in 0..args.limit { perm_tx.send(()).ok(); }
    let perm_rx = Arc::new(Mutex::new(perm_rx));

    let results: Arc<Mutex<(usize, usize, u64)>> = Arc::new(Mutex::new((0, 0, 0)));
    let mut handles = Vec::new();

    for url in urls.iter().cloned() {
        let client = Arc::clone(&client);
        let hdrs = Arc::clone(&hdrs);
        let perm_rx = Arc::clone(&perm_rx);
        let ticket_rx = Arc::clone(&ticket_rx);
        let perm_tx = perm_tx.clone();
        let output_dir = Arc::clone(&output_dir);
        let results = Arc::clone(&results);

        let h = thread::spawn(move || {
            let _permit = perm_rx.lock().unwrap().recv().ok();
            let _ = ticket_rx.lock().unwrap().recv();

            let fname = match filename_from_url(&url) {
                Ok(f) => f,
                Err(_) => PathBuf::from("download"),
            };
            let target = output_dir.join(&fname);
            if target.exists() && !force {
                println!("  {} {}  ({})", "SKIP".yellow(), url.cyan(), "exists".dimmed());
                perm_tx.send(()).ok();
                return;
            }
            let start = Instant::now();
            let req = apply_headers(client.get(&url), &hdrs);
            match req.send() {
                Ok(r) => {
                    if !r.status().is_success() {
                        println!("  {} {} HTTP {}", "FAIL".red(), url.cyan(), r.status().as_u16());
                        results.lock().unwrap().1 += 1;
                        perm_tx.send(()).ok();
                        return;
                    }
                    let bytes = match read_body_bytes(r) { Ok(b) => b, Err(_) => { perm_tx.send(()).ok(); return; } };
                    if std::fs::write(&target, &bytes).is_err() {
                        println!("  {} {} write failed", "FAIL".red(), url.cyan());
                        results.lock().unwrap().1 += 1;
                        perm_tx.send(()).ok();
                        return;
                    }
                    let ms = start.elapsed().as_millis();
                    let mut r = results.lock().unwrap();
                    r.0 += 1;
                    r.2 += bytes.len() as u64;
                    drop(r);
                    println!("  {} {} → {}  ({}, {}ms)",
                        "OK".green(), url.cyan(), target.display().to_string().yellow(),
                        fmt_bytes(bytes.len() as u64).dimmed(), ms.to_string().dimmed());
                }
                Err(e) => {
                    println!("  {} {}: {}", "FAIL".red(), url.cyan(), e.to_string().dimmed());
                    results.lock().unwrap().1 += 1;
                }
            }
            perm_tx.send(()).ok();
        });
        handles.push(h);
    }
    for h in handles { h.join().ok(); }
    dispatcher.join().ok();

    let (ok, failed, total) = *results.lock().unwrap();
    println!("\n{} {} downloaded, {} failed, {} total",
        "Summary:".bold(),
        ok.to_string().green(),
        failed.to_string().red(),
        fmt_bytes(total).yellow()
    );
    Ok(())
}
