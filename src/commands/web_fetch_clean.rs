use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::time::Duration;

use crate::engine::ai::config::load;

#[derive(Args)]
pub struct WebFetchCleanArgs {
    url: String,

    /// Max chars to keep (default from config)
    #[arg(short = 'm', long)]
    max_chars: Option<usize>,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: WebFetchCleanArgs) -> Result<()> {
    let cfg = load()?;
    let max = args.max_chars.unwrap_or(cfg.search_fetch_max_chars);

    let rt = crate::engine::ai::runtime::build_runtime()?;
    let text = rt.block_on(async {
        let c = reqwest::Client::builder()
            .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(cfg.search_timeout_secs.max(10)))
            .build()?;
        crate::engine::ai::search::fetch_clean(&c, &args.url, max).await
    });

    let text = text?;
    match args.output {
        Some(p) => {
            std::fs::write(&p, &text)?;
            eprintln!("{} {}  ({} chars)",
                "Wrote:".green().bold(),
                p.display().to_string().cyan(),
                text.chars().count().to_string().yellow());
        }
        None => print!("{}", text),
    }
    Ok(())
}
