use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{changed_files, ensure_repo, FileFilter};

#[derive(Args)]
pub struct GitChangedArgs {
    /// Only files matching (substring)
    #[arg(long)]
    only: Option<String>,

    /// Exclude files matching
    #[arg(long)]
    except: Option<String>,

    /// Only files whose basename starts with
    #[arg(long)]
    starts: Option<String>,

    /// Only files in this subdirectory
    #[arg(long)]
    changed_in: Option<String>,

    /// Only files whose content contains this substring
    #[arg(long)]
    matching: Option<String>,

    /// Include only staged
    #[arg(long)]
    staged: bool,

    /// Include only unstaged
    #[arg(long)]
    unstaged: bool,

    /// Include only untracked
    #[arg(long)]
    untracked: bool,

    /// Print paths only, no color/decoration (for piping)
    #[arg(short = 'p', long)]
    paths_only: bool,
}

pub fn run(args: GitChangedArgs) -> Result<()> {
    ensure_repo()?;
    let files = changed_files()?;

    // Filter by state
    let filtered: Vec<(String, String)> = files.into_iter().filter(|(status, _)| {
        let staged_char = status.chars().next().unwrap_or(' ');
        let unstaged_char = status.chars().nth(1).unwrap_or(' ');
        let is_untracked = status.starts_with('?');
        let is_staged = staged_char != ' ' && !is_untracked;
        let is_unstaged = unstaged_char != ' ' && !is_untracked;

        // If any state flag set, apply. Else include all.
        if args.staged || args.unstaged || args.untracked {
            (args.staged && is_staged) || (args.unstaged && is_unstaged) || (args.untracked && is_untracked)
        } else {
            true
        }
    }).collect();

    let filter = FileFilter {
        only: args.only,
        except: args.except,
        starts: args.starts,
        matching: args.matching,
        changed_in: args.changed_in,
    };

    let paths: Vec<String> = filtered.iter().map(|(_, p)| p.clone()).collect();
    let kept = filter.apply(paths);

    for p in &kept {
        if args.paths_only {
            println!("{}", p);
        } else {
            println!("{}", p.cyan());
        }
    }
    if !args.paths_only {
        eprintln!("\n{} files", kept.len().to_string().yellow());
    }
    Ok(())
}
