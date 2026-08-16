use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Follow the call chain FROM an entry function, N levels deep.
/// Shows what functions are called from the entry, and what THEY call.
#[derive(Args)]
pub struct FollowArgs {
    entry: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Depth of chain expansion
    #[arg(short = 'd', long, default_value = "3")]
    depth: usize,
}

pub fn run(args: FollowArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: false,
        respect_gitignore: true,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;

    // Cache all file contents
    let file_contents: Vec<(PathBuf, String)> = files.iter()
        .filter_map(|f| read_file_smart(f).ok().map(|c| (f.clone(), c)))
        .collect();

    println!("{}", format!("═══ FOLLOW: {} (depth {}) ═══", args.entry, args.depth).cyan().bold());

    let mut visited: HashSet<String> = HashSet::new();
    expand(&args.entry, &file_contents, args.depth, 0, &mut visited);

    eprintln!("\n{} {} unique functions in call tree",
        "follow:".bold(),
        visited.len().to_string().yellow()
    );

    Ok(())
}

fn expand(
    fn_name: &str,
    files: &[(PathBuf, String)],
    max_depth: usize,
    current_depth: usize,
    visited: &mut HashSet<String>,
) {
    if current_depth >= max_depth { return; }
    if visited.contains(fn_name) { return; }
    visited.insert(fn_name.to_string());

    let indent = "  ".repeat(current_depth);
    println!("{}{} {}", indent, "→".cyan(), fn_name.yellow().bold());

    // Find the function definition start (regex crate has no look-around,
    // so the body is extracted afterwards via brace matching).
    let def_pattern = format!(
        r"(?m)^[ \t]*(?:pub\s+)?(?:export\s+)?(?:async\s+)?(?:function|fn|def|const|let|var)\s+{}\b",
        regex::escape(fn_name)
    );
    let def_re = match RegexBuilder::new(&def_pattern).build() {
        Ok(r) => r,
        Err(_) => return,
    };

    let mut body: Option<String> = None;
    for (_f, content) in files {
        if let Some(m) = def_re.find(content) {
            body = Some(extract_body(content, m.start()));
            break;
        }
    }

    let body = match body {
        Some(b) => b,
        None => {
            println!("{}  {}", indent, "(not found)".dimmed());
            return;
        }
    };

    // Extract called functions from the body
    let call_re = RegexBuilder::new(r"\b([a-zA-Z_][a-zA-Z0-9_]{2,})\s*\(").build().unwrap();
    let mut called: Vec<String> = Vec::new();
    for cap in call_re.captures_iter(&body) {
        let name = cap[1].to_string();
        // Filter noise: skip keywords and common built-ins
        if is_keyword_or_builtin(&name) { continue; }
        if name == fn_name { continue; }
        if !called.contains(&name) { called.push(name); }
    }

    for c in called {
        expand(&c, files, max_depth, current_depth + 1, visited);
    }
}

/// Extract a function body from a definition start using brace matching,
/// skipping strings and comments. Falls back to the rest of the file (Python
/// style, brace-less) if no braces are found.
fn extract_body(content: &str, start: usize) -> String {
    let bytes = content.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'{' { i += 1; }
    if i >= bytes.len() { return content[start..].to_string(); }

    let mut depth = 0;
    let mut in_str = false;
    let mut str_ch: u8 = 0;
    let mut in_lc = false;
    let mut in_bc = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if c == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if c == str_ch { in_str = false; }
            i += 1;
            continue;
        }
        if in_lc {
            if c == b'\n' { in_lc = false; }
            i += 1;
            continue;
        }
        if in_bc {
            if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_bc = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' { in_str = true; str_ch = c; i += 1; continue; }
        if c == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' { in_lc = true; i += 2; continue; }
            if bytes[i + 1] == b'*' { in_bc = true; i += 2; continue; }
        }
        if c == b'{' { depth += 1; }
        else if c == b'}' {
            depth -= 1;
            if depth == 0 { return content[start..=i].to_string(); }
        }
        i += 1;
    }
    content[start..].to_string()
}

fn is_keyword_or_builtin(name: &str) -> bool {
    matches!(name,
        "if" | "for" | "while" | "return" | "let" | "const" | "var" | "function" | "async" |
        "await" | "new" | "typeof" | "instanceof" | "throw" | "catch" | "try" | "switch" |
        "case" | "break" | "continue" | "delete" | "in" | "of" | "yield" | "class" | "extends" |
        "super" | "this" | "true" | "false" | "null" | "undefined" |
        "console" | "Math" | "Number" | "String" | "Array" | "Object" | "Boolean" |
        "JSON" | "Promise" | "Set" | "Map" | "Date" | "RegExp" | "Error" |
        "print" | "println" | "eprintln" | "format" | "vec" |
        "pub" | "fn" | "impl" | "trait" | "struct" | "enum" | "match" | "mod" | "use" |
        "def" | "self" | "cls" | "import" | "from" | "as" | "with"
    )
}
