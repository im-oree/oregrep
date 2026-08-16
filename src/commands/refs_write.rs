use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::commands::trace_mutation::{TraceMutationArgs, run as trace_run};

/// Alias for trace-mutation — filter refs to only WRITE sites
#[derive(Args)]
pub struct RefsWriteArgs {
    symbol: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    #[arg(short = 'C', long, default_value = "1")]
    context: usize,

    #[arg(short = 'i', long)]
    ignore_case: bool,

    #[arg(short = 'l', long)]
    lines_only: bool,
}

pub fn run(args: RefsWriteArgs) -> Result<()> {
    trace_run(TraceMutationArgs {
        property: args.symbol,
        path: args.path,
        ext: args.ext,
        exclude: args.exclude,
        context: args.context,
        ignore_case: args.ignore_case,
        lines_only: args.lines_only,
    })
}
