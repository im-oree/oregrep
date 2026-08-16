use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::usage::query_history;

#[derive(Args)]
pub struct AiHistoryArgs {
    /// Max entries to show (default 50)
    #[arg(short = 'n', long, default_value = "50")]
    limit: i64,

    /// Filter by task label (ask, explain, review, fix, refactor, agent, chat, commit-message)
    #[arg(short = 't', long)]
    task: Option<String>,

    /// Only show entries from today
    #[arg(long)]
    today: bool,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AiHistoryArgs) -> Result<()> {
    let since: Option<i64> = if args.today {
        let now = chrono::Local::now();
        let midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        Some(midnight.and_utc().timestamp())
    } else {
        None
    };

    let rows = query_history(args.limit, args.task.as_deref(), since)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("{}", "(no history)".dimmed());
        return Ok(());
    }

    println!("{}", "AI history:".cyan().bold());
    println!("{:<20} {:<16} {:<42} {:<8} {:<6} {:<6} {}",
        "time".dimmed(),
        "task".dimmed(),
        "model".dimmed(),
        "cost".dimmed(),
        "in".dimmed(),
        "out".dimmed(),
        "ms".dimmed());
    println!("{}", "─".repeat(110).dimmed());

    for r in &rows {
        let ts = chrono::DateTime::from_timestamp(r.ts, 0)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| r.ts.to_string());
        let model_str = format!("{}:{}", r.provider, r.model);
        let model_short: String = model_str.chars().take(40).collect();
        let task = r.task.as_deref().unwrap_or("-");
        println!("{:<20} {:<16} {:<42} ${:<7.5} {:<6} {:<6} {}",
            ts.dimmed(),
            task.yellow(),
            model_short.cyan(),
            r.cost_usd,
            r.tokens_in.to_string().dimmed(),
            r.tokens_out.to_string().dimmed(),
            r.duration_ms.to_string().dimmed());
    }
    println!("\n{} {} entries", "Total:".bold(), rows.len());
    Ok(())
}
