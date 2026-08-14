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
    Find(commands::find::FindArgs),
    Cat(commands::cat::CatArgs),
    Line(commands::line::LineArgs),
    Tree(commands::tree::TreeArgs),
    Backup(commands::backup::BackupArgs),
    Restore(commands::restore::RestoreArgs),
    Patch(commands::patch::PatchArgs),
    Replace(commands::replace::ReplaceArgs),
    Diff(commands::diff::DiffArgs),
    Encoding(commands::encoding::EncodingArgs),
    Newlines(commands::newlines::NewlinesArgs),
    Insert(commands::insert::InsertArgs),
    DeleteLines(commands::delete_lines::DeleteLinesArgs),
    ReplaceLine(commands::replace_line::ReplaceLineArgs),
    ReplaceRange(commands::replace_range::ReplaceRangeArgs),
    Before(commands::before::BeforeArgs),
    After(commands::after::AfterArgs),
    Surround(commands::surround::SurroundArgs),
    ReplaceProject(commands::replace_project::ReplaceProjectArgs),
    ReplaceExt(commands::replace_ext::ReplaceExtArgs),
    ReplaceDir(commands::replace_dir::ReplaceDirArgs),
    PatchProject(commands::patch_project::PatchProjectArgs),
    RenameBulk(commands::rename_bulk::RenameBulkArgs),
    Head(commands::head::HeadArgs),
    Tail(commands::tail::TailArgs),
    Count(commands::count::CountArgs),
    Stats(commands::stats::StatsArgs),
    Wc(commands::wc::WcArgs),
    DedupLines(commands::dedup_lines::DedupLinesArgs),
    SortLines(commands::sort_lines::SortLinesArgs),
    Trim(commands::trim::TrimArgs),
    StripBlankLines(commands::strip_blank_lines::StripBlankLinesArgs),
    CollapseBlankLines(commands::collapse_blank_lines::CollapseBlankLinesArgs),
    PurgeBackups(commands::purge_backups::PurgeBackupsArgs),
    /// Move/rename a file or directory (auto-backup on overwrite)
    Mv(commands::mv::MvArgs),
    /// Copy a file or directory (auto-backup on overwrite)
    Cp(commands::cp::CpArgs),
    /// Delete files/directories with confirmation and backup
    Rm(commands::rm::RmArgs),
    /// Create empty file(s) or update mtime
    Touch(commands::touch::TouchArgs),
    /// Create directory (recursive)
    Mkdir(commands::mkdir::MkdirArgs),
    /// Create a file with optional initial content
    Mkfile(commands::mkfile::MkfileArgs),
    /// Compute file checksum (sha256/md5/crc32/all)
    Checksum(commands::checksum::ChecksumArgs),
    /// Find duplicate files by content hash
    FindDupes(commands::find_dupes::FindDupesArgs),
    /// Verify a file against an expected checksum
    VerifyChecksum(commands::verify_checksum::VerifyChecksumArgs),
    /// Extract line ranges from one or more files (multi-file, multi-range, labels)
    Extract(commands::extract::ExtractArgs),
    /// Pack files into an AI-ready blob (md/xml/tag/plain, with tree, strip, truncate)
    Pack(commands::pack::PackArgs),
    /// Slice content between pattern markers (start/end regex)
    Slice(commands::slice::SliceArgs),
    /// Codebase map: per-file lines/size/exports/imports overview
    Map(commands::map::MapArgs),
    /// Git working tree status
    GitStatus(commands::git_status::GitStatusArgs),
    /// List changed files (with filters)
    GitChanged(commands::git_changed::GitChangedArgs),
    /// Show git diff (staged/unstaged, per-file, or commit)
    GitDiff(commands::git_diff::GitDiffArgs),
    /// Commit history for a file
    GitHistory(commands::git_history::GitHistoryArgs),
    /// Git blame with range support
    GitBlame(commands::git_blame::GitBlameArgs),
    /// Search git history by content or commit message
    GitSearch(commands::git_search::GitSearchArgs),
    /// Contributors to a file (ranked by commits)
    GitWho(commands::git_who::GitWhoArgs),
    /// Stage files with filters (--only/--except/--starts/--changed-in)
    GitStage(commands::git_stage::GitStageArgs),
    /// Commit files with filters (--only/--except/--starts/--changed-in)
    GitCommit(commands::git_commit::GitCommitArgs),
    /// Git log with filters (--mine, --author, --grep, --since, --until)
    GitLog(commands::git_log::GitLogArgs),
}

fn main() -> Result<()> {
    #[cfg(windows)]
    { let _ = enable_ansi_support(); }

    let cli = Cli::parse();

    match cli.command {
        Commands::Find(a) => commands::find::run(a)?,
        Commands::Cat(a) => commands::cat::run(a)?,
        Commands::Line(a) => commands::line::run(a)?,
        Commands::Tree(a) => commands::tree::run(a)?,
        Commands::Backup(a) => commands::backup::run(a)?,
        Commands::Restore(a) => commands::restore::run(a)?,
        Commands::Patch(a) => commands::patch::run(a)?,
        Commands::Replace(a) => commands::replace::run(a)?,
        Commands::Diff(a) => commands::diff::run(a)?,
        Commands::Encoding(a) => commands::encoding::run(a)?,
        Commands::Newlines(a) => commands::newlines::run(a)?,
        Commands::Insert(a) => commands::insert::run(a)?,
        Commands::DeleteLines(a) => commands::delete_lines::run(a)?,
        Commands::ReplaceLine(a) => commands::replace_line::run(a)?,
        Commands::ReplaceRange(a) => commands::replace_range::run(a)?,
        Commands::Before(a) => commands::before::run(a)?,
        Commands::After(a) => commands::after::run(a)?,
        Commands::Surround(a) => commands::surround::run(a)?,
        Commands::ReplaceProject(a) => commands::replace_project::run(a)?,
        Commands::ReplaceExt(a) => commands::replace_ext::run(a)?,
        Commands::ReplaceDir(a) => commands::replace_dir::run(a)?,
        Commands::PatchProject(a) => commands::patch_project::run(a)?,
        Commands::RenameBulk(a) => commands::rename_bulk::run(a)?,
        Commands::Head(a) => commands::head::run(a)?,
        Commands::Tail(a) => commands::tail::run(a)?,
        Commands::Count(a) => commands::count::run(a)?,
        Commands::Stats(a) => commands::stats::run(a)?,
        Commands::Wc(a) => commands::wc::run(a)?,
        Commands::DedupLines(a) => commands::dedup_lines::run(a)?,
        Commands::SortLines(a) => commands::sort_lines::run(a)?,
        Commands::Trim(a) => commands::trim::run(a)?,
        Commands::StripBlankLines(a) => commands::strip_blank_lines::run(a)?,
        Commands::CollapseBlankLines(a) => commands::collapse_blank_lines::run(a)?,
        Commands::PurgeBackups(a) => commands::purge_backups::run(a)?,
        Commands::Mv(a) => commands::mv::run(a)?,
        Commands::Cp(a) => commands::cp::run(a)?,
        Commands::Rm(a) => commands::rm::run(a)?,
        Commands::Touch(a) => commands::touch::run(a)?,
        Commands::Mkdir(a) => commands::mkdir::run(a)?,
        Commands::Mkfile(a) => commands::mkfile::run(a)?,
        Commands::Checksum(a) => commands::checksum::run(a)?,
        Commands::FindDupes(a) => commands::find_dupes::run(a)?,
        Commands::VerifyChecksum(a) => commands::verify_checksum::run(a)?,
        Commands::Extract(a) => commands::extract::run(a)?,
        Commands::Pack(a) => commands::pack::run(a)?,
        Commands::Slice(a) => commands::slice::run(a)?,
        Commands::Map(a) => commands::map::run(a)?,
        Commands::GitStatus(a) => commands::git_status::run(a)?,
        Commands::GitChanged(a) => commands::git_changed::run(a)?,
        Commands::GitDiff(a) => commands::git_diff::run(a)?,
        Commands::GitHistory(a) => commands::git_history::run(a)?,
        Commands::GitBlame(a) => commands::git_blame::run(a)?,
        Commands::GitSearch(a) => commands::git_search::run(a)?,
        Commands::GitWho(a) => commands::git_who::run(a)?,
        Commands::GitStage(a) => commands::git_stage::run(a)?,
        Commands::GitCommit(a) => commands::git_commit::run(a)?,
        Commands::GitLog(a) => commands::git_log::run(a)?,
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
