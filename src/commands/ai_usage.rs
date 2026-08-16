use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::usage::{summary_by_model, total_cost};

#[derive(Args)]
pub struct AiUsageArgs {
    /// Filter to last N days (0 = all-time)
    #[arg(short = 'd', long, default_value = "0")]
    days: i64,
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AiUsageArgs) -> Result<()> {
    let days = if args.days > 0 { Some(args.days) } else { None };
    let rows = summary_by_model(days)?;
    let total = total_cost(days)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("{}", "(no usage recorded)".dimmed());
        return Ok(());
    }
    println!("{}", "AI usage:".cyan().bold());
    for r in &rows {
        println!("  {:<12} {:<40}  {} calls  {}↑ {}↓  ${:.4}",
            r.provider.cyan(),
            r.model.yellow(),
            r.calls.to_string().dimmed(),
            r.tokens_in.to_string().dimmed(),
            r.tokens_out.to_string().dimmed(),
            r.cost_usd);
    }
    println!("\n{} ${:.4}", "Total:".bold(), total);
    Ok(())
}
