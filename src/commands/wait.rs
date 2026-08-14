use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct WaitArgs {
    /// Wait for file to exist
    #[arg(long)]
    file: Option<PathBuf>,

    /// Wait for file to be MISSING
    #[arg(long)]
    file_missing: Option<PathBuf>,

    /// Wait for file to change (mtime)
    #[arg(long)]
    file_changed: Option<PathBuf>,

    /// Wait for TCP port to be open (localhost:PORT)
    #[arg(long)]
    port: Option<u16>,

    /// Wait for port to be closed
    #[arg(long)]
    port_closed: Option<u16>,

    /// Wait N seconds
    #[arg(long)]
    time: Option<f64>,

    /// Run this command repeatedly until it succeeds (exit 0)
    #[arg(long)]
    command: Option<String>,

    /// Run this command until its stdout contains this text
    #[arg(long)]
    output_contains: Option<String>,

    /// Wait for HTTP URL to return 200 (uses curl)
    #[arg(long)]
    url: Option<String>,

    /// Expected HTTP status (default 200)
    #[arg(long, default_value = "200")]
    status: u16,

    /// Polling interval in seconds
    #[arg(short = 'i', long, default_value = "0.5")]
    interval: f64,

    /// Timeout in seconds (0 = no timeout)
    #[arg(short = 't', long, default_value = "0")]
    timeout: f64,

    /// Suppress polling output
    #[arg(short = 'q', long)]
    silent: bool,
}

pub fn run(args: WaitArgs) -> Result<()> {
    // Time wait is trivial
    if let Some(secs) = args.time {
        if !args.silent {
            println!("{} {}s", "Sleeping".cyan(), secs);
        }
        std::thread::sleep(Duration::from_secs_f64(secs));
        return Ok(());
    }

    let interval = Duration::from_secs_f64(args.interval.max(0.05));
    let deadline = if args.timeout > 0.0 {
        Some(Instant::now() + Duration::from_secs_f64(args.timeout))
    } else {
        None
    };

    let mut attempts = 0usize;
    loop {
        attempts += 1;
        let done = check(&args)?;
        if done {
            if !args.silent {
                println!("{} after {} attempts", "Ready".green().bold(), attempts.to_string().yellow());
            }
            return Ok(());
        }
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                anyhow::bail!("Wait timed out after {}s ({} attempts)", args.timeout, attempts);
            }
        }
        if !args.silent && attempts % 10 == 0 {
            eprintln!("{} still waiting... ({} attempts)", "…".dimmed(), attempts.to_string().dimmed());
        }
        std::thread::sleep(interval);
    }
}

fn check(args: &WaitArgs) -> Result<bool> {
    if let Some(p) = &args.file {
        return Ok(p.exists());
    }
    if let Some(p) = &args.file_missing {
        return Ok(!p.exists());
    }
    if let Some(p) = &args.file_changed {
        // Compare current mtime to baseline (stored in static via env var trick)
        static BASELINE: std::sync::OnceLock<std::sync::Mutex<Option<std::time::SystemTime>>> = std::sync::OnceLock::new();
        let cell = BASELINE.get_or_init(|| std::sync::Mutex::new(None));
        let mut base = cell.lock().unwrap();
        let current = std::fs::metadata(p).ok().and_then(|m| m.modified().ok());
        if base.is_none() {
            *base = current;
            return Ok(false);
        }
        return Ok(current != *base);
    }
    if let Some(port) = args.port {
        return Ok(is_port_open("127.0.0.1", port));
    }
    if let Some(port) = args.port_closed {
        return Ok(!is_port_open("127.0.0.1", port));
    }
    if let Some(cmd) = &args.command {
        let r = run_cmd(cmd, false, true)?;
        if let Some(needle) = &args.output_contains {
            return Ok(r.success() && r.stdout.contains(needle));
        }
        return Ok(r.success());
    }
    if let Some(url) = &args.url {
        let cmd = format!("curl -s -o NUL -w \"%{{http_code}}\" \"{}\"", url);
        let r = run_cmd(&cmd, false, true)?;
        let code: u16 = r.stdout.trim().parse().unwrap_or(0);
        return Ok(code == args.status);
    }
    anyhow::bail!("Provide one of: --file --file-missing --file-changed --port --port-closed --time --command --url");
}

fn is_port_open(host: &str, port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
    let addr = format!("{}:{}", host, port);
    if let Ok(mut iter) = addr.to_socket_addrs() {
        if let Some(sa) = iter.next() {
            let sa: SocketAddr = sa;
            return TcpStream::connect_timeout(&sa, Duration::from_millis(500)).is_ok();
        }
    }
    false
}
