use anyhow::Result;
use clap::Args;
use colored::*;
use std::net::ToSocketAddrs;

#[derive(Args)]
pub struct DnsArgs {
    host: String,

    #[arg(short = 'p', long, default_value = "80")]
    port: u16,
}

pub fn run(args: DnsArgs) -> Result<()> {
    let addr = format!("{}:{}", args.host, args.port);
    let start = std::time::Instant::now();
    let addrs = addr.to_socket_addrs().map_err(|e| anyhow::anyhow!("DNS lookup failed: {}", e))?;
    let elapsed = start.elapsed().as_millis();
    let list: Vec<_> = addrs.collect();
    if list.is_empty() { anyhow::bail!("No addresses returned for {}", args.host); }
    println!("{} {} ({}ms)", "Resolved".cyan().bold(), args.host.yellow(), elapsed.to_string().dimmed());
    for a in list {
        let kind = if a.is_ipv4() { "IPv4".green() } else { "IPv6".magenta() };
        println!("  {} {}", kind, a.ip().to_string().cyan());
    }
    Ok(())
}
