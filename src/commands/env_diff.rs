use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct EnvDiffArgs {
    file_a: PathBuf,
    file_b: PathBuf,
    /// Only show differing keys (skip identical)
    #[arg(short = 'D', long, default_value = "true")]
    only_diff: bool,
}

pub fn run(args: EnvDiffArgs) -> Result<()> {
    let a = load(&args.file_a)?;
    let b = load(&args.file_b)?;
    let mut all_keys: Vec<String> = a.keys().cloned().collect();
    for k in b.keys() { if !all_keys.contains(k) { all_keys.push(k.clone()); } }
    all_keys.sort();
    let mut differ = 0usize;
    for k in &all_keys {
        let av = a.get(k);
        let bv = b.get(k);
        if av == bv { if !args.only_diff { println!("  {} = {}", k.cyan(), av.cloned().unwrap_or_default().dimmed()); } continue; }
        differ += 1;
        match (av, bv) {
            (Some(x), Some(y)) => {
                println!("  {} {}", "~".yellow(), k.cyan());
                println!("    {} {}", "A:".red(), x);
                println!("    {} {}", "B:".green(), y);
            }
            (Some(x), None) => println!("  {} {} = {}   (only in A)", "-".red(), k.cyan(), x.dimmed()),
            (None, Some(y)) => println!("  {} {} = {}   (only in B)", "+".green(), k.cyan(), y.dimmed()),
            _ => {}
        }
    }
    println!("\n{} {} keys differ", "Summary:".bold(), differ.to_string().yellow());
    if differ > 0 { std::process::exit(1); }
    Ok(())
}

fn load(p: &std::path::Path) -> Result<std::collections::BTreeMap<String, String>> {
    let iter = dotenvy::from_path_iter(p)?;
    let mut m = std::collections::BTreeMap::new();
    for r in iter { let (k, v) = r?; m.insert(k, v); }
    Ok(m)
}
