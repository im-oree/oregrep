use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::session::search_messages;

#[derive(Args)]
pub struct AiRecallArgs {
    /// Search term (substring match across all session messages)
    query: String,

    /// Max results
    #[arg(short = 'n', long, default_value = "20")]
    limit: i64,

    /// Filter to a specific session
    #[arg(short = 's', long)]
    session: Option<String>,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AiRecallArgs) -> Result<()> {
    let results = search_messages(&args.query, args.limit, args.session.as_deref())?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("{}", "(no matches)".dimmed());
        return Ok(());
    }

    println!("{} {} matches for '{}'",
        "ai-recall:".cyan().bold(),
        results.len(),
        args.query.yellow());
    println!("{}", "─".repeat(80).dimmed());

    for r in &results {
        let ts = chrono::DateTime::from_timestamp(r.ts, 0)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| r.ts.to_string());

        let role_colored = match r.role.as_str() {
            "user" => r.role.magenta(),
            "assistant" => r.role.cyan(),
            _ => r.role.dimmed(),
        };

        // Show snippet around the match
        let snippet = make_snippet(&r.content, &args.query, 200);

        println!("[{}] {} {} {}",
            ts.dimmed(),
            r.session.yellow(),
            role_colored,
            "─".dimmed());
        println!("  {}", snippet.dimmed());
        println!();
    }
    Ok(())
}

fn make_snippet(content: &str, query: &str, max: usize) -> String {
    let lower = content.to_lowercase();
    let q_lower = query.to_lowercase();
    if let Some(pos) = lower.find(&q_lower) {
        let start = pos.saturating_sub(60);
        let end = (pos + query.len() + 140).min(content.len());
        let mut s = String::new();
        if start > 0 { s.push_str("…"); }
        s.push_str(&content[start..end].replace('\n', " "));
        if end < content.len() { s.push_str("…"); }
        s
    } else {
        let t: String = content.chars().take(max).collect();
        if content.len() > max { format!("{}…", t) } else { t }
    }
}
