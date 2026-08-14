mod commands;
mod engine;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ore",
    version,
    about = "Powerful all-in-one file, code, and codebase manipulation CLI",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for a pattern in files (recursive, gitignore-aware)
    Find(commands::find::FindArgs),
    /// Print a file with smart encoding detection
    Cat(commands::cat::CatArgs),
    /// Print a specific line or line range from a file
    Line(commands::line::LineArgs),
    /// Print a directory tree
    Tree(commands::tree::TreeArgs),
    /// Create a backup of a file (max 3 per file, oldest deleted)
    Backup(commands::backup::BackupArgs),
    /// Restore a file from backup
    Restore(commands::restore::RestoreArgs),
    /// Apply a find/replace patch to a file (atomic, encoding-safe)
    Patch(commands::patch::PatchArgs),
}

fn main() -> Result<()> {
    #[cfg(windows)]
    {
        let _ = enable_ansi_support();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Find(args) => commands::find::run(args)?,
        Commands::Cat(args) => commands::cat::run(args)?,
        Commands::Line(args) => commands::line::run(args)?,
        Commands::Tree(args) => commands::tree::run(args)?,
        Commands::Backup(args) => commands::backup::run(args)?,
        Commands::Restore(args) => commands::restore::run(args)?,
        Commands::Patch(args) => commands::patch::run(args)?,
    }

    Ok(())
}

#[cfg(windows)]
fn enable_ansi_support() -> Result<()> {
    use std::os::windows::io::AsRawHandle;
    let handle = std::io::stdout().as_raw_handle();
    unsafe {
        let handle_ptr = handle as *mut std::ffi::c_void;
        let mut mode: u32 = 0;
        extern "system" {
            fn GetConsoleMode(handle: *mut std::ffi::c_void, mode: *mut u32) -> i32;
            fn SetConsoleMode(handle: *mut std::ffi::c_void, mode: u32) -> i32;
        }
        if GetConsoleMode(handle_ptr, &mut mode) != 0 {
            SetConsoleMode(handle_ptr, mode | 0x0004);
        }
    }
    Ok(())
}
