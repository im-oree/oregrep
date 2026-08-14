use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::commands::apply_patch::{run as run_apply, ApplyPatchArgs};

#[derive(Args)]
pub struct RevertPatchArgs {
    patch: PathBuf,

    #[arg(short = 'p', long)]
    path: Option<PathBuf>,

    #[arg(long)]
    no_backup: bool,

    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: RevertPatchArgs) -> Result<()> {
    let inner = ApplyPatchArgs {
        patch: args.patch,
        path: args.path,
        no_backup: args.no_backup,
        label: args.label,
        reverse: true,
        check: false,
    };
    run_apply(inner)
}
