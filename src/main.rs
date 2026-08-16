mod commands;
mod engine;
mod tui;

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
    /// Search multiple patterns at once with per-pattern counts
    FindMulti(commands::find_multi::FindMultiArgs),
    /// Find a pattern only in lines changed in the git diff
    FindInChangedLines(commands::find_in_changed_lines::FindInChangedLinesArgs),
    Cat(commands::cat::CatArgs),
    /// Print lines around every match of a pattern (context viewer)
    CatAround(commands::cat_around::CatAroundArgs),
    Line(commands::line::LineArgs),
    Tree(commands::tree::TreeArgs),
    Backup(commands::backup::BackupArgs),
    Restore(commands::restore::RestoreArgs),
    Patch(commands::patch::PatchArgs),
    /// Apply a .orepatch file with --atomic (all-or-nothing) and --report (pre-flight) support
    PatchBatch(commands::patch_batch::PatchBatchArgs),
    /// Insert text before or after a specific line number
    PatchInsert(commands::patch_insert::PatchInsertArgs),
    /// Replace an exact line or inclusive line range with new text
    PatchLines(commands::patch_lines::PatchLinesArgs),
    /// Preview a patch as a unified diff without writing (exits 0=found, 1=not found)
    PatchPreview(commands::patch_preview::PatchPreviewArgs),
    /// Patch a file using a regular expression with capture group support
    PatchRegex(commands::patch_regex::PatchRegexArgs),
    /// Patch with fuzzy anchor matching (similarity threshold, no exact match needed)
    PatchFuzzy(commands::patch_fuzzy::PatchFuzzyArgs),
    Replace(commands::replace::ReplaceArgs),
    Diff(commands::diff::DiffArgs),
    /// Shorthand for `diff <file> --backup [-l LABEL]` — compare file to its backup
    DiffBackup(commands::diff_backup::DiffBackupArgs),
    Encoding(commands::encoding::EncodingArgs),
    /// Normalize file encoding + line endings (default: UTF-8 LF, no BOM)
    EncodingNormalize(commands::encoding_normalize::EncodingNormalizeArgs),
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
    /// Tag files with labels for session tracking (read, patched, reviewed, etc.)
    Tag(commands::tag::TagArgs),
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
    /// Create a file from clipboard, stdin, or another file
    MkfileFrom(commands::mkfile_from::MkfileFromArgs),
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
    /// Pack all files changed since a git ref (default: HEAD)
    PackChanged(commands::pack_changed::PackChangedArgs),
    /// Pack specific line ranges from multiple files: file:N-M file2:A-B
    PackLines(commands::pack_lines::PackLinesArgs),
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
    /// Blame a single line (file:line) with full commit context
    GitBlameLine(commands::git_blame_line::GitBlameLineArgs),
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
    /// Full history of a specific line (file:line) via git log -L
    GitLogLine(commands::git_log_line::GitLogLineArgs),
    /// Search git history for commits touching a pattern (git log -S pickaxe)
    SearchDiff(commands::search_diff::SearchDiffArgs),
    /// Run a command with capture/stream/silent options
    Run(commands::run::RunArgs),
    /// Wait for a condition (file, port, url, command output, time)
    Wait(commands::wait::WaitArgs),
    /// Retry a command until success (with backoff)
    Retry(commands::retry::RetryArgs),
    /// Run multiple commands in parallel
    Parallel(commands::parallel::ParallelArgs),
    /// Run commands sequentially (stop or continue on fail)
    Sequence(commands::sequence::SequenceArgs),
    /// Watch a path and run a command when it changes
    Watch(commands::watch::WatchArgs),
    /// Run a fallback command if the first fails
    OnError(commands::on_error::OnErrorArgs),
    /// Run a follow-up command if the first succeeds
    OnSuccess(commands::on_success::OnSuccessArgs),
    /// Find files containing ALL given patterns
    SearchAnd(commands::search_and::SearchAndArgs),
    /// Find files containing ANY given patterns
    SearchOr(commands::search_or::SearchOrArgs),
    /// Find files that do NOT contain a pattern (optionally require another)
    SearchNegative(commands::search_negative::SearchNegativeArgs),
    /// Search patterns that span multiple lines
    SearchMultiline(commands::search_multiline::SearchMultilineArgs),
    /// Typo-tolerant fuzzy search (filenames + content)
    SearchFuzzy(commands::search_fuzzy::SearchFuzzyArgs),
    /// Search only in git-changed files (staged/unstaged/untracked filters)
    SearchChanged(commands::search_changed::SearchChangedArgs),
    /// Search across git history (pickaxe / regex)
    SearchHistory(commands::search_history::SearchHistoryArgs),
    /// Word/character-level diff
    DiffWord(commands::diff_word::DiffWordArgs),
    /// Semantic diff (ignores whitespace + comments)
    DiffSemantic(commands::diff_semantic::DiffSemanticArgs),
    /// Diff with configurable ignore flags (whitespace, blank, case, comments)
    DiffIgnore(commands::diff_ignore::DiffIgnoreArgs),
    /// Diff two directory trees
    DiffDirs(commands::diff_dirs::DiffDirsArgs),
    /// Three-way merge (base + ours + theirs)
    Merge3(commands::merge3::Merge3Args),
    /// Apply a .patch/.diff file (via git apply, with backups)
    ApplyPatch(commands::apply_patch::ApplyPatchArgs),
    /// Revert (reverse-apply) a .patch/.diff file
    RevertPatch(commands::revert_patch::RevertPatchArgs),
    /// HTTP GET a URL (with headers, output, pretty JSON)
    Fetch(commands::fetch::FetchArgs),
    /// HTTP POST/PUT/PATCH/DELETE with body from string/file/json
    Post(commands::post::PostArgs),
    /// Download a URL to a file
    Download(commands::download::DownloadArgs),
    /// Show response headers only
    Headers(commands::headers::HeadersArgs),
    /// Show HTTP status code only
    Status(commands::status::StatusArgs),
    /// TCP ping (host:port reachability test)
    Ping(commands::ping::PingArgs),
    /// DNS resolution
    Dns(commands::dns::DnsArgs),
    /// Run API tests from a .ore-api spec file
    ApiTest(commands::api_test::ApiTestArgs),
    /// Get remote file size (one or many URLs) without downloading
    Filesize(commands::filesize::FilesizeArgs),
    /// Multipart file upload (with fields, headers)
    Upload(commands::upload::UploadArgs),
    /// Parallel HTTP GET of many URLs (rate-limit + save)
    FetchMany(commands::fetch_many::FetchManyArgs),
    /// Parallel download of many URLs
    DownloadMany(commands::download_many::DownloadManyArgs),
    /// Bulk URL health checker (2xx/3xx/4xx/5xx)
    CheckUrls(commands::check_urls::CheckUrlsArgs),
    /// Resumable download using HTTP Range
    ResumeDownload(commands::resume_download::ResumeDownloadArgs),
    /// Benchmark a URL (N reqs, concurrency, p50/p95/p99)
    BenchUrl(commands::bench_url::BenchUrlArgs),
    /// WebSocket client (send/receive/listen)
    Ws(commands::ws::WsArgs),
    /// Crawl a URL by following links (bounded depth + count)
    Crawl(commands::crawl::CrawlArgs),
    /// Show content (stdin or file) in an editor (default: notepad). Strips ANSI colors.
    Show(commands::show::ShowArgs),
    /// Copy content (stdin or file) to system clipboard
    Copy(commands::copy::CopyArgs),
    /// Write stdin to a temp file and print its path (chain with ore open)
    ToTemp(commands::to_temp::ToTempArgs),
    /// Open a file or folder in default OS handler or a specified editor
    OpenFile(commands::open_file::OpenFileArgs),
    /// View file as hex+ASCII (paged, with offset/length/width)
    HexView(commands::hex_view::HexViewArgs),
    /// Find hex pattern in binary (with ?? wildcards)
    HexFind(commands::hex_find::HexFindArgs),
    /// Replace hex bytes (same-length in-place)
    HexReplace(commands::hex_replace::HexReplaceArgs),
    /// Write hex bytes at a specific offset (same-length or extend)
    HexPatch(commands::hex_patch::HexPatchArgs),
    /// Binary diff with offset + hex dump
    HexDiff(commands::hex_diff::HexDiffArgs),
    /// Extract a byte range from a file
    HexExtract(commands::hex_extract::HexExtractArgs),
    /// Insert bytes at offset (existing bytes shift)
    HexInsert(commands::hex_insert::HexInsertArgs),
    /// Delete a byte range from a file
    HexDelete(commands::hex_delete::HexDeleteArgs),
    /// Extract printable strings (ASCII + optional UTF-16)
    Strings(commands::strings::StringsArgs),
    /// Identify file type by magic bytes
    Magic(commands::magic::MagicArgs),
    /// Byte frequency + entropy + histogram
    BinStats(commands::bin_stats::BinStatsArgs),
    /// Base64 encode
    Base64Encode(commands::base64_encode::Base64EncodeArgs),
    /// Base64 decode
    Base64Decode(commands::base64_decode::Base64DecodeArgs),
    /// Raw xxd-style hex dump
    Xxd(commands::xxd::XxdArgs),
    /// Extract byte range to a new file
    BinSlice(commands::bin_slice::BinSliceArgs),
    /// Concatenate binary files
    BinCat(commands::bin_cat::BinCatArgs),
    /// Global config (get/set/list persistent settings)
    Config(commands::config_cmd::ConfigArgs),
    /// User-defined command aliases
    Alias(commands::alias::AliasArgs),
    /// Set/show/clear a focus path for the workspace
    Focus(commands::focus::FocusArgs),
    /// Session tracking (start/end/log/notes)
    Session(commands::session::SessionArgs),
    /// Export session handoff document (git status, notes, history, modified files)
    SessionExport(commands::session_export::SessionExportArgs),
    /// Run tsc --noEmit, parse errors, cache them
    CompileTs(commands::compile_ts::CompileTsArgs),
    /// Run cargo check/build, parse errors, cache them
    CompileRust(commands::compile_rust::CompileRustArgs),
    /// Run npm/yarn/pnpm script, cache output
    CompileNode(commands::compile_node::CompileNodeArgs),
    /// Replay the last cached compile errors (grouped/filtered/JSON)
    ErrorsLast(commands::errors_last::ErrorsLastArgs),
    /// Run typecheck + lint + tests in sequence
    Verify(commands::verify::VerifyArgs),
    /// Check that an exact text anchor exists in a file before patching (exits 0=found, 1=not found)
    VerifyAnchor(commands::verify_anchor::VerifyAnchorArgs),
    /// Auto-detect ts/rust/node project and run the appropriate compile check
    VerifyCompile(commands::verify_compile::VerifyCompileArgs),
    /// Apply .orepatch atomically then verify — auto-rollback on failure
    VerifyAndApply(commands::verify_and_apply::VerifyAndApplyArgs),
    /// Show closest matches to a failed anchor (fuzzy find with similarity %)
    ReAnchor(commands::re_anchor::ReAnchorArgs),
    /// Codebase health report (score, todos, code smells, meta files)
    Health(commands::health::HealthArgs),
    /// Validate one or more JSON files
    VerifyJson(commands::verify_json::VerifyJsonArgs),
    /// Basic syntax check (JSON, TOML, brace-balance for code)
    VerifySyntax(commands::verify_syntax::VerifySyntaxArgs),
    /// Validate UTF-8 encoding of one or more files
    VerifyEncoding(commands::verify_encoding::VerifyEncodingArgs),
    /// Verify relative imports resolve (JS/TS)
    VerifyImports(commands::verify_imports::VerifyImportsArgs),
    /// Mark file(s) as locked (registry-only, use with rm/mv guards later)
    Lock(commands::lock::LockArgs),
    /// Unlock file(s)
    Unlock(commands::unlock::UnlockArgs),
    /// List all locked files
    Locks(commands::locks::LocksArgs),
    /// Launch interactive TUI (file tree, preview, search, command palette, git panel)
    Tui(commands::tui::TuiArgs),
    /// JSON: get value by dot/bracket path
    JsonGet(commands::json_get::JsonGetArgs),
    /// JSON: set value by path (creates intermediate objects)
    JsonSet(commands::json_set::JsonSetArgs),
    /// JSON: deep-merge multiple files into base
    JsonMerge(commands::json_merge::JsonMergeArgs),
    /// JSON: format (pretty/compact/sort-keys)
    JsonFmt(commands::json_fmt::JsonFmtArgs),
    /// JSON: JSONPath query (with $.foo.bar[?(@.x>1)] syntax)
    JsonQuery(commands::json_query::JsonQueryArgs),
    /// JSON: list keys (flat or recursive with types)
    JsonKeys(commands::json_keys::JsonKeysArgs),
    /// YAML: get value by path
    YamlGet(commands::yaml_get::YamlGetArgs),
    /// YAML: set value by path
    YamlSet(commands::yaml_set::YamlSetArgs),
    /// YAML: format
    YamlFmt(commands::yaml_fmt::YamlFmtArgs),
    /// YAML: convert to JSON
    YamlToJson(commands::yaml_to_json::YamlToJsonArgs),
    /// TOML: get value by path
    TomlGet(commands::toml_get::TomlGetArgs),
    /// TOML: set value by path
    TomlSet(commands::toml_set::TomlSetArgs),
    /// TOML: format
    TomlFmt(commands::toml_fmt::TomlFmtArgs),
    /// TOML: convert to JSON
    TomlToJson(commands::toml_to_json::TomlToJsonArgs),
    /// CSV: query a column with optional --where filters
    CsvQuery(commands::csv_query::CsvQueryArgs),
    /// CSV: filter rows by column=value
    CsvFilter(commands::csv_filter::CsvFilterArgs),
    /// CSV: select subset of columns
    CsvSelect(commands::csv_select::CsvSelectArgs),
    /// CSV: convert to JSON (array of objects)
    CsvToJson(commands::csv_to_json::CsvToJsonArgs),
    /// CSV: per-column stats (unique count, empties, numeric?)
    CsvStats(commands::csv_stats::CsvStatsArgs),
    /// .env: get value by key (or list all)
    EnvGet(commands::env_get::EnvGetArgs),
    /// .env: set / delete key
    EnvSet(commands::env_set::EnvSetArgs),
    /// .env: diff two files
    EnvDiff(commands::env_diff::EnvDiffArgs),
    /// XML: get element text or attribute value
    XmlGet(commands::xml_get::XmlGetArgs),
    /// XML: reformat with indentation
    XmlFmt(commands::xml_fmt::XmlFmtArgs),
    /// XML: convert to JSON
    XmlToJson(commands::xml_to_json::XmlToJsonArgs),
    /// List all exported/named symbols across a path (regex-based, TS/JS/Rust/Python)
    Symbols(commands::symbols::SymbolsArgs),
    /// Outline one file's structure
    Outline(commands::outline::OutlineArgs),
    /// Extract a function/class/type by name from a file
    Snippet(commands::snippet::SnippetArgs),
    /// Pluck exports/imports/types/hooks/components from a file
    Pluck(commands::pluck::PluckArgs),
    /// Find every reference to a symbol across a path
    Refs(commands::refs::RefsArgs),
    /// List files that import from a given file
    UsedBy(commands::used_by::UsedByArgs),
    /// Show what a file imports (with optional resolution)
    ImportsOf(commands::imports_of::ImportsOfArgs),
    /// Recursive dependency neighborhood around a file (with optional pack)
    Neighbors(commands::neighbors::NeighborsArgs),
    /// Add a named/default import to a file (merges with existing)
    AddImport(commands::add_import::AddImportArgs),
    /// Remove a named import or an entire import line
    RemoveImport(commands::remove_import::RemoveImportArgs),
    /// Split a multi-symbol file into per-symbol files (with optional barrel hub)
    SplitFile(commands::split_file::SplitFileArgs),
    /// Merge multiple files into one (dedup imports, headers per file)
    MergeFiles(commands::merge_files::MergeFilesArgs),
    /// Extract a named function/class into a new file, re-export from source
    ExtractFn(commands::extract_fn::ExtractFnArgs),
    /// Move a file and update every importer's path
    MoveWithImports(commands::move_with_imports::MoveWithImportsArgs),
    /// Create a barrel index.ts (or mod.rs / __init__.py) from a folder
    Hub(commands::hub::HubArgs),
    /// Inline all re-exports of a hub into a single file
    FlattenHub(commands::flatten_hub::FlattenHubArgs),
    /// Rename a symbol across the codebase (word-boundary regex, all files)
    RenameSymbol(commands::rename_symbol::RenameSymbolArgs),
    /// Analyze and optionally reorganize top-level files into folders
    Organize(commands::organize::OrganizeArgs),
    /// Auto-generate + apply commit message from staged diff
    GitAutoCommit(commands::git_auto_commit::GitAutoCommitArgs),
    /// Generate a commit message from diff (don't commit)
    GitAutoMessage(commands::git_auto_message::GitAutoMessageArgs),
    /// Suggest a commit message and explain the rationale
    GitSuggestCommit(commands::git_suggest_commit::GitSuggestCommitArgs),
    /// Compose commit with your subject + generated body
    GitCommitBody(commands::git_commit_body::GitCommitBodyArgs),
    /// Generate CHANGELOG markdown from git history
    GitChangelog(commands::git_changelog::GitChangelogArgs),
    /// Generate release notes for a version
    GitReleaseNotes(commands::git_release_notes::GitReleaseNotesArgs),
    /// Undo last N commits (soft/mixed/hard)
    GitUndoCommit(commands::git_undo_commit::GitUndoCommitArgs),
    /// Amend the last commit
    GitAmend(commands::git_amend::GitAmendArgs),
    /// Create a fixup commit targeting a previous SHA (with optional autosquash)
    GitFixup(commands::git_fixup::GitFixupArgs),
    /// Delete merged/orphaned local branches
    GitCleanupBranches(commands::git_cleanup_branches::GitCleanupBranchesArgs),
    /// Named stash save/list/apply/pop/drop/show
    GitStashNamed(commands::git_stash_named::GitStashNamedArgs),
    /// Scaffold a new project from a template
    Scaffold(commands::scaffold::ScaffoldArgs),
    /// Add a feature (tailwind, zustand, prettier, eslint, etc.) to a project
    ScaffoldAdd(commands::scaffold_add::ScaffoldAddArgs),
    /// Scaffold a React component
    ScaffoldComponent(commands::scaffold_component::ScaffoldComponentArgs),
    /// Scaffold a React hook
    ScaffoldHook(commands::scaffold_hook::ScaffoldHookArgs),
    /// Scaffold a Zustand store
    ScaffoldStore(commands::scaffold_store::ScaffoldStoreArgs),
    /// Scaffold a React context + provider + hook
    ScaffoldContext(commands::scaffold_context::ScaffoldContextArgs),
    /// Scaffold a REST API client module
    ScaffoldApi(commands::scaffold_api::ScaffoldApiArgs),
    /// Scaffold a test file for an existing source file
    ScaffoldTest(commands::scaffold_test::ScaffoldTestArgs),
    /// Verify a toolchain is installed (rust/node/git/python/env)
    Setup(commands::setup::SetupArgs),
    /// Check that a set of tools are available on PATH
    CheckDeps(commands::check_deps::CheckDepsArgs),
    /// Install missing tools via winget/choco/npm/cargo/scoop
    InstallIfMissing(commands::install_if_missing::InstallIfMissingArgs),
    /// Snippet manager (save/load/list/copy/find/export/import)
    Snip(commands::snip::SnipArgs),
    /// Template manager with variable interpolation ({{var}})
    Template(commands::template::TemplateArgs),
    /// Macro manager (save/run/list — sequence of commands)
    Macro(commands::macro_cmd::MacroArgs),
    /// Watch multiple paths with different commands per path
    WatchMulti(commands::watch_multi::WatchMultiArgs),
    /// Long-running monitor of a command with alerts on change/error/text
    Monitor(commands::monitor::MonitorArgs),
    /// Persistent project notes — key-value memory across sessions
    Notes(commands::notes::NotesArgs),
    /// Send an OS notification
    Notify(commands::notify::NotifyArgs),
    /// Windows Task Scheduler wrapper (create/list/rm/run)
    Schedule(commands::schedule::ScheduleArgs),
    /// Countdown timer with optional notification and follow-up command
    Timer(commands::timer::TimerArgs),
    /// Benchmark a command (runs, min/mean/p50/p95/p99/max)
    Benchmark(commands::benchmark::BenchmarkArgs),
    /// Bookmarks: named file:line references for quick navigation
    Bookmark(commands::bookmark::BookmarkArgs),
    /// Codebase digest for AI (structural summary, per-file exports/imports)
    Digest(commands::digest::DigestArgs),
    /// Condense a file (strip comments/blanks/whitespace to save tokens)
    Condense(commands::condense::CondenseArgs),
    /// Split a file into per-function/class/section chunks with a manifest
    Chunk(commands::chunk::ChunkArgs),
    /// Build a task-focused AI prompt (finds relevant files, packs them)
    AiPrompt(commands::ai_prompt::AiPromptArgs),
    /// Full workspace snapshot (health + structure + git + analysis) as markdown
    WorkspaceReport(commands::workspace_report::WorkspaceReportArgs),
    /// English summary of what changed between two refs
    DiffSummary(commands::diff_summary::DiffSummaryArgs),
    /// Everything that changed since <date or ref>
    Since(commands::since::SinceArgs),
    /// Files with highest git churn (hotspots for refactoring)
    HotFiles(commands::hot_files::HotFilesArgs),
    /// Files nobody has touched in a long time
    StaleFiles(commands::stale_files::StaleFilesArgs),
    /// Every call site of a function/method with context
    Trace(commands::trace::TraceArgs),
    /// Transitive impact of changing a symbol (depth-based)
    BlastRadius(commands::blast_radius::BlastRadiusArgs),
    /// Files that "go together" with a given file (siblings + imports + git co-change)
    Related(commands::related::RelatedArgs),
    /// Caller/callee tree for a file (upstream + downstream)
    Route(commands::route::RouteArgs),
    /// Strip `export` keyword from unused exports (with backup + dry-run)
    TrimDead(commands::trim_dead::TrimDeadArgs),
    /// Find near-duplicate function bodies across the codebase
    Consolidate(commands::consolidate::ConsolidateArgs),
    /// Rename a symbol, run verify, auto-rollback on failure
    RenameSafe(commands::rename_safe::RenameSafeArgs),
    /// Build a SQLite index of files, symbols, and imports (fast reuse across commands)
    IndexBuild(commands::index_build::IndexBuildArgs),
    /// Incremental refresh: reindex only changed/new files
    IndexUpdate(commands::index_update::IndexUpdateArgs),
    /// Show index size, file/symbol/import counts, staleness
    IndexStatus(commands::index_status::IndexStatusArgs),
    /// Delete the index database
    IndexClear(commands::index_clear::IndexClearArgs),
    /// Print the index database path
    IndexLocate(commands::index_locate::IndexLocateArgs),
    /// Remove orphaned entries + vacuum
    IndexGc(commands::index_gc::IndexGcArgs),
    /// Fast symbol search via the index
    IndexSearch(commands::index_search::IndexSearchArgs),
    /// Show operation history (backups, patches, deletes) — auto-recorded
    History(commands::history::HistoryArgs),
    /// Undo the last N recorded operations (restores from backup)
    Undo(commands::undo::UndoArgs),
    /// Mark undone operations as redone (does not replay changes)
    Redo(commands::redo::RedoArgs),
    /// Import graph: fanout / fanin per file
    AnalyzeImports(commands::analyze_imports::AnalyzeImportsArgs),
    /// Export counts per file
    AnalyzeExports(commands::analyze_exports::AnalyzeExportsArgs),
    /// Coupling score (fanout+fanin — most entangled files)
    AnalyzeCoupling(commands::analyze_coupling::AnalyzeCouplingArgs),
    /// Files with the highest git churn
    AnalyzeChurn(commands::analyze_churn::AnalyzeChurnArgs),
    /// Hotspot analysis (churn × complexity)
    AnalyzeHotspot(commands::analyze_hotspot::AnalyzeHotspotArgs),
    /// Cyclomatic complexity per function (above threshold)
    AnalyzeComplexity(commands::analyze_complexity::AnalyzeComplexityArgs),
    /// Exported symbols never imported anywhere
    AnalyzeDeadExports(commands::analyze_dead_exports::AnalyzeDeadExportsArgs),
    /// Circular import detection
    AnalyzeCircular(commands::analyze_circular::AnalyzeCircularArgs),
    /// TS type coverage (any-density)
    AnalyzeTypeCoverage(commands::analyze_type_coverage::AnalyzeTypeCoverageArgs),
    /// Duplicated code blocks across files
    AnalyzeDuplication(commands::analyze_duplication::AnalyzeDuplicationArgs),
    /// Transitive impact if a file changes (upstream propagation)
    Impact(commands::impact::ImpactArgs),
    /// Heuristic English explanation of what a file does
    Explain(commands::explain::ExplainArgs),
    /// Report: codebase health as markdown
    ReportHealth(commands::report_health::ReportHealthArgs),
    /// Report: all TODO/FIXME/HACK comments as markdown
    ReportTodos(commands::report_todos::ReportTodosArgs),
    /// Report: import graph as markdown
    ReportImports(commands::report_imports::ReportImportsArgs),
    /// Report: public API surface as markdown
    ReportApi(commands::report_api::ReportApiArgs),
    /// Report: git contributors as markdown
    ReportContributors(commands::report_contributors::ReportContributorsArgs),
    /// Report: structural test coverage as markdown
    ReportCoverage(commands::report_coverage::ReportCoverageArgs),
    /// Report: recent git changes as markdown
    ReportChanges(commands::report_changes::ReportChangesArgs),
    /// Report: last cached compile errors as markdown
    ReportErrors(commands::report_errors::ReportErrorsArgs),
    /// Open a URL in a headless browser (or --visible)
    WebOpen(commands::web_open::WebOpenArgs),
    /// Screenshot a page (viewport / full-page / per-selector, device presets)
    WebScreenshot(commands::web_screenshot::WebScreenshotArgs),
    /// Render a page to PDF
    WebPdf(commands::web_pdf::WebPdfArgs),
    /// Extract visible text (optionally from a selector)
    WebText(commands::web_text::WebTextArgs),
    /// Extract rendered HTML (optionally per-selector)
    WebHtml(commands::web_html::WebHtmlArgs),
    /// Print the page title
    WebTitle(commands::web_title::WebTitleArgs),
    /// Extract all links from a page (with filters + same-domain)
    WebLinks(commands::web_links::WebLinksArgs),
    /// Click an element and inspect the resulting state
    WebClick(commands::web_click::WebClickArgs),
    /// Type into an input (with optional --submit and --clear)
    WebType(commands::web_type::WebTypeArgs),
    /// Evaluate JavaScript on a page and print the return value
    WebEval(commands::web_eval::WebEvalArgs),
    /// Wait for a selector / text / URL substring
    WebWait(commands::web_wait::WebWaitArgs),
    /// Structured scrape: field=selector pairs, optional repeating container
    WebScrape(commands::web_scrape::WebScrapeArgs),
    /// Screenshot many URLs to a directory
    WebScreenshotMany(commands::web_screenshot_many::WebScreenshotManyArgs),
    /// Screenshot one URL at multiple viewport widths (responsive audit)
    WebScreenshotSet(commands::web_screenshot_set::WebScreenshotSetArgs),
    /// Dump all cookies for a URL
    WebCookies(commands::web_cookies::WebCookiesArgs),
    /// Quick ready-state check for a URL (exits 1 if not ready)
    WebWsStatus(commands::web_ws_status::WebWsStatusArgs),
    /// Bulk headless render check across many URLs
    WebCheck(commands::web_check::WebCheckArgs),
    /// Manage AI provider API keys (register/unregister/list/test/rotate)
    AiKeys(commands::ai_keys::AiKeysArgs),
    /// AI configuration (default provider/model/budget/temperature/etc.)
    AiConfig(commands::ai_config::AiConfigArgs),
    /// List models available from a provider (with pricing + context window)
    AiModels(commands::ai_models::AiModelsArgs),
    /// Show all configured/available providers
    AiProviders(commands::ai_providers::AiProvidersArgs),
    /// Cumulative token + cost usage across all AI calls
    AiUsage(commands::ai_usage::AiUsageArgs),
    /// Edit / list / reset the AI system prompts
    AiPrompts(commands::ai_prompts::AiPromptsArgs),
    /// One-shot AI question with streaming + auto model selection
    AiAsk(commands::ai_ask::AiAskArgs),
    /// Search the web via SearXNG with DuckDuckGo fallback
    WebSearch(commands::web_search::WebSearchArgs),
    /// Configure search endpoint, fallbacks, limits
    WebSearchConfig(commands::web_search_config::WebSearchConfigArgs),
    /// List/test SearXNG instances (primary + fallbacks) with latency
    WebSearchInstances(commands::web_search_instances::WebSearchInstancesArgs),
    /// Fetch a URL and strip to article text (removes nav/scripts/styles)
    WebFetchClean(commands::web_fetch_clean::WebFetchCleanArgs),
    /// Dry-run: show exactly what an agent would retrieve for a query
    AiSearchTest(commands::ai_search_test::AiSearchTestArgs),
    /// Persistent multi-turn chat (with session storage)
    AiChat(commands::ai_chat::AiChatArgs),
    /// Autonomous agent loop with tool access (research, exploration, edits)
    AiAgent(commands::ai_agent::AiAgentArgs),
    /// LLM-quality explanation of what a file does
    AiExplain(commands::ai_explain::AiExplainArgs),
    /// AI code review with severity + line refs
    AiReview(commands::ai_review::AiReviewArgs),
    /// Agent that analyzes + patches + verifies + rolls back on failure
    AiFix(commands::ai_fix::AiFixArgs),
    /// Multi-step agent that plans → executes → verifies a refactor intent
    AiRefactor(commands::ai_refactor::AiRefactorArgs),
    /// Generate (and optionally apply) a git commit message from the diff
    AiCommitMessage(commands::ai_commit_message::AiCommitMessageArgs),
    /// Manage AI chat sessions (list/show/rm)
    AiSession(commands::ai_session::AiSessionArgs),
    /// Full history of every AI call (timestamp, task, model, cost, tokens)
    AiHistory(commands::ai_history::AiHistoryArgs),
    /// Full-text search across all past AI session messages
    AiRecall(commands::ai_recall::AiRecallArgs),
    /// Show current process AI spend vs configured caps
    AiBudget(commands::ai_budget::AiBudgetArgs),
    /// Interactive ore shell (no encoding corruption, built-in pipes)
    Shell(commands::shell::ShellArgs),
    /// Run a PowerShell one-liner directly (escape hatch)
    Ps(commands::ps::PsArgs),
    /// Find every place a property/variable is WRITTEN (assignment, mutation)
    TraceMutation(commands::trace_mutation::TraceMutationArgs),
    /// Filter refs to only WRITE sites (alias for trace-mutation)
    RefsWrite(commands::refs_write::RefsWriteArgs),
    /// Find only READ sites (excludes assignments)
    RefsRead(commands::refs_read::RefsReadArgs),
    /// Pack every file containing a symbol with context around usages
    PackSymbol(commands::pack_symbol::PackSymbolArgs),
    /// Execute a multi-step patch plan (DSL for multi-file atomic operations)
    PatchPlan(commands::patch_plan::PatchPlanArgs),
    /// Persistent multi-patch buffer (start/add/apply)
    PatchSession(commands::patch_session::PatchSessionArgs),
    /// Find every place a property is assigned a falsy/zero/reset value
    WhyReset(commands::why_reset::WhyResetArgs),
    /// Compare two git branches (optionally filtered by pattern)
    CompareBranches(commands::compare_branches::CompareBranchesArgs),
    /// Replace an entire function body by name (TS/JS/Rust/Python)
    PatchFn(commands::patch_fn::PatchFnArgs),
    /// Show data flow for a symbol: definition + callers with context
    Flow(commands::flow::FlowArgs),
    /// Follow the call chain FROM an entry function, N levels deep
    Follow(commands::follow::FollowArgs),
    /// Show file state: imports, exports, symbols, encoding, modified time
    State(commands::state::StateArgs),
    /// Show every call site of a symbol with context (reverse call graph)
    WhoCalls(commands::who_calls::WhoCallsArgs),
    /// Full symbol understanding: definition + all callers + call context
    ExplainSymbol(commands::explain_symbol::ExplainSymbolArgs),
    /// Smart read: file, function (--fn), line range (--range), or pattern (--around)
    Read(commands::read_cmd::ReadArgs),
    /// Semantic preview: find files likely to need changes for an intent
    PatchPlanPreview(commands::patch_plan_preview::PatchPlanPreviewArgs),
}

fn main() -> Result<()> {
    // Windows default stack (1MB) is too small for clap with 150+ subcommands.
    // Move real work to a thread with a bigger stack.
    let handle = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)  // 8MB
        .spawn(real_main)
        .expect("failed to spawn main thread");
    match handle.join() {
        Ok(res) => res,
        Err(_) => std::process::exit(1),
    }
}

fn real_main() -> Result<()> {
    // Exit quietly when downstream closes the pipe early (e.g. `ore find ... | head`),
    // instead of panicking with "failed printing to stdout: The pipe is being closed".
    install_broken_pipe_hook();

    #[cfg(windows)]
    { let _ = enable_ansi_support(); }

    let cli = Cli::parse();

    match cli.command {
        Commands::Find(a) => commands::find::run(a)?,
        Commands::FindMulti(a) => commands::find_multi::run(a)?,
        Commands::FindInChangedLines(a) => commands::find_in_changed_lines::run(a)?,
        Commands::Cat(a) => commands::cat::run(a)?,
        Commands::CatAround(a) => commands::cat_around::run(a)?,
        Commands::Line(a) => commands::line::run(a)?,
        Commands::Tree(a) => commands::tree::run(a)?,
        Commands::Backup(a) => commands::backup::run(a)?,
        Commands::Restore(a) => commands::restore::run(a)?,
        Commands::Patch(a) => commands::patch::run(a)?,
        Commands::PatchBatch(a) => commands::patch_batch::run(a)?,
        Commands::PatchInsert(a) => commands::patch_insert::run(a)?,
        Commands::PatchLines(a) => commands::patch_lines::run(a)?,
        Commands::PatchPreview(a) => commands::patch_preview::run(a)?,
        Commands::PatchRegex(a) => commands::patch_regex::run(a)?,
        Commands::PatchFuzzy(a) => commands::patch_fuzzy::run(a)?,
        Commands::Replace(a) => commands::replace::run(a)?,
        Commands::Diff(a) => commands::diff::run(a)?,
        Commands::DiffBackup(a) => commands::diff_backup::run(a)?,
        Commands::Encoding(a) => commands::encoding::run(a)?,
        Commands::EncodingNormalize(a) => commands::encoding_normalize::run(a)?,
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
        Commands::Tag(a) => commands::tag::run(a)?,
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
        Commands::MkfileFrom(a) => commands::mkfile_from::run(a)?,
        Commands::Checksum(a) => commands::checksum::run(a)?,
        Commands::FindDupes(a) => commands::find_dupes::run(a)?,
        Commands::VerifyChecksum(a) => commands::verify_checksum::run(a)?,
        Commands::Extract(a) => commands::extract::run(a)?,
        Commands::Pack(a) => commands::pack::run(a)?,
        Commands::PackChanged(a) => commands::pack_changed::run(a)?,
        Commands::PackLines(a) => commands::pack_lines::run(a)?,
        Commands::Slice(a) => commands::slice::run(a)?,
        Commands::Map(a) => commands::map::run(a)?,
        Commands::GitStatus(a) => commands::git_status::run(a)?,
        Commands::GitChanged(a) => commands::git_changed::run(a)?,
        Commands::GitDiff(a) => commands::git_diff::run(a)?,
        Commands::GitHistory(a) => commands::git_history::run(a)?,
        Commands::GitBlame(a) => commands::git_blame::run(a)?,
        Commands::GitBlameLine(a) => commands::git_blame_line::run(a)?,
        Commands::GitSearch(a) => commands::git_search::run(a)?,
        Commands::GitWho(a) => commands::git_who::run(a)?,
        Commands::GitStage(a) => commands::git_stage::run(a)?,
        Commands::GitCommit(a) => commands::git_commit::run(a)?,
        Commands::GitLog(a) => commands::git_log::run(a)?,
        Commands::GitLogLine(a) => commands::git_log_line::run(a)?,
        Commands::SearchDiff(a) => commands::search_diff::run(a)?,
        Commands::Run(a) => commands::run::run(a)?,
        Commands::Wait(a) => commands::wait::run(a)?,
        Commands::Retry(a) => commands::retry::run(a)?,
        Commands::Parallel(a) => commands::parallel::run(a)?,
        Commands::Sequence(a) => commands::sequence::run(a)?,
        Commands::Watch(a) => commands::watch::run(a)?,
        Commands::OnError(a) => commands::on_error::run(a)?,
        Commands::OnSuccess(a) => commands::on_success::run(a)?,
        Commands::SearchAnd(a) => commands::search_and::run(a)?,
        Commands::SearchOr(a) => commands::search_or::run(a)?,
        Commands::SearchNegative(a) => commands::search_negative::run(a)?,
        Commands::SearchMultiline(a) => commands::search_multiline::run(a)?,
        Commands::SearchFuzzy(a) => commands::search_fuzzy::run(a)?,
        Commands::SearchChanged(a) => commands::search_changed::run(a)?,
        Commands::SearchHistory(a) => commands::search_history::run(a)?,
        Commands::DiffWord(a) => commands::diff_word::run(a)?,
        Commands::DiffSemantic(a) => commands::diff_semantic::run(a)?,
        Commands::DiffIgnore(a) => commands::diff_ignore::run(a)?,
        Commands::DiffDirs(a) => commands::diff_dirs::run(a)?,
        Commands::Merge3(a) => commands::merge3::run(a)?,
        Commands::ApplyPatch(a) => commands::apply_patch::run(a)?,
        Commands::RevertPatch(a) => commands::revert_patch::run(a)?,
        Commands::Fetch(a) => commands::fetch::run(a)?,
        Commands::Post(a) => commands::post::run(a)?,
        Commands::Download(a) => commands::download::run(a)?,
        Commands::Headers(a) => commands::headers::run(a)?,
        Commands::Status(a) => commands::status::run(a)?,
        Commands::Ping(a) => commands::ping::run(a)?,
        Commands::Dns(a) => commands::dns::run(a)?,
        Commands::ApiTest(a) => commands::api_test::run(a)?,
        Commands::Filesize(a) => commands::filesize::run(a)?,
        Commands::Upload(a) => commands::upload::run(a)?,
        Commands::FetchMany(a) => commands::fetch_many::run(a)?,
        Commands::DownloadMany(a) => commands::download_many::run(a)?,
        Commands::CheckUrls(a) => commands::check_urls::run(a)?,
        Commands::ResumeDownload(a) => commands::resume_download::run(a)?,
        Commands::BenchUrl(a) => commands::bench_url::run(a)?,
        Commands::Ws(a) => commands::ws::run(a)?,
        Commands::Crawl(a) => commands::crawl::run(a)?,
        Commands::Show(a) => commands::show::run(a)?,
        Commands::Copy(a) => commands::copy::run(a)?,
        Commands::ToTemp(a) => commands::to_temp::run(a)?,
        Commands::OpenFile(a) => commands::open_file::run(a)?,
        Commands::HexView(a) => commands::hex_view::run(a)?,
        Commands::HexFind(a) => commands::hex_find::run(a)?,
        Commands::HexReplace(a) => commands::hex_replace::run(a)?,
        Commands::HexPatch(a) => commands::hex_patch::run(a)?,
        Commands::HexDiff(a) => commands::hex_diff::run(a)?,
        Commands::HexExtract(a) => commands::hex_extract::run(a)?,
        Commands::HexInsert(a) => commands::hex_insert::run(a)?,
        Commands::HexDelete(a) => commands::hex_delete::run(a)?,
        Commands::Strings(a) => commands::strings::run(a)?,
        Commands::Magic(a) => commands::magic::run(a)?,
        Commands::BinStats(a) => commands::bin_stats::run(a)?,
        Commands::Base64Encode(a) => commands::base64_encode::run(a)?,
        Commands::Base64Decode(a) => commands::base64_decode::run(a)?,
        Commands::Xxd(a) => commands::xxd::run(a)?,
        Commands::BinSlice(a) => commands::bin_slice::run(a)?,
        Commands::BinCat(a) => commands::bin_cat::run(a)?,
        Commands::Config(a) => commands::config_cmd::run(a)?,
        Commands::Alias(a) => commands::alias::run(a)?,
        Commands::Focus(a) => commands::focus::run(a)?,
        Commands::Session(a) => commands::session::run(a)?,
        Commands::SessionExport(a) => commands::session_export::run(a)?,
        Commands::CompileTs(a) => commands::compile_ts::run(a)?,
        Commands::CompileRust(a) => commands::compile_rust::run(a)?,
        Commands::CompileNode(a) => commands::compile_node::run(a)?,
        Commands::ErrorsLast(a) => commands::errors_last::run(a)?,
        Commands::Verify(a) => commands::verify::run(a)?,
        Commands::VerifyAnchor(a) => commands::verify_anchor::run(a)?,
        Commands::VerifyCompile(a) => commands::verify_compile::run(a)?,
        Commands::VerifyAndApply(a) => commands::verify_and_apply::run(a)?,
        Commands::ReAnchor(a) => commands::re_anchor::run(a)?,
        Commands::Health(a) => commands::health::run(a)?,
        Commands::VerifyJson(a) => commands::verify_json::run(a)?,
        Commands::VerifySyntax(a) => commands::verify_syntax::run(a)?,
        Commands::VerifyEncoding(a) => commands::verify_encoding::run(a)?,
        Commands::VerifyImports(a) => commands::verify_imports::run(a)?,
        Commands::Lock(a) => commands::lock::run(a)?,
        Commands::Unlock(a) => commands::unlock::run(a)?,
        Commands::Locks(a) => commands::locks::run(a)?,
        Commands::Tui(a) => commands::tui::run(a)?,
        Commands::JsonGet(a) => commands::json_get::run(a)?,
        Commands::JsonSet(a) => commands::json_set::run(a)?,
        Commands::JsonMerge(a) => commands::json_merge::run(a)?,
        Commands::JsonFmt(a) => commands::json_fmt::run(a)?,
        Commands::JsonQuery(a) => commands::json_query::run(a)?,
        Commands::JsonKeys(a) => commands::json_keys::run(a)?,
        Commands::YamlGet(a) => commands::yaml_get::run(a)?,
        Commands::YamlSet(a) => commands::yaml_set::run(a)?,
        Commands::YamlFmt(a) => commands::yaml_fmt::run(a)?,
        Commands::YamlToJson(a) => commands::yaml_to_json::run(a)?,
        Commands::TomlGet(a) => commands::toml_get::run(a)?,
        Commands::TomlSet(a) => commands::toml_set::run(a)?,
        Commands::TomlFmt(a) => commands::toml_fmt::run(a)?,
        Commands::TomlToJson(a) => commands::toml_to_json::run(a)?,
        Commands::CsvQuery(a) => commands::csv_query::run(a)?,
        Commands::CsvFilter(a) => commands::csv_filter::run(a)?,
        Commands::CsvSelect(a) => commands::csv_select::run(a)?,
        Commands::CsvToJson(a) => commands::csv_to_json::run(a)?,
        Commands::CsvStats(a) => commands::csv_stats::run(a)?,
        Commands::EnvGet(a) => commands::env_get::run(a)?,
        Commands::EnvSet(a) => commands::env_set::run(a)?,
        Commands::EnvDiff(a) => commands::env_diff::run(a)?,
        Commands::XmlGet(a) => commands::xml_get::run(a)?,
        Commands::XmlFmt(a) => commands::xml_fmt::run(a)?,
        Commands::XmlToJson(a) => commands::xml_to_json::run(a)?,
        Commands::Symbols(a) => commands::symbols::run(a)?,
        Commands::Outline(a) => commands::outline::run(a)?,
        Commands::Snippet(a) => commands::snippet::run(a)?,
        Commands::Pluck(a) => commands::pluck::run(a)?,
        Commands::Refs(a) => commands::refs::run(a)?,
        Commands::UsedBy(a) => commands::used_by::run(a)?,
        Commands::ImportsOf(a) => commands::imports_of::run(a)?,
        Commands::Neighbors(a) => commands::neighbors::run(a)?,
        Commands::AddImport(a) => commands::add_import::run(a)?,
        Commands::RemoveImport(a) => commands::remove_import::run(a)?,
        Commands::SplitFile(a) => commands::split_file::run(a)?,
        Commands::MergeFiles(a) => commands::merge_files::run(a)?,
        Commands::ExtractFn(a) => commands::extract_fn::run(a)?,
        Commands::MoveWithImports(a) => commands::move_with_imports::run(a)?,
        Commands::Hub(a) => commands::hub::run(a)?,
        Commands::FlattenHub(a) => commands::flatten_hub::run(a)?,
        Commands::RenameSymbol(a) => commands::rename_symbol::run(a)?,
        Commands::Organize(a) => commands::organize::run(a)?,
        Commands::GitAutoCommit(a) => commands::git_auto_commit::run(a)?,
        Commands::GitAutoMessage(a) => commands::git_auto_message::run(a)?,
        Commands::GitSuggestCommit(a) => commands::git_suggest_commit::run(a)?,
        Commands::GitCommitBody(a) => commands::git_commit_body::run(a)?,
        Commands::GitChangelog(a) => commands::git_changelog::run(a)?,
        Commands::GitReleaseNotes(a) => commands::git_release_notes::run(a)?,
        Commands::GitUndoCommit(a) => commands::git_undo_commit::run(a)?,
        Commands::GitAmend(a) => commands::git_amend::run(a)?,
        Commands::GitFixup(a) => commands::git_fixup::run(a)?,
        Commands::GitCleanupBranches(a) => commands::git_cleanup_branches::run(a)?,
        Commands::GitStashNamed(a) => commands::git_stash_named::run(a)?,
        Commands::Scaffold(a) => commands::scaffold::run(a)?,
        Commands::ScaffoldAdd(a) => commands::scaffold_add::run(a)?,
        Commands::ScaffoldComponent(a) => commands::scaffold_component::run(a)?,
        Commands::ScaffoldHook(a) => commands::scaffold_hook::run(a)?,
        Commands::ScaffoldStore(a) => commands::scaffold_store::run(a)?,
        Commands::ScaffoldContext(a) => commands::scaffold_context::run(a)?,
        Commands::ScaffoldApi(a) => commands::scaffold_api::run(a)?,
        Commands::ScaffoldTest(a) => commands::scaffold_test::run(a)?,
        Commands::Setup(a) => commands::setup::run(a)?,
        Commands::CheckDeps(a) => commands::check_deps::run(a)?,
        Commands::InstallIfMissing(a) => commands::install_if_missing::run(a)?,
        Commands::Snip(a) => commands::snip::run(a)?,
        Commands::Template(a) => commands::template::run(a)?,
        Commands::Macro(a) => commands::macro_cmd::run(a)?,
        Commands::WatchMulti(a) => commands::watch_multi::run(a)?,
        Commands::Monitor(a) => commands::monitor::run(a)?,
        Commands::Notes(a) => commands::notes::run(a)?,
        Commands::Notify(a) => commands::notify::run(a)?,
        Commands::Schedule(a) => commands::schedule::run(a)?,
        Commands::Timer(a) => commands::timer::run(a)?,
        Commands::Benchmark(a) => commands::benchmark::run(a)?,
        Commands::Bookmark(a) => commands::bookmark::run(a)?,
        Commands::Digest(a) => commands::digest::run(a)?,
        Commands::Condense(a) => commands::condense::run(a)?,
        Commands::Chunk(a) => commands::chunk::run(a)?,
        Commands::AiPrompt(a) => commands::ai_prompt::run(a)?,
        Commands::WorkspaceReport(a) => commands::workspace_report::run(a)?,
        Commands::DiffSummary(a) => commands::diff_summary::run(a)?,
        Commands::Since(a) => commands::since::run(a)?,
        Commands::HotFiles(a) => commands::hot_files::run(a)?,
        Commands::StaleFiles(a) => commands::stale_files::run(a)?,
        Commands::Trace(a) => commands::trace::run(a)?,
        Commands::BlastRadius(a) => commands::blast_radius::run(a)?,
        Commands::Related(a) => commands::related::run(a)?,
        Commands::Route(a) => commands::route::run(a)?,
        Commands::TrimDead(a) => commands::trim_dead::run(a)?,
        Commands::Consolidate(a) => commands::consolidate::run(a)?,
        Commands::RenameSafe(a) => commands::rename_safe::run(a)?,
        Commands::IndexBuild(a) => commands::index_build::run(a)?,
        Commands::IndexUpdate(a) => commands::index_update::run(a)?,
        Commands::IndexStatus(a) => commands::index_status::run(a)?,
        Commands::IndexClear(a) => commands::index_clear::run(a)?,
        Commands::IndexLocate(a) => commands::index_locate::run(a)?,
        Commands::IndexGc(a) => commands::index_gc::run(a)?,
        Commands::IndexSearch(a) => commands::index_search::run(a)?,
        Commands::History(a) => commands::history::run(a)?,
        Commands::Undo(a) => commands::undo::run(a)?,
        Commands::Redo(a) => commands::redo::run(a)?,
        Commands::AnalyzeImports(a) => commands::analyze_imports::run(a)?,
        Commands::AnalyzeExports(a) => commands::analyze_exports::run(a)?,
        Commands::AnalyzeCoupling(a) => commands::analyze_coupling::run(a)?,
        Commands::AnalyzeChurn(a) => commands::analyze_churn::run(a)?,
        Commands::AnalyzeHotspot(a) => commands::analyze_hotspot::run(a)?,
        Commands::AnalyzeComplexity(a) => commands::analyze_complexity::run(a)?,
        Commands::AnalyzeDeadExports(a) => commands::analyze_dead_exports::run(a)?,
        Commands::AnalyzeCircular(a) => commands::analyze_circular::run(a)?,
        Commands::AnalyzeTypeCoverage(a) => commands::analyze_type_coverage::run(a)?,
        Commands::AnalyzeDuplication(a) => commands::analyze_duplication::run(a)?,
        Commands::Impact(a) => commands::impact::run(a)?,
        Commands::Explain(a) => commands::explain::run(a)?,
        Commands::ReportHealth(a) => commands::report_health::run(a)?,
        Commands::ReportTodos(a) => commands::report_todos::run(a)?,
        Commands::ReportImports(a) => commands::report_imports::run(a)?,
        Commands::ReportApi(a) => commands::report_api::run(a)?,
        Commands::ReportContributors(a) => commands::report_contributors::run(a)?,
        Commands::ReportCoverage(a) => commands::report_coverage::run(a)?,
        Commands::ReportChanges(a) => commands::report_changes::run(a)?,
        Commands::ReportErrors(a) => commands::report_errors::run(a)?,
        Commands::WebOpen(a) => commands::web_open::run(a)?,
        Commands::WebScreenshot(a) => commands::web_screenshot::run(a)?,
        Commands::WebPdf(a) => commands::web_pdf::run(a)?,
        Commands::WebText(a) => commands::web_text::run(a)?,
        Commands::WebHtml(a) => commands::web_html::run(a)?,
        Commands::WebTitle(a) => commands::web_title::run(a)?,
        Commands::WebLinks(a) => commands::web_links::run(a)?,
        Commands::WebClick(a) => commands::web_click::run(a)?,
        Commands::WebType(a) => commands::web_type::run(a)?,
        Commands::WebEval(a) => commands::web_eval::run(a)?,
        Commands::WebWait(a) => commands::web_wait::run(a)?,
        Commands::WebScrape(a) => commands::web_scrape::run(a)?,
        Commands::WebScreenshotMany(a) => commands::web_screenshot_many::run(a)?,
        Commands::WebScreenshotSet(a) => commands::web_screenshot_set::run(a)?,
        Commands::WebCookies(a) => commands::web_cookies::run(a)?,
        Commands::WebWsStatus(a) => commands::web_ws_status::run(a)?,
        Commands::WebCheck(a) => commands::web_check::run(a)?,
        Commands::AiKeys(a) => commands::ai_keys::run(a)?,
        Commands::AiConfig(a) => commands::ai_config::run(a)?,
        Commands::AiModels(a) => commands::ai_models::run(a)?,
        Commands::AiProviders(a) => commands::ai_providers::run(a)?,
        Commands::AiUsage(a) => commands::ai_usage::run(a)?,
        Commands::AiPrompts(a) => commands::ai_prompts::run(a)?,
        Commands::AiAsk(a) => commands::ai_ask::run(a)?,
        Commands::WebSearch(a) => commands::web_search::run(a)?,
        Commands::WebSearchConfig(a) => commands::web_search_config::run(a)?,
        Commands::WebSearchInstances(a) => commands::web_search_instances::run(a)?,
        Commands::WebFetchClean(a) => commands::web_fetch_clean::run(a)?,
        Commands::AiSearchTest(a) => commands::ai_search_test::run(a)?,
        Commands::AiChat(a) => commands::ai_chat::run(a)?,
        Commands::AiAgent(a) => commands::ai_agent::run(a)?,
        Commands::AiExplain(a) => commands::ai_explain::run(a)?,
        Commands::AiReview(a) => commands::ai_review::run(a)?,
        Commands::AiFix(a) => commands::ai_fix::run(a)?,
        Commands::AiRefactor(a) => commands::ai_refactor::run(a)?,
        Commands::AiCommitMessage(a) => commands::ai_commit_message::run(a)?,
        Commands::AiSession(a) => commands::ai_session::run(a)?,
        Commands::AiHistory(a) => commands::ai_history::run(a)?,
        Commands::AiRecall(a) => commands::ai_recall::run(a)?,
        Commands::AiBudget(a) => commands::ai_budget::run(a)?,
        Commands::Shell(a) => commands::shell::run(a)?,
        Commands::Ps(a) => commands::ps::run(a)?,
        Commands::TraceMutation(a) => commands::trace_mutation::run(a)?,
        Commands::RefsWrite(a) => commands::refs_write::run(a)?,
        Commands::RefsRead(a) => commands::refs_read::run(a)?,
        Commands::PackSymbol(a) => commands::pack_symbol::run(a)?,
        Commands::PatchPlan(a) => commands::patch_plan::run(a)?,
        Commands::PatchSession(a) => commands::patch_session::run(a)?,
        Commands::WhyReset(a) => commands::why_reset::run(a)?,
        Commands::CompareBranches(a) => commands::compare_branches::run(a)?,
        Commands::PatchFn(a) => commands::patch_fn::run(a)?,
        Commands::Flow(a) => commands::flow::run(a)?,
        Commands::Follow(a) => commands::follow::run(a)?,
        Commands::State(a) => commands::state::run(a)?,
        Commands::WhoCalls(a) => commands::who_calls::run(a)?,
        Commands::ExplainSymbol(a) => commands::explain_symbol::run(a)?,
        Commands::Read(a) => commands::read_cmd::run(a)?,
        Commands::PatchPlanPreview(a) => commands::patch_plan_preview::run(a)?,
    }
    Ok(())
}

fn install_broken_pipe_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        if msg.contains("failed printing to stdout") || msg.contains("Broken pipe") || msg.contains("pipe is being closed") {
            std::process::exit(0);
        }
        default_hook(info);
    }));
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
