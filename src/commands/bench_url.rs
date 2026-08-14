use anyhow::Result;
use clap::Args;
use colored::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::engine::http::{build_client, status_color};

#[derive(Args)]
pub struct BenchUrlArgs {
    url: String,

    /// Total requests to send
    #[arg(short = 'n', long, default_value = "100")]
    count: usize,

    /// Concurrency
    #[arg(short = 'c', long, default_value = "10")]
    concurrency: usize,

    /// Method
    #[arg(short = 'X', long, default_value = "GET")]
    method: String,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    /// Warmup requests (not counted)
    #[arg(long, default_value = "5")]
    warmup: usize,
}

pub fn run(args: BenchUrlArgs) -> Result<()> {
    let client = Arc::new(build_client(args.timeout, true, None)?);
    let method = reqwest::Method::from_bytes(args.method.as_bytes())?;

    // Warmup
    if args.warmup > 0 {
        println!("{} {} warmup requests", "Warmup:".dimmed(), args.warmup.to_string().dimmed());
        for _ in 0..args.warmup {
            let _ = client.request(method.clone(), &args.url).send();
        }
    }

    println!("{} {} requests, concurrency {}", "Benchmarking:".cyan().bold(),
        args.count.to_string().yellow(), args.concurrency.to_string().yellow());

    let (perm_tx, perm_rx) = std::sync::mpsc::sync_channel(args.concurrency);
    for _ in 0..args.concurrency { perm_tx.send(()).ok(); }
    let perm_rx = Arc::new(Mutex::new(perm_rx));

    let times: Arc<Mutex<Vec<u128>>> = Arc::new(Mutex::new(Vec::with_capacity(args.count)));
    let statuses: Arc<Mutex<std::collections::HashMap<u16, usize>>> = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let errors: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let overall = std::time::Instant::now();

    let mut handles = Vec::new();
    for _ in 0..args.count {
        let client = Arc::clone(&client);
        let perm_rx = Arc::clone(&perm_rx);
        let perm_tx = perm_tx.clone();
        let times = Arc::clone(&times);
        let statuses = Arc::clone(&statuses);
        let errors = Arc::clone(&errors);
        let url = args.url.clone();
        let method = method.clone();

        let h = thread::spawn(move || {
            let _permit = perm_rx.lock().unwrap().recv().ok();
            let start = std::time::Instant::now();
            match client.request(method, &url).send() {
                Ok(r) => {
                    let ms = start.elapsed().as_millis();
                    times.lock().unwrap().push(ms);
                    *statuses.lock().unwrap().entry(r.status().as_u16()).or_insert(0) += 1;
                }
                Err(_) => {
                    *errors.lock().unwrap() += 1;
                }
            }
            perm_tx.send(()).ok();
        });
        handles.push(h);
    }
    for h in handles { h.join().ok(); }
    let total_ms = overall.elapsed().as_millis();

    let mut t = times.lock().unwrap().clone();
    t.sort();
    let errs = *errors.lock().unwrap();
    let done = t.len();
    let rps = if total_ms > 0 { (done as f64) * 1000.0 / (total_ms as f64) } else { 0.0 };

    println!("\n{}", "Results:".bold());
    println!("  URL:         {}", args.url.cyan());
    println!("  Total time:  {} ms", total_ms.to_string().yellow());
    println!("  Requests:    {} done, {} errors", done.to_string().green(), errs.to_string().red());
    println!("  Throughput:  {:.1} req/s", rps);

    if !t.is_empty() {
        let min = t.first().unwrap();
        let max = t.last().unwrap();
        let avg = t.iter().sum::<u128>() / t.len() as u128;
        let p50 = t[t.len() * 50 / 100];
        let p95 = t[t.len() * 95 / 100];
        let p99 = t[t.len() * 99 / 100];
        println!("  Latency ms:");
        println!("    min:  {}", min.to_string().green());
        println!("    avg:  {}", avg.to_string().green());
        println!("    p50:  {}", p50.to_string().green());
        println!("    p95:  {}", p95.to_string().yellow());
        println!("    p99:  {}", p99.to_string().yellow());
        println!("    max:  {}", max.to_string().red());
    }
    println!("\n{}", "Status codes:".bold());
    let mut s: Vec<(u16, usize)> = statuses.lock().unwrap().iter().map(|(k, v)| (*k, *v)).collect();
    s.sort();
    for (code, count) in s {
        let color = status_color(code);
        println!("  {} × {}", code.to_string().color(color).bold(), count.to_string().yellow());
    }
    Ok(())
}
