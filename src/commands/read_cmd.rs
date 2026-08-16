use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Read a file — smart mode: whole file, function, line range, or symbol.
/// Alias for common read operations. Delegates to state/snippet/cat internally.
#[derive(Args)]
pub struct ReadArgs {
    /// File to read
    file: PathBuf,

    /// Extract only this function/class/type
    #[arg(long, value_name = "NAME")]
    r#fn: Option<String>,

    /// Show only this line range: N, N:M, N-M
    #[arg(long, value_name = "RANGE")]
    range: Option<String>,

    /// Show context around a pattern (uses cat-around)
    #[arg(long, value_name = "PATTERN")]
    around: Option<String>,

    /// Context lines for --around
    #[arg(short = 'C', long, default_value = "5")]
    context: usize,

    /// Show line numbers
    #[arg(short = 'n', long)]
    numbers: bool,
}

pub fn run(args: ReadArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    // Priority: --fn > --range > --around > whole file
    if let Some(name) = args.r#fn {
        // Delegate to state --at
        let state_args = crate::commands::state::StateArgs {
            file: args.file,
            lines: args.numbers,
            compact: false,
            json: false,
            at: Some(name),
            context: 0,
        };
        return crate::commands::state::run(state_args);
    }

    if let Some(range) = args.range {
        // Delegate to line
        let line_args = crate::commands::line::LineArgs {
            file: args.file,
            range,
            no_number: !args.numbers,
            context: 0,
        };
        return crate::commands::line::run(line_args);
    }

    if let Some(pattern) = args.around {
        // Delegate to cat-around
        let ca_args = crate::commands::cat_around::CatAroundArgs {
            file: args.file,
            pattern,
            context: args.context,
            line_numbers: args.numbers,
            ignore_case: false,
            regex: false,
        };
        return crate::commands::cat_around::run(ca_args);
    }

    // Fallback: cat
    let cat_args = crate::commands::cat::CatArgs {
        file: args.file,
        number: args.numbers,
        binary: false,
        grep: None,
        raw: false,
    };
    crate::commands::cat::run(cat_args)
}
