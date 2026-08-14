use anyhow::Result;
use clap::Args;
use colored::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Args)]
pub struct PingArgs {
    host: String,

    #[arg(short = 'p', long, default_value = "80")]
    port: u16,

    #[arg(short = 'n', long, default_value = "4")]
    count: usize,

    #[arg(short = 't', long, default_value = "2")]
    timeout: f64,

    #[arg(short = 'i', long, default_value = "1.0")]
    interval: f64,
}

pub fn run(args: PingArgs) -> Result<()> {
    println!("{} {}:{}", "TCP ping".cyan().bold(), args.host.yellow(), args.port.to_string().yellow());
    let mut successes = 0usize;
    let mut times: Vec<u128> = Vec::new();

    for i in 1..=args.count {
        let addr_str = format!("{}:{}", args.host, args.port);
        let start = Instant::now();
        let result = match addr_str.to_socket_addrs() {
            Ok(mut it) => match it.next() {
                Some(sa) => TcpStream::connect_timeout(&sa, Duration::from_secs_f64(args.timeout)),
                None => Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "no address")),
            },
            Err(e) => Err(e),
        };
        let elapsed = start.elapsed().as_millis();
        match result {
            Ok(_) => {
                successes += 1;
                times.push(elapsed);
                println!("  {} [{}] {} in {}ms", "OK".green(), i.to_string().dimmed(), addr_str.cyan(), elapsed.to_string().yellow());
            }
            Err(e) => {
                println!("  {} [{}] {}: {}", "FAIL".red(), i.to_string().dimmed(), addr_str.cyan(), e.to_string().dimmed());
            }
        }
        if i < args.count { std::thread::sleep(Duration::from_secs_f64(args.interval)); }
    }
    let loss_pct = 100.0 - (successes as f64 / args.count as f64) * 100.0;
    println!("\n{}", "Summary:".bold());
    println!("  Sent: {}, Success: {}, Loss: {:.1}%",
        args.count.to_string().yellow(), successes.to_string().green(), loss_pct);
    if !times.is_empty() {
        let min = *times.iter().min().unwrap();
        let max = *times.iter().max().unwrap();
        let avg = times.iter().sum::<u128>() / times.len() as u128;
        println!("  RTT min/avg/max: {}/{}/{} ms", min.to_string().green(), avg.to_string().green(), max.to_string().green());
    }
    if successes == 0 { std::process::exit(1); }
    Ok(())
}
