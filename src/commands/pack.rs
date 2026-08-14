use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct PackArgs {
    /// Glob patterns or directories to include
    inputs: Vec<PathBuf>,

    /// Extensions to include
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Excludes
    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Output format
    #[arg(short = 'f', long, default_value = "md")]
    format: PackFormat,

    /// Write to file instead of stdout
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Also copy to clipboard (Windows only for now)
    #[arg(long)]
    copy: bool,

    /// Truncate each file to N lines (0 = no limit)
    #[arg(long, default_value = "0")]
    max_lines_per_file: usize,

    /// Skip blank lines (saves tokens)
    #[arg(long)]
    strip_blanks: bool,

    /// Strip // and # comment lines (saves tokens)
    #[arg(long)]
    strip_comments: bool,

    /// Prepend the directory tree
    #[arg(long)]
    include_tree: bool,

    /// Include hidden
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Ignore .gitignore
    #[arg(long)]
    no_ignore: bool,

    /// Include binary files
    #[arg(long)]
    binary: bool,

    /// Suppress line numbers
    #[arg(short = 'N', long)]
    no_numbers: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PackFormat {
    Md,
    Xml,
    Tag,
    Plain,
}

pub fn run(args: PackArgs) -> Result<()> {
    if args.inputs.is_empty() {
        anyhow::bail!("Provide at least one path or glob");
    }

    // Collect files from all inputs
    let mut all_files: Vec<PathBuf> = Vec::new();
    for input in &args.inputs {
        if input.is_file() {
            all_files.push(input.clone());
        } else if input.is_dir() {
            let cfg = WalkConfig {
                root: input.clone(),
                extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
                excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
                hidden: args.hidden,
                respect_gitignore: !args.no_ignore,
                include_binary: args.binary,
                skip_backups: true,
            };
            let files = collect_files(&cfg)?;
            all_files.extend(files);
        } else {
            // Try glob
            let pattern = input.to_string_lossy();
            match glob::glob(&pattern) {
                Ok(paths) => {
                    for p in paths.flatten() {
                        if p.is_file() {
                            all_files.push(p);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("{} {}", "SKIP".yellow(), input.display());
                }
            }
        }
    }

    all_files.sort();
    all_files.dedup();

    let mut out = String::new();

    // Tree header
    if args.include_tree {
        out.push_str("# Directory Tree\n\n```\n");
        for f in &all_files {
            out.push_str(&format!("{}\n", f.display()));
        }
        out.push_str("```\n\n");
    }

    for path in &all_files {
        let content = match read_file_smart(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let processed = process_content(&content, &args);

        match args.format {
            PackFormat::Md => {
                let lang = detect_lang(path);
                out.push_str(&format!("## {}\n\n```{}\n", path.display(), lang));
                push_lines(&mut out, &processed, args.no_numbers);
                out.push_str("```\n\n");
            }
            PackFormat::Xml => {
                out.push_str(&format!("<file path=\"{}\">\n", path.display()));
                push_lines(&mut out, &processed, args.no_numbers);
                out.push_str("</file>\n\n");
            }
            PackFormat::Tag => {
                out.push_str(&format!("=== {} ===\n", path.display()));
                push_lines(&mut out, &processed, args.no_numbers);
                out.push_str("=== END ===\n\n");
            }
            PackFormat::Plain => {
                push_lines(&mut out, &processed, args.no_numbers);
                out.push('\n');
            }
        }
    }

    let file_count = all_files.len();
    let byte_count = out.len();

    if let Some(o) = &args.output {
        std::fs::write(o, &out)?;
        println!("{} {}  ({} files, {} bytes)",
            "Wrote:".green().bold(),
            o.display().to_string().cyan(),
            file_count.to_string().yellow(),
            byte_count.to_string().yellow()
        );
    } else if args.copy {
        // Print to stdout too if user didn't specify output
        print!("{}", out);
    } else {
        print!("{}", out);
    }

    if args.copy {
        #[cfg(windows)]
        {
            if let Err(e) = copy_to_clipboard(&out) {
                eprintln!("{} Could not copy to clipboard: {}", "WARN".yellow(), e);
            } else {
                eprintln!("{} Copied to clipboard ({} bytes)", "OK".green(), byte_count);
            }
        }
        #[cfg(not(windows))]
        {
            eprintln!("{} Clipboard not supported on this platform yet", "WARN".yellow());
        }
    }

    Ok(())
}

fn process_content(content: &str, args: &PackArgs) -> String {
    let mut lines: Vec<&str> = content.lines().collect();

    if args.strip_blanks {
        lines.retain(|l| !l.trim().is_empty());
    }
    if args.strip_comments {
        lines.retain(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with('#') && !t.starts_with("/*")
        });
    }
    if args.max_lines_per_file > 0 && lines.len() > args.max_lines_per_file {
        lines.truncate(args.max_lines_per_file);
        lines.push("// ... [truncated by ore pack]");
    }
    lines.join("\n")
}

fn push_lines(out: &mut String, content: &str, no_numbers: bool) {
    if no_numbers {
        out.push_str(content);
        if !content.ends_with('\n') {
            out.push('\n');
        }
    } else {
        for (i, line) in content.lines().enumerate() {
            out.push_str(&format!("{:>5} | {}\n", i + 1, line));
        }
    }
}

fn detect_lang(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "ts" | "tsx" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "rs" => "rust",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" | "hh" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "sh" | "bash" => "bash",
        "ps1" | "psm1" => "powershell",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" => "markdown",
        "sql" => "sql",
        _ => "",
    }
}

#[cfg(windows)]
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    let mut child = Command::new("clip.exe")
        .stdin(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}
