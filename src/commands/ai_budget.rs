use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::usage::budget_status;

#[derive(Args)]
pub struct AiBudgetArgs {
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AiBudgetArgs) -> Result<()> {
    let cfg = load_cfg()?;
    let status = budget_status(&cfg)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    let remaining = if status.session_budget_usd > 0.0 {
        (status.session_budget_usd - status.process_spend_usd).max(0.0)
    } else {
        f64::INFINITY
    };

    println!("{}", "AI budget:".cyan().bold());
    println!("  {:<24} ${:.5}", "process_spend_usd".cyan(), status.process_spend_usd);
    println!("  {:<24} {}", "session_budget_usd".cyan(),
        if status.session_budget_usd > 0.0 { format!("${:.5}", status.session_budget_usd).yellow().to_string() } else { "unlimited".dimmed().to_string() });
    println!("  {:<24} {}", "remaining_usd".cyan(),
        if remaining.is_finite() { format!("${:.5}", remaining).green().to_string() } else { "unlimited".dimmed().to_string() });
    println!("  {:<24} {}", "call_budget_usd".cyan(),
        if status.call_budget_usd > 0.0 { format!("${:.5}", status.call_budget_usd).yellow().to_string() } else { "unlimited".dimmed().to_string() });
    println!("  {:<24} ${:.5}", "historical_total_usd".cyan(), status.historical_total_usd);

    if status.session_budget_usd > 0.0 && status.process_spend_usd >= status.session_budget_usd {
        println!("\n{}", "session budget exceeded for this process".red().bold());
    }

    Ok(())
}
