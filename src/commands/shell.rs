use anyhow::Result;
use clap::Args;
use colored::*;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::{DefaultHistory, History, SearchDirection};
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};
use std::io::Write;
use std::path::PathBuf;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct ShellArgs {
    /// Working directory (default: current)
    #[arg(short = 'd', long)]
    dir: Option<PathBuf>,

    /// Don't show the banner
    #[arg(long)]
    no_banner: bool,
}

struct OreHelper {
    commands: Vec<String>,
}

impl OreHelper {
    fn new() -> Self {
        OreHelper {
            commands: COMMANDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Completer for OreHelper {
    type Candidate = String;
    fn complete(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let trimmed = line[..pos].trim();
        // If line starts with ore, complete subcommands
        let prefix = if let Some(rest) = trimmed.strip_prefix("ore ") {
            rest
        } else {
            trimmed
        };
        let candidates: Vec<String> = self.commands.iter()
            .filter(|c| c.starts_with(prefix))
            .map(|c| c.clone())
            .collect();
        let start = pos - prefix.len();
        Ok((start, candidates))
    }
}

impl Helper for OreHelper {}
impl Highlighter for OreHelper {}
impl Hinter for OreHelper {
    type Hint = String;
}
impl Validator for OreHelper {}

pub fn run(args: ShellArgs) -> Result<()> {
    // Set working dir
    if let Some(d) = &args.dir {
        if !d.exists() { anyhow::bail!("Directory not found: {}", d.display()); }
        std::env::set_current_dir(d)?;
    }

    // Force UTF-8 console output on Windows
    #[cfg(windows)]
    {
        unsafe {
            extern "system" { fn SetConsoleOutputCP(id: u32) -> i32; }
            SetConsoleOutputCP(65001);
        }
        // Also set input codepage
        unsafe {
            extern "system" { fn SetConsoleCP(id: u32) -> i32; }
            SetConsoleCP(65001);
        }
    }

    if !args.no_banner {
        print_banner();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("{} {}", "Directory:".dimmed(), cwd.display().to_string().cyan());
    println!("{}", "Type ore commands without the 'ore' prefix. Type 'help' for usage, 'exit' to quit.".dimmed());
    println!();

    let helper = OreHelper::new();
    let mut rl = Editor::<OreHelper, DefaultHistory>::new()?;
    rl.set_helper(Some(helper));

    // Load history
    let history_path = dirs_hint().join("ore_shell_history.txt");
    let _ = rl.load_history(&history_path);

    let mut multiline_buf: Option<String> = None;

    loop {
        let prompt = if multiline_buf.is_some() {
            "...> ".magenta().bold().to_string()
        } else {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let short = cwd.file_name().and_then(|n| n.to_str()).unwrap_or(".");
            format!("{} {} ", "ore".green().bold(), short.cyan())
        };

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim_end().to_string();

                // Multi-line mode: collecting a heredoc/patch block
                if let Some(ref mut buf) = multiline_buf {
                    if line == "." || line == "EOF" || line == "END" {
                        let complete = std::mem::take(buf);
                        multiline_buf = None;
                        execute_input(&complete);
                        continue;
                    }
                    buf.push_str(&line);
                    buf.push('\n');
                    continue;
                }

                // Empty line
                if line.is_empty() { continue; }

                // Add to history
                let _ = rl.add_history_entry(line.as_str());

                // Built-in shell commands
                match line.as_str() {
                    "exit" | "quit" | "q" => break,
                    "help" | "?" => { print_help(); continue; }
                    "clear" | "cls" => { clear_screen(); continue; }
                    "pwd" => { println!("{}", std::env::current_dir().unwrap_or_default().display()); continue; }
                    "history" => { print_history(&rl); continue; }
                    _ => {}
                }

                // cd command
                if line.starts_with("cd ") {
                    let target = line[3..].trim();
                    let target = if target == "~" {
                        dirs_home().unwrap_or_else(|| PathBuf::from("."))
                    } else {
                        PathBuf::from(target)
                    };
                    match std::env::set_current_dir(&target) {
                        Ok(_) => println!("{} {}", "→".green(), std::env::current_dir().unwrap_or_default().display().to_string().cyan()),
                        Err(e) => eprintln!("{} {}: {}", "!".red(), target.display(), e),
                    }
                    continue;
                }

                // Start multi-line input: lines ending with `<<` or the word `begin`
                if line.ends_with("<<") || line.to_lowercase() == "begin" {
                    let prefix = if line.ends_with("<<") { &line[..line.len()-2] } else { "" };
                    let mut buf = String::new();
                    if !prefix.trim().is_empty() {
                        buf.push_str(prefix.trim());
                        buf.push('\n');
                    }
                    multiline_buf = Some(buf);
                    println!("{}", "(multi-line input: type '.' or 'EOF' on a blank line to execute)".dimmed());
                    continue;
                }

                execute_input(&line);
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C: cancel current input
                multiline_buf = None;
                println!("{}", "^C".dimmed());
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D: exit
                break;
            }
            Err(e) => {
                eprintln!("{} readline error: {}", "!".red(), e);
                break;
            }
        }
    }

    // Save history
    let _ = std::fs::create_dir_all(history_path.parent().unwrap_or(&PathBuf::from(".")));
    let _ = rl.save_history(&history_path);
    println!("{}", "bye.".dimmed());
    Ok(())
}

fn execute_input(input: &str) {
    let input = input.trim();
    if input.is_empty() { return; }

    let (cmd_part, pipe) = parse_pipe(input);
    let cmd_part = cmd_part.trim();

    // Strip leading "ore " if user typed it
    let effective = if cmd_part.starts_with("ore ") { &cmd_part[4..] } else { cmd_part };

    // Parse the command line into tokens ourselves (handles quotes, escapes properly)
    let tokens = match shell_tokenize(effective) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} parse error: {}", "!".red(), e);
            return;
        }
    };

    if tokens.is_empty() { return; }

    // Check if first token is an ore subcommand
    let first = tokens[0].as_str();
    if is_ore_command(first) {
        let start = std::time::Instant::now();

        // Spawn ore.exe directly with per-arg OS strings — no cmd.exe, so
        // characters like $ % & ^ | < > ( ) # ! ' ` \ { } [ ] are all literal.
        let ore_exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("ore"));

        let result = std::process::Command::new(&ore_exe)
            .args(&tokens)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        let elapsed = start.elapsed().as_millis();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = if stderr.is_empty() { stdout.clone() } else { format!("{}\n{}", stdout, stderr) };
                let exit_code = output.status.code().unwrap_or(-1);

                handle_output(&combined, &pipe);

                if exit_code != 0 {
                    eprintln!("{} exit {} ({}ms)", "✗".red(), exit_code, elapsed);
                }
            }
            Err(e) => {
                eprintln!("{} failed to execute: {}", "✗".red(), e.to_string().red());
            }
        }
    } else {
        // Non-ore command: fall through to the system shell (git, npm, dir, ...).
        let start = std::time::Instant::now();
        let result = run_cmd(cmd_part, false, false);
        let elapsed = start.elapsed().as_millis();
        match result {
            Ok(r) => {
                let combined = format!("{}{}", r.stdout, if r.stderr.is_empty() { String::new() } else { format!("\n{}", r.stderr) });
                handle_output(&combined, &pipe);
                if !r.success() {
                    eprintln!("{} exit {} ({}ms)", "✗".red(), r.exit_code, elapsed);
                }
            }
            Err(e) => eprintln!("{} {}", "✗".red(), e.to_string().red()),
        }
    }
}

fn handle_output(output: &str, pipe: &Pipe) {
    if output.is_empty() { return; }
    match pipe {
        Pipe::None => { print!("{}", output); }
        Pipe::Notepad | Pipe::Show => {
            let stripped = strip_ansi_codes(output);
            open_in_notepad(&stripped);
        }
        Pipe::Copy => {
            let stripped = strip_ansi_codes(output);
            copy_to_clipboard(&stripped);
            eprintln!("{} {} bytes copied", "OK".green(), stripped.len().to_string().yellow());
        }
        Pipe::File(path) => {
            let stripped = strip_ansi_codes(output);
            match std::fs::write(path, &stripped) {
                Ok(_) => eprintln!("{} {} ({} bytes)", "Wrote:".green(), path.cyan(), stripped.len().to_string().yellow()),
                Err(e) => eprintln!("{} write failed: {}", "!".red(), e),
            }
        }
        Pipe::Append(path) => {
            let stripped = strip_ansi_codes(output);
            match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut f) => {
                    let _ = f.write_all(stripped.as_bytes());
                    eprintln!("{} appended to {} ({} bytes)", "OK".green(), path.cyan(), stripped.len().to_string().yellow());
                }
                Err(e) => eprintln!("{} append failed: {}", "!".red(), e),
            }
        }
    }
}

#[derive(Debug)]
enum Pipe {
    None,
    Notepad,
    Copy,
    Show,
    File(String),
    Append(String),
}

/// Split a command line into (command, pipe target). Quote-aware: `|`, `>` and
/// `>>` only count as operators when they appear OUTSIDE quotes, so content
/// like `-c "x > 0"` is not mistaken for a redirect. The LAST outside-quote
/// operator wins.
fn parse_pipe(input: &str) -> (&str, Pipe) {
    let bytes = input.as_bytes();
    let n = bytes.len();
    let mut in_double = false;
    let mut in_single = false;
    let mut last: Option<(usize, Pipe)> = None; // byte index of operator start

    let mut i = 0;
    while i < n {
        let c = bytes[i];
        if in_single {
            if c == b'\'' { in_single = false; }
            i += 1;
            continue;
        }
        if in_double {
            if c == b'\\' && i + 1 < n { i += 2; continue; }
            if c == b'"' { in_double = false; }
            i += 1;
            continue;
        }
        match c {
            b'\'' => { in_single = true; i += 1; }
            b'"' => { in_double = true; i += 1; }
            b'|' => {
                let target = input[i+1..].trim().to_lowercase();
                match target.as_str() {
                    "notepad" | "np" => last = Some((i, Pipe::Notepad)),
                    "copy" | "clip" | "clipboard" => last = Some((i, Pipe::Copy)),
                    "show" => last = Some((i, Pipe::Show)),
                    _ => {}
                }
                i += 1;
            }
            b'>' => {
                if i + 1 < n && bytes[i+1] == b'>' {
                    let target = input[i+2..].trim();
                    if !target.is_empty() && !target.starts_with('>') && !target.starts_with('=') {
                        last = Some((i, Pipe::Append(target.to_string())));
                    }
                    i += 2;
                } else {
                    let target = input[i+1..].trim();
                    if !target.is_empty() && !target.starts_with('>') && !target.starts_with('=') {
                        last = Some((i, Pipe::File(target.to_string())));
                    }
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    match last {
        Some((idx, pipe)) => (&input[..idx], pipe),
        None => (input, Pipe::None),
    }
}

fn is_ore_command(input: &str) -> bool {
    let first_word = input.split_whitespace().next().unwrap_or("");
    COMMANDS.contains(&first_word)
}

/// Tokenize a command line into args, handling:
/// - Double-quoted strings (preserves content literally, no $var or special char interpretation)
/// - Single-quoted strings (fully literal)
/// - Escaped characters with backslash (\")
/// - Everything else split on whitespace
///
/// This replaces cmd.exe's broken parsing for ore commands.
fn shell_tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    while let Some(c) = chars.next() {
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current.push(c);
            }
            continue;
        }

        if in_double_quote {
            if c == '\\' {
                // Backslash escape inside double quotes
                if let Some(&next) = chars.peek() {
                    match next {
                        '"' | '\\' | 'n' | 't' | 'r' => {
                            chars.next();
                            match next {
                                '"' => current.push('"'),
                                '\\' => current.push('\\'),
                                'n' => current.push('\n'),
                                't' => current.push('\t'),
                                'r' => current.push('\r'),
                                _ => {}
                            }
                        }
                        _ => {
                            // Not a recognized escape — keep the backslash
                            current.push('\\');
                        }
                    }
                } else {
                    current.push('\\');
                }
            } else if c == '"' {
                in_double_quote = false;
            } else {
                // Everything else is literal inside double quotes
                // $, %, &, ^, |, (, ), <, >, #, !, `, {, }, [, ] — ALL literal
                current.push(c);
            }
            continue;
        }

        // Not inside any quote
        match c {
            '"' => { in_double_quote = true; }
            '\'' => { in_single_quote = true; }
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            '\\' => {
                // Windows paths: keep the backslash literal outside quotes
                current.push('\\');
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_double_quote {
        return Err("Unterminated double quote".to_string());
    }
    if in_single_quote {
        return Err("Unterminated single quote".to_string());
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn open_in_notepad(text: &str) {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
    let path = std::env::temp_dir().join(format!("ore-shell-{}.txt", ts));
    if let Err(e) = std::fs::write(&path, text) {
        eprintln!("{} failed to write temp: {}", "!".red(), e);
        return;
    }
    eprintln!("{} {} ({} bytes)", "Opening:".cyan(), path.display().to_string().dimmed(), text.len().to_string().yellow());
    let _ = std::process::Command::new("notepad").arg(&path).spawn();
}

fn copy_to_clipboard(text: &str) {
    #[cfg(windows)]
    {
        use std::process::{Command, Stdio};
        if let Ok(mut child) = Command::new("clip.exe").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("{}", "Clipboard not implemented on this platform".yellow());
    }
}

fn strip_ansi_codes(s: &str) -> String {
    // Byte-level ANSI stripping so multi-byte UTF-8 survives intact
    // (no encoding corruption — that's the shell's whole point).
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let c = bytes[i];
                i += 1;
                if (0x40..=0x7E).contains(&c) { break; }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::stdout().flush();
}

fn print_history(rl: &Editor<OreHelper, DefaultHistory>) {
    let hist = rl.history();
    let len = hist.len();
    let start = len.saturating_sub(30);
    for i in start..len {
        if let Ok(Some(res)) = hist.get(i, SearchDirection::Forward) {
            println!("  {} {}", format!("{:>3}", i + 1).dimmed(), res.entry);
        }
    }
}

fn print_banner() {
    println!("{}", r#"
  ██████╗ ██████╗ ███████╗
 ██╔═══██╗██╔══██╗██╔════╝
 ██║   ██║██████╔╝█████╗
 ██║   ██║██╔══██╗██╔══╝
 ╚██████╔╝██║  ██║███████╗
  ╚═════╝ ╚═╝  ╚═╝╚══════╝
"#.green());
    println!("  {} {}", "ore shell".green().bold(), format!("v{}", env!("CARGO_PKG_VERSION")).dimmed());
    println!("  {}", "Interactive command shell — no encoding corruption, no escaping.".dimmed());
    println!();
}

fn print_help() {
    println!("{}", "Ore Shell — built-in commands:".cyan().bold());
    println!();
    println!("  {}       {}", "exit / quit / q".yellow(), "Exit the shell");
    println!("  {}             {}", "help / ?".yellow(), "Show this help");
    println!("  {}         {}", "clear / cls".yellow(), "Clear screen");
    println!("  {}                 {}", "pwd".yellow(), "Print working directory");
    println!("  {}          {}", "cd <path>".yellow(), "Change directory");
    println!("  {}             {}", "history".yellow(), "Show command history");
    println!();
    println!("{}", "Ore commands:".cyan().bold());
    println!("  Type any ore command without the 'ore' prefix:");
    println!("  {}     →  ore find \"TODO\" src/ -c", "find \"TODO\" src/ -c".yellow());
    println!("  {}                →  ore tree src/ -e rs", "tree src/ -e rs".yellow());
    println!("  {}        →  ore ai-ask \"what is this?\"", "ai-ask \"what is this?\"".yellow());
    println!();
    println!("{}", "Pipes:".cyan().bold());
    println!("  {}    {}", "cmd | notepad".yellow(), "Open output in notepad (ANSI stripped)");
    println!("  {}       {}", "cmd | copy".yellow(), "Copy output to clipboard");
    println!("  {}       {}", "cmd | show".yellow(), "Same as notepad");
    println!("  {}      {}", "cmd > file".yellow(), "Write output to file");
    println!("  {}     {}", "cmd >> file".yellow(), "Append output to file");
    println!();
    println!("{}", "Multi-line input:".cyan().bold());
    println!("  End a line with {} to start multi-line mode.", "<<".yellow());
    println!("  Type {} or {} on a blank line to execute.", ".".yellow(), "EOF".yellow());
    println!();
    println!("{}", "Examples:".cyan().bold());
    println!("  {}", "find \"useEffect\" src/ -e ts,tsx | notepad".dimmed());
    println!("  {}", "pack src/App.tsx src/utils.ts -f tag | copy".dimmed());
    println!("  {}", "ai-ask \"what does this project do?\"".dimmed());
    println!("  {}", "health . -e ts,tsx > report.txt".dimmed());
    println!("  {}", "patch src/App.tsx -f \"old\" -r \"new\"".dimmed());
    println!();
}

fn dirs_hint() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("ore")
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config").join("ore")).unwrap_or_else(|| PathBuf::from("."))
    }
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    { std::env::var_os("USERPROFILE").map(PathBuf::from) }
    #[cfg(not(windows))]
    { std::env::var_os("HOME").map(PathBuf::from) }
}

// All ore subcommands for tab completion
const COMMANDS: &[&str] = &[
    "find", "cat", "line", "head", "tail", "tree", "backup", "restore", "patch", "replace",
    "diff", "encoding", "newlines", "insert", "delete-lines", "replace-line", "replace-range",
    "before", "after", "surround", "replace-project", "replace-ext", "replace-dir",
    "patch-project", "rename-bulk", "count", "stats", "wc", "dedup-lines", "sort-lines",
    "trim", "strip-blank-lines", "collapse-blank-lines", "purge-backups",
    "mv", "cp", "rm", "touch", "mkdir", "mkfile", "checksum", "find-dupes", "verify-checksum",
    "extract", "pack", "slice", "map", "show", "copy", "to-temp", "open-file",
    "hex-view", "hex-find", "hex-replace", "hex-patch", "hex-diff", "hex-extract",
    "hex-insert", "hex-delete", "strings", "magic", "bin-stats",
    "base64-encode", "base64-decode", "xxd", "bin-slice", "bin-cat",
    "git-status", "git-changed", "git-diff", "git-history", "git-blame", "git-search",
    "git-who", "git-stage", "git-commit", "git-log",
    "git-auto-commit", "git-auto-message", "git-suggest-commit", "git-commit-body",
    "git-changelog", "git-release-notes", "git-undo-commit", "git-amend", "git-fixup",
    "git-cleanup-branches", "git-stash-named",
    "run", "wait", "retry", "parallel", "sequence", "watch", "watch-multi", "monitor",
    "on-error", "on-success", "notify", "schedule", "timer", "benchmark",
    "search-and", "search-or", "search-negative", "search-multiline", "search-fuzzy",
    "search-changed", "search-history",
    "diff-word", "diff-semantic", "diff-ignore", "diff-dirs", "merge3",
    "apply-patch", "revert-patch",
    "fetch", "post", "download", "headers", "status", "ping", "dns", "api-test",
    "filesize", "upload", "fetch-many", "download-many", "check-urls",
    "resume-download", "bench-url", "ws", "crawl",
    "web-open", "web-screenshot", "web-pdf", "web-text", "web-html", "web-title",
    "web-links", "web-click", "web-type", "web-eval", "web-wait", "web-scrape",
    "web-screenshot-many", "web-screenshot-set", "web-cookies", "web-ws-status", "web-check",
    "web-search", "web-search-config", "web-search-instances", "web-fetch-clean",
    "config", "alias", "focus", "session",
    "compile-ts", "compile-rust", "compile-node", "errors-last", "verify", "health",
    "verify-json", "verify-syntax", "verify-encoding", "verify-imports",
    "lock", "unlock", "locks",
    "json-get", "json-set", "json-merge", "json-fmt", "json-query", "json-keys",
    "yaml-get", "yaml-set", "yaml-fmt", "yaml-to-json",
    "toml-get", "toml-set", "toml-fmt", "toml-to-json",
    "csv-query", "csv-filter", "csv-select", "csv-to-json", "csv-stats",
    "env-get", "env-set", "env-diff",
    "xml-get", "xml-fmt", "xml-to-json",
    "symbols", "outline", "snippet", "pluck", "refs", "used-by", "imports-of", "neighbors",
    "add-import", "remove-import",
    "split-file", "merge-files", "extract-fn", "move-with-imports", "hub", "flatten-hub",
    "rename-symbol", "organize",
    "scaffold", "scaffold-add", "scaffold-component", "scaffold-hook", "scaffold-store",
    "scaffold-context", "scaffold-api", "scaffold-test", "setup", "check-deps", "install-if-missing",
    "snip", "template", "macro",
    "digest", "condense", "chunk", "ai-prompt", "workspace-report",
    "diff-summary", "since", "hot-files", "stale-files", "trace", "blast-radius",
    "related", "route", "trim-dead", "consolidate", "rename-safe",
    "index-build", "index-update", "index-status", "index-clear", "index-locate",
    "index-gc", "index-search", "history", "undo", "redo",
    "tui", "shell",
    "analyze-imports", "analyze-exports", "analyze-coupling", "analyze-churn",
    "analyze-hotspot", "analyze-complexity", "analyze-dead-exports", "analyze-circular",
    "analyze-type-coverage", "analyze-duplication", "impact", "explain",
    "report-health", "report-todos", "report-imports", "report-api",
    "report-contributors", "report-coverage", "report-changes", "report-errors",
    "ai-keys", "ai-config", "ai-models", "ai-providers", "ai-usage", "ai-prompts",
    "ai-ask", "ai-chat", "ai-agent", "ai-explain", "ai-review", "ai-fix",
    "ai-refactor", "ai-commit-message", "ai-session", "ai-history", "ai-recall",
    "ai-budget",
];
