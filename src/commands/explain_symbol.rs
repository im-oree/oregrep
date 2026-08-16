use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

/// Full understanding of a symbol in one command:
/// - Definition + signature
/// - All direct callers across the codebase
/// - Call site context
///
/// Equivalent to: state --at <sym> + who-calls <sym> in one output.
#[derive(Args)]
pub struct ExplainSymbolArgs {
    /// Symbol name
    symbol: String,

    /// File where the symbol is defined
    file: PathBuf,

    /// Path to search for callers (default: parent dir of file)
    #[arg(long)]
    callers_path: Option<PathBuf>,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Context lines around each caller (default: 2)
    #[arg(short = 'C', long, default_value = "2")]
    context: usize,

    /// Max callers to show per file
    #[arg(long, default_value = "3")]
    per_file: usize,
}

pub fn run(args: ExplainSymbolArgs) -> Result<()> {
    println!("{}", "═══════════════════════════════════".cyan().bold());
    println!("{}", format!("  EXPLAIN: {}", args.symbol).cyan().bold());
    println!("{}", "═══════════════════════════════════".cyan().bold());
    println!();

    // Phase 1: Definition — reuse `state --at`
    println!("{}", "── DEFINITION ──".yellow().bold());
    let state_args = crate::commands::state::StateArgs {
        file: args.file.clone(),
        lines: false,
        compact: false,
        json: false,
        at: Some(args.symbol.clone()),
        context: 0,
    };
    let _ = crate::commands::state::run(state_args);

    // Phase 2: Callers — reuse `who-calls`
    println!();
    println!("{}", "── CALLERS ──".yellow().bold());
    let callers_path = args.callers_path.clone().unwrap_or_else(|| {
        args.file.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    });

    let who_args = crate::commands::who_calls::WhoCallsArgs {
        symbol: args.symbol.clone(),
        path: callers_path,
        ext: args.ext,
        exclude: args.exclude,
        context: args.context,
        ignore_case: false,
        external_only: true, // don't count internal recursive calls
        files_only: false,
        per_file: args.per_file,
    };
    let _ = crate::commands::who_calls::run(who_args);

    Ok(())
}
