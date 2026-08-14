use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;

#[derive(Args)]
pub struct Merge3Args {
    /// Base version (common ancestor)
    base: PathBuf,

    /// Our version
    ours: PathBuf,

    /// Their version
    theirs: PathBuf,

    /// Output file
    #[arg(short = 'o', long)]
    output: PathBuf,

    /// Auto-resolve conflicts by taking OURS
    #[arg(long)]
    prefer_ours: bool,

    /// Auto-resolve conflicts by taking THEIRS
    #[arg(long, conflicts_with = "prefer_ours")]
    prefer_theirs: bool,

    /// Auto-resolve conflicts by unioning both sides
    #[arg(long, conflicts_with_all = ["prefer_ours", "prefer_theirs"])]
    union: bool,

    /// Just show conflict count, don't write
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: Merge3Args) -> Result<()> {
    let base = read_file_smart(&args.base)?;
    let ours = read_file_smart(&args.ours)?;
    let theirs = read_file_smart(&args.theirs)?;

    let base_lines: Vec<&str> = base.lines().collect();
    let ours_lines: Vec<&str> = ours.lines().collect();
    let theirs_lines: Vec<&str> = theirs.lines().collect();

    // Very simple 3-way: diff base→ours and base→theirs by line, then merge
    // For a genuine merge we'd use similar's algorithms; keeping it simple + explicit
    let mut result: Vec<String> = Vec::new();
    let mut conflicts = 0usize;

    let max_len = base_lines.len().max(ours_lines.len()).max(theirs_lines.len());
    for i in 0..max_len {
        let b = base_lines.get(i).copied();
        let o = ours_lines.get(i).copied();
        let t = theirs_lines.get(i).copied();
        match (b, o, t) {
            (Some(_), Some(o), Some(t)) if o == t => { result.push(o.to_string()); }
            (Some(b), Some(o), Some(t)) if o == b && t != b => { result.push(t.to_string()); }
            (Some(b), Some(o), Some(t)) if t == b && o != b => { result.push(o.to_string()); }
            (Some(_), Some(o), Some(t)) => {
                conflicts += 1;
                if args.prefer_ours { result.push(o.to_string()); }
                else if args.prefer_theirs { result.push(t.to_string()); }
                else if args.union {
                    result.push(o.to_string());
                    result.push(t.to_string());
                } else {
                    result.push("<<<<<<< OURS".to_string());
                    result.push(o.to_string());
                    result.push("=======".to_string());
                    result.push(t.to_string());
                    result.push(">>>>>>> THEIRS".to_string());
                }
            }
            (None, Some(o), Some(t)) if o == t => result.push(o.to_string()),
            (None, Some(o), None) => result.push(o.to_string()),
            (None, None, Some(t)) => result.push(t.to_string()),
            (None, Some(o), Some(t)) => {
                conflicts += 1;
                if args.prefer_ours { result.push(o.to_string()); }
                else if args.prefer_theirs { result.push(t.to_string()); }
                else if args.union {
                    result.push(o.to_string());
                    result.push(t.to_string());
                } else {
                    result.push("<<<<<<< OURS".to_string());
                    result.push(o.to_string());
                    result.push("=======".to_string());
                    result.push(t.to_string());
                    result.push(">>>>>>> THEIRS".to_string());
                }
            }
            (Some(_), Some(o), None) => result.push(o.to_string()),
            (Some(_), None, Some(t)) => result.push(t.to_string()),
            _ => {}
        }
    }

    let joined = result.join("\n");

    println!("{} {} conflict{}",
        "Merged:".cyan().bold(),
        conflicts.to_string().yellow(),
        if conflicts == 1 { "" } else { "s" }
    );

    if args.dry_run {
        println!("{}", "[DRY RUN — nothing written]".yellow().bold());
        return Ok(());
    }

    write_atomic(&args.output, &joined, false)?;
    println!("{} {}", "Wrote:".green().bold(), args.output.display().to_string().cyan());
    if conflicts > 0 && !args.prefer_ours && !args.prefer_theirs && !args.union {
        println!("{} conflict markers embedded in output; resolve manually.", "!".yellow());
    }
    Ok(())
}
