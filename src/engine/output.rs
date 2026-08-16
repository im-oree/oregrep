#![allow(dead_code)] // Staged infrastructure — consumed by the Index/Database batch and future retrofits.

use anyhow::Result;
use clap::ValueEnum;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;

/// Global output format. Consumed by any command that wants unified `--json` / `--md` / etc.
#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub enum OutFormat {
    /// Human-readable colored (default)
    Plain,
    /// Machine-readable JSON
    Json,
    /// Markdown
    Md,
    /// Comma-separated values
    Csv,
    /// Tab-separated
    Tsv,
    /// No color, plain text (for pipes)
    Raw,
}

impl Default for OutFormat {
    fn default() -> Self { OutFormat::Plain }
}

/// Global output options bundle. Commands accept an `OutputOpts` and pass to `emit()`.
pub struct OutputOpts {
    pub format: OutFormat,
    pub to_file: Option<PathBuf>,
    pub append: bool,
    pub copy_clipboard: bool,
    pub quiet: bool,
    pub pager: bool,
    pub truncate: Option<usize>,
}

impl Default for OutputOpts {
    fn default() -> Self {
        OutputOpts {
            format: OutFormat::Plain,
            to_file: None,
            append: false,
            copy_clipboard: false,
            quiet: false,
            pager: false,
            truncate: None,
        }
    }
}

/// Emit content according to options: stdout, file, or clipboard.
pub fn emit(content: &str, opts: &OutputOpts) -> Result<()> {
    let payload = if let Some(n) = opts.truncate {
        content.lines().take(n).collect::<Vec<_>>().join("\n")
    } else {
        content.to_string()
    };

    if opts.copy_clipboard {
        copy_to_clipboard(&payload)?;
    }
    if let Some(path) = &opts.to_file {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        if opts.append {
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
            f.write_all(payload.as_bytes())?;
            if !payload.ends_with('\n') { f.write_all(b"\n")?; }
        } else {
            std::fs::write(path, &payload)?;
        }
        if !opts.quiet {
            eprintln!("Wrote: {} ({} bytes)", path.display(), payload.len());
        }
        return Ok(());
    }
    if !opts.quiet {
        print!("{}", payload);
        if !payload.ends_with('\n') { println!(); }
    }
    Ok(())
}

/// Serialize any Serialize-able value into `opts.format` and emit.
pub fn emit_value<T: Serialize>(value: &T, opts: &OutputOpts) -> Result<()> {
    let content = match opts.format {
        OutFormat::Json => serde_json::to_string_pretty(value)?,
        OutFormat::Csv | OutFormat::Tsv => value_to_delim(value, opts.format == OutFormat::Tsv)?,
        _ => serde_json::to_string_pretty(value)?, // Fall back to JSON for unstructured requests
    };
    emit(&content, opts)
}

fn value_to_delim<T: Serialize>(value: &T, tab: bool) -> Result<String> {
    let json = serde_json::to_value(value)?;
    let sep = if tab { '\t' } else { ',' };
    let mut out = String::new();
    match json {
        serde_json::Value::Array(arr) => {
            if let Some(serde_json::Value::Object(first)) = arr.first() {
                // Header from first object's keys
                let keys: Vec<String> = first.keys().cloned().collect();
                out.push_str(&keys.join(&sep.to_string()));
                out.push('\n');
                for row in arr {
                    if let serde_json::Value::Object(map) = row {
                        let vals: Vec<String> = keys.iter().map(|k| {
                            match map.get(k) {
                                Some(serde_json::Value::String(s)) => escape_csv(s, sep),
                                Some(other) => escape_csv(&other.to_string(), sep),
                                None => String::new(),
                            }
                        }).collect();
                        out.push_str(&vals.join(&sep.to_string()));
                        out.push('\n');
                    }
                }
            } else {
                for row in arr {
                    out.push_str(&row.to_string());
                    out.push('\n');
                }
            }
        }
        _ => out.push_str(&json.to_string()),
    }
    Ok(out)
}

fn escape_csv(s: &str, sep: char) -> String {
    if s.contains(sep) || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(windows)]
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("clip.exe").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}
#[cfg(target_os = "macos")]
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() { stdin.write_all(text.as_bytes())?; }
    child.wait()?;
    Ok(())
}
#[cfg(all(unix, not(target_os = "macos")))]
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn()
        .or_else(|_| Command::new("xsel").args(["-b", "-i"]).stdin(Stdio::piped()).spawn())?;
    if let Some(mut stdin) = child.stdin.take() { stdin.write_all(text.as_bytes())?; }
    child.wait()?;
    Ok(())
}
