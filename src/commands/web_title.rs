use anyhow::Result;
use clap::Args;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebTitleArgs {
    url: String,
    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: WebTitleArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let title = tab.get_title().unwrap_or_default();
    println!("{}", title);
    Ok(())
}
