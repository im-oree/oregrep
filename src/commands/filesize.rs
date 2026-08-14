use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::http::{build_client, fmt_bytes};

#[derive(Args)]
pub struct FilesizeArgs {
    /// One or more URLs
    urls: Vec<String>,

    #[arg(short = 't', long, default_value = "10")]
    timeout: u64,

    /// Raw bytes only, one per line (for scripts)
    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: FilesizeArgs) -> Result<()> {
    if args.urls.is_empty() { anyhow::bail!("Provide at least one URL"); }
    let client = build_client(args.timeout, true, None)?;
    let mut total: u64 = 0;
    for url in &args.urls {
        let size = match client.head(url).send() {
            Ok(r) => r.content_length(),
            Err(_) => None,
        };
        match size {
            Some(s) => {
                total += s;
                if args.quiet { println!("{}", s); }
                else {
                    println!("{}  {}  {}",
                        fmt_bytes(s).green().bold(),
                        format!("({} bytes)", s).dimmed(),
                        url.cyan()
                    );
                }
            }
            None => {
                if args.quiet { println!("-1"); }
                else { println!("{} unknown  {}", "?".yellow(), url.cyan()); }
            }
        }
    }
    if args.urls.len() > 1 && !args.quiet {
        println!("\n{} {} ({} bytes across {} URLs)",
            "Total:".bold(),
            fmt_bytes(total).yellow(),
            total.to_string().dimmed(),
            args.urls.len().to_string().yellow()
        );
    }
    Ok(())
}
