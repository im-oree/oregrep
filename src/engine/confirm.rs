use anyhow::Result;
use colored::*;
use std::io::{self, Write};

/// Prompt the user for yes/no confirmation.
/// Returns true if confirmed (yes or all).
/// If `yes_flag` is true, auto-confirms without prompting.
pub fn confirm(prompt: &str, yes_flag: bool) -> Result<bool> {
    if yes_flag {
        return Ok(true);
    }
    print!("{} {} ", prompt.yellow().bold(), "[y/N]".dimmed());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

/// Prompt with y/n/all/quit options. Returns:
///   Ok(Some(true))  = yes (this one)
///   Ok(Some(false)) = no (this one)
///   Ok(None)        = "all" (yes to all remaining)
///   Err on quit
#[allow(dead_code)]
pub fn confirm_each(prompt: &str, yes_flag: bool) -> Result<Option<bool>> {
    if yes_flag {
        return Ok(None); // treat -y as "all"
    }
    print!("{} {} ", prompt.yellow().bold(), "[y/N/a/q]".dimmed());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    match answer.as_str() {
        "y" | "yes" => Ok(Some(true)),
        "a" | "all" => Ok(None),
        "q" | "quit" => anyhow::bail!("Aborted by user"),
        _ => Ok(Some(false)),
    }
}
