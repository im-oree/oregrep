use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Result of a single command execution.
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
}

impl RunResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Execute a command. The command line is parsed into program + args and
/// spawned DIRECTLY — no cmd.exe / sh -c layer — so characters like
/// `$ % & ^ | < > ( ) # ! ' " \ { } [ ]` are passed to the program literally.
///
/// The system shell is used only when the command genuinely requires it:
/// pipes, redirects, command chaining (`&&`, `||`), or cmd builtins (echo,
/// dir, del, ...) that don't exist as executables.
pub fn run_cmd(cmd_line: &str, stream: bool, silent: bool) -> Result<RunResult> {
    run_cmd_in(cmd_line, None, stream, silent)
}

/// Like `run_cmd`, but runs with `cwd` as the child's working directory.
pub fn run_cmd_in(cmd_line: &str, cwd: Option<&Path>, stream: bool, silent: bool) -> Result<RunResult> {
    let start = std::time::Instant::now();
    let trimmed = cmd_line.trim();

    // Decide: direct spawn or shell fallback?
    let (program, args) = if needs_system_shell(trimmed) {
        // Has pipes, redirects, or chained commands — must use system shell
        system_shell_args(trimmed)
    } else {
        // Parse into program + args ourselves — no shell involved
        match parse_command_line(trimmed) {
            Ok((prog, a)) => (resolve_ore(prog), a),
            Err(_) => system_shell_args(trimmed),
        }
    };

    let mut child = spawn_or_fallback(&program, &args, trimmed, cwd)
        .with_context(|| format!("Failed to spawn: {} {:?}", program, args))?;

    let stdout_pipe = child.stdout.take().unwrap();
    let stderr_pipe = child.stderr.take().unwrap();

    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    let sb = Arc::clone(&stdout_buf);
    let eb = Arc::clone(&stderr_buf);

    let should_stream_out = stream && !silent;
    let should_stream_err = stream && !silent;

    let t_out = thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        for line in reader.lines().flatten() {
            if should_stream_out {
                println!("{}", line);
            }
            let mut b = sb.lock().unwrap();
            b.push_str(&line);
            b.push('\n');
        }
    });

    let t_err = thread::spawn(move || {
        let reader = BufReader::new(stderr_pipe);
        for line in reader.lines().flatten() {
            if should_stream_err {
                eprintln!("{}", line);
            }
            let mut b = eb.lock().unwrap();
            b.push_str(&line);
            b.push('\n');
        }
    });

    let status = child.wait().with_context(|| format!("Wait failed: {}", cmd_line))?;
    t_out.join().ok();
    t_err.join().ok();

    let out = Arc::try_unwrap(stdout_buf).map(|m| m.into_inner().unwrap()).unwrap_or_default();
    let err = Arc::try_unwrap(stderr_buf).map(|m| m.into_inner().unwrap()).unwrap_or_default();

    Ok(RunResult {
        exit_code: status.code().unwrap_or(-1),
        stdout: out,
        stderr: err,
        duration_ms: start.elapsed().as_millis(),
    })
}

/// Resolve a bare `ore` invocation to the currently running executable, so
/// `sequence "ore cat ..."`, agent tools, aliases and .ore scripts always run
/// THIS ore binary regardless of PATH.
fn resolve_ore(program: String) -> String {
    let lower = program.to_lowercase();
    if lower == "ore" || lower == "ore.exe" {
        std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(program)
    } else {
        program
    }
}

/// Spawn a command directly; on Windows, if the program isn't directly
/// spawnable (e.g. a .cmd/.bat shim like `npx` that only resolves via
/// cmd.exe), retry through the system shell with the full original line.
fn spawn_or_fallback(
    program: &str,
    args: &[String],
    cmd_line: &str,
    cwd: Option<&Path>,
) -> std::io::Result<std::process::Child> {
    let mut builder = Command::new(program);
    builder
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = cwd {
        builder.current_dir(d);
    }
    match builder.spawn() {
        Ok(c) => Ok(c),
        Err(e) => {
            #[cfg(windows)]
            {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Windows .cmd/.bat shims (npx, yarn, ...) aren't directly
                    // spawnable — retry via cmd.exe with the original line.
                    let (sh_prog, sh_args) = system_shell_args(cmd_line);
                    let mut b = Command::new(sh_prog);
                    b.args(&sh_args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    if let Some(d) = cwd {
                        b.current_dir(d);
                    }
                    return b.spawn();
                }
            }
            Err(e)
        }
    }
}

/// Check if a command string contains syntax that REQUIRES a system shell.
/// Only these patterns force a shell fallback:
/// - `&&` or `||` (command chaining)
/// - Unquoted `|` (pipe to another program)
/// - Unquoted `>` or `<` (redirects)
/// - cmd builtins (echo, dir, del, ...) that have no .exe
/// - Batch files (.bat/.cmd)
fn needs_system_shell(cmd: &str) -> bool {
    // Quote-aware scan for shell operators
    let mut in_dq = false;
    let mut in_sq = false;
    let bytes = cmd.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'"' && !in_sq { in_dq = !in_dq; }
        if c == b'\'' && !in_dq { in_sq = !in_sq; }
        if !in_dq && !in_sq {
            // Check for && and || first
            if c == b'&' && i + 1 < bytes.len() && bytes[i + 1] == b'&' { return true; }
            if c == b'|' && i + 1 < bytes.len() && bytes[i + 1] == b'|' { return true; }
            // Unquoted single | (pipe)
            if c == b'|' { return true; }
            // Unquoted > (redirect) and < (input redirect)
            if c == b'>' { return true; }
            if c == b'<' { return true; }
        }
        i += 1;
    }

    // Windows built-in commands that only work via cmd.exe
    #[cfg(windows)]
    {
        let first = cmd.split_whitespace().next().unwrap_or("")
            .trim_start_matches('@')
            .trim_matches('"')
            .trim_matches('\'')
            .to_lowercase();
        if matches!(first.as_str(),
            "cd" | "chdir" | "cls" | "copy" | "date" | "del" | "dir" | "echo" | "endlocal" |
            "erase" | "exit" | "for" | "ftype" | "goto" | "if" | "md" | "mkdir" | "mklink" |
            "move" | "path" | "pause" | "popd" | "prompt" | "pushd" | "rd" | "rem" | "ren" |
            "rename" | "rmdir" | "set" | "setlocal" | "setx" | "shift" | "start" | "time" |
            "title" | "type" | "ver" | "vol" | "assoc" | "where" | "more") {
            return true;
        }
        // Batch files can only be launched through cmd.exe
        if first.ends_with(".bat") || first.ends_with(".cmd") { return true; }
    }

    false
}

/// Build system shell invocation. Uses cmd.exe on Windows, sh on Unix.
#[cfg(windows)]
fn system_shell_args(cmd: &str) -> (String, Vec<String>) {
    ("cmd".to_string(), vec!["/C".to_string(), cmd.to_string()])
}

#[cfg(not(windows))]
fn system_shell_args(cmd: &str) -> (String, Vec<String>) {
    ("sh".to_string(), vec!["-c".to_string(), cmd.to_string()])
}

/// Parse a command line into (program, args) handling:
/// - Double-quoted strings (literal content, no shell expansion)
/// - Single-quoted strings (fully literal)
/// - Backslash escapes inside double quotes (\", \\, \n, \t, \r)
/// - Whitespace splitting
///
/// Outside quotes, backslashes are KEPT literal (Windows paths like
/// `C:\Users\...` must survive). Returns Err if quotes are unbalanced.
fn parse_command_line(input: &str) -> Result<(String, Vec<String>), String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() { return Err("Empty command".to_string()); }
    let mut iter = tokens.into_iter();
    let program = iter.next().unwrap();
    let args: Vec<String> = iter.collect();
    Ok((program, args))
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
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
                if let Some(&next) = chars.peek() {
                    match next {
                        '"' => { chars.next(); current.push('"'); }
                        '\\' => { chars.next(); current.push('\\'); }
                        'n' => { chars.next(); current.push('\n'); }
                        't' => { chars.next(); current.push('\t'); }
                        'r' => { chars.next(); current.push('\r'); }
                        _ => { current.push('\\'); }
                    }
                } else {
                    current.push('\\');
                }
            } else if c == '"' {
                in_double_quote = false;
            } else {
                // ALL characters are literal inside double quotes:
                // $, %, &, ^, |, (, ), <, >, #, !, `, {, }, [, ] — ALL literal
                current.push(c);
            }
            continue;
        }

        // Outside any quotes
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

    if in_double_quote { return Err("Unterminated double quote".to_string()); }
    if in_single_quote { return Err("Unterminated single quote".to_string()); }
    if !current.is_empty() { tokens.push(current); }
    Ok(tokens)
}
