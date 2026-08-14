use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::commands::replace_project::{run as run_project, ReplaceProjectArgs};

#[derive(Args)]
pub struct ReplaceDirArgs {
    /// Regex pattern
    pattern: String,

    /// Replacement
    replacement: String,

    /// Directory to search within
    dir: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'F', long)]
    literal: bool,
    #[arg(short = 'i', long)]
    ignore_case: bool,
    #[arg(short = 'w', long)]
    word: bool,
    #[arg(short = 'm', long)]
    multiline: bool,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,
    #[arg(long)]
    binary: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(long)]
    keep_going: bool,
}

pub fn run(args: ReplaceDirArgs) -> Result<()> {
    let inner = ReplaceProjectArgs {
        pattern: args.pattern,
        replacement: args.replacement,
        path: args.dir,
        ext: args.ext,
        exclude: args.exclude,
        literal: args.literal,
        ignore_case: args.ignore_case,
        word: args.word,
        multiline: args.multiline,
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        binary: args.binary,
        dry_run: args.dry_run,
        no_backup: args.no_backup,
        label: args.label,
        keep_going: args.keep_going,
    };
    run_project(inner)
}
