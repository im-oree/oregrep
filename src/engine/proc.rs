use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
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

/// Execute a command (via cmd.exe on Windows so shell builtins/pipes work).
/// If `stream`, output is echoed to console live AND captured.
/// If `silent`, no output is echoed.
pub fn run_cmd(cmd_line: &str, stream: bool, silent: bool) -> Result<RunResult> {
    let start = std::time::Instant::now();

    #[cfg(windows)]
    let (program, args): (&str, Vec<&str>) = ("cmd", vec!["/C", cmd_line]);
    #[cfg(not(windows))]
    let (program, args): (&str, Vec<&str>) = ("sh", vec!["-c", cmd_line]);

    let mut child = Command::new(program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn: {}", cmd_line))?;

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
