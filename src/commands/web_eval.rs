use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebEvalArgs {
    url: String,
    /// JS expression to evaluate. Return value is printed.
    expression: String,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: WebEvalArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let result = tab.evaluate(&args.expression, true)?;
    let value = result.value;
    if args.json {
        let out = serde_json::to_string_pretty(&value)?;
        println!("{}", out);
    } else {
        match value {
            Some(serde_json::Value::String(s)) => println!("{}", s),
            Some(v) => println!("{}", v),
            None => println!("{}", "(no result)".dimmed()),
        }
    }
    Ok(())
}
