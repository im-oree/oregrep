use anyhow::Result;
use clap::Args;

use crate::engine::commit_msg::{analyze_diff, compose_message, detect_convention};
use crate::engine::git::ensure_repo;

#[derive(Args)]
pub struct GitAutoMessageArgs {
    /// Analyze staged changes (default: unstaged/working tree vs HEAD)
    #[arg(short = 's', long)]
    staged: bool,

    /// Force conventional-commits style
    #[arg(long)]
    conventional: bool,

    /// Force simple English style
    #[arg(long, conflicts_with = "conventional")]
    simple: bool,

    /// Skip body, subject line only
    #[arg(short = 'S', long)]
    subject_only: bool,
}

pub fn run(args: GitAutoMessageArgs) -> Result<()> {
    ensure_repo()?;
    let a = analyze_diff(args.staged)?;
    if a.files.is_empty() {
        eprintln!("No changes to describe.");
        return Ok(());
    }
    let style = if args.conventional { "conventional".to_string() }
        else if args.simple { "simple".to_string() }
        else { detect_convention() };
    let msg = compose_message(&a, &style, !args.subject_only);
    println!("{}", msg);
    Ok(())
}
