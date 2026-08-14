use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::http::{build_client, status_color};

#[derive(Args)]
pub struct StatusArgs {
    url: String,

    #[arg(short = 't', long, default_value = "10")]
    timeout: u64,

    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let client = build_client(args.timeout, true, None)?;
    let start = std::time::Instant::now();
    let resp = client.head(&args.url).send()?;
    let elapsed = start.elapsed().as_millis();
    if args.quiet {
        println!("{}", resp.status().as_u16());
    } else {
        let color = status_color(resp.status().as_u16());
        println!("{} {}  {}  ({}ms)",
            resp.status().as_u16().to_string().color(color).bold(),
            resp.status().canonical_reason().unwrap_or("").dimmed(),
            resp.url().to_string().cyan(),
            elapsed.to_string().dimmed()
        );
    }
    Ok(())
}
