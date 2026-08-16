use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Shorthand: diff a file against one of its backups.
/// Equivalent to: diff <file> --backup [--label LABEL]
#[derive(Args)]
pub struct DiffBackupArgs {
    /// File to compare against its backup
    file: PathBuf,

    /// Specific backup label (default: latest backup)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Show line numbers
    #[arg(short = 'n', long, default_value = "true")]
    number: bool,

    /// Context lines
    #[arg(short = 'C', long, default_value = "3")]
    context: usize,

    /// Stats only
    #[arg(short = 's', long)]
    stats: bool,
}

pub fn run(args: DiffBackupArgs) -> Result<()> {
    let diff_args = crate::commands::diff::DiffArgs {
        file_a: args.file,
        file_b: None,
        backup: true,
        label: args.label,
        number: args.number,
        context: args.context,
        stats: args.stats,
    };
    crate::commands::diff::run(diff_args)
}
