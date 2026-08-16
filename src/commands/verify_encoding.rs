use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct VerifyEncodingArgs {
    files: Vec<PathBuf>,

    /// Also flag BOM presence as warning
    #[arg(short = 'b', long)]
    strict_bom: bool,
}

pub fn run(args: VerifyEncodingArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("At least one file required"); }
    let mut ok = 0usize;
    let mut fail = 0usize;
    let mut warn = 0usize;
    for f in &args.files {
        if !f.exists() { println!("  {} {}", "MISSING".red(), f.display()); fail += 1; continue; }
        let bytes = std::fs::read(f)?;
        let has_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
        let body = if has_bom { &bytes[3..] } else { &bytes[..] };
        match std::str::from_utf8(body) {
            Ok(_) => {
                if has_bom && args.strict_bom {
                    warn += 1;
                    println!("  {} {} (UTF-8 with BOM)", "WARN".yellow().bold(), f.display().to_string().cyan());
                } else {
                    ok += 1;
                    let tag = if has_bom { " (with BOM)".dimmed().to_string() } else { String::new() };
                    println!("  {} {}{}", "OK".green().bold(), f.display().to_string().cyan(), tag);
                }
            }
            Err(e) => {
                fail += 1;
                println!("  {} {}  invalid at byte {}: {}", "FAIL".red().bold(), f.display().to_string().cyan(), e.valid_up_to(), e);
            }
        }
    }
    println!("\n{} {} ok, {} warnings, {} invalid",
        "Summary:".bold(),
        ok.to_string().green(),
        warn.to_string().yellow(),
        fail.to_string().red());
    if fail > 0 { std::process::exit(1); }
    Ok(())
}
