use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Type,
    Enum,
    Const,
    Let,
    Var,
    Struct,
    Trait,
    Impl,
    Module,
    Hook,       // React hook (function whose name starts with "use")
    Component,  // React component (Pascal-case function/const returning JSX)
    Method,
    Property,
    Other,
}

impl SymbolKind {
    pub fn short(&self) -> &'static str {
        match self {
            SymbolKind::Function => "fn",
            SymbolKind::Class => "cls",
            SymbolKind::Interface => "iface",
            SymbolKind::Type => "type",
            SymbolKind::Enum => "enum",
            SymbolKind::Const => "const",
            SymbolKind::Let => "let",
            SymbolKind::Var => "var",
            SymbolKind::Struct => "struct",
            SymbolKind::Trait => "trait",
            SymbolKind::Impl => "impl",
            SymbolKind::Module => "mod",
            SymbolKind::Hook => "hook",
            SymbolKind::Component => "comp",
            SymbolKind::Method => "method",
            SymbolKind::Property => "prop",
            SymbolKind::Other => "?",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub column: usize,
    pub exported: bool,
    pub file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub source: String,           // the module path in the "from" clause
    pub named: Vec<String>,       // named imports
    pub default: Option<String>,  // default import
    pub namespace: Option<String>, // * as Foo
    pub line: usize,
}

// Lazy regexes
static TS_EXPORT_FN: OnceLock<Regex> = OnceLock::new();
static TS_EXPORT_CONST: OnceLock<Regex> = OnceLock::new();
static TS_EXPORT_CLASS: OnceLock<Regex> = OnceLock::new();
static TS_EXPORT_INTERFACE: OnceLock<Regex> = OnceLock::new();
static TS_EXPORT_TYPE: OnceLock<Regex> = OnceLock::new();
static TS_EXPORT_ENUM: OnceLock<Regex> = OnceLock::new();
static TS_IMPORT: OnceLock<Regex> = OnceLock::new();
static RS_FN: OnceLock<Regex> = OnceLock::new();
static RS_STRUCT: OnceLock<Regex> = OnceLock::new();
static RS_ENUM: OnceLock<Regex> = OnceLock::new();
static RS_TRAIT: OnceLock<Regex> = OnceLock::new();
static RS_IMPL: OnceLock<Regex> = OnceLock::new();
static RS_TYPE: OnceLock<Regex> = OnceLock::new();
static RS_CONST: OnceLock<Regex> = OnceLock::new();
static RS_MOD: OnceLock<Regex> = OnceLock::new();
static RS_USE: OnceLock<Regex> = OnceLock::new();
static PY_DEF: OnceLock<Regex> = OnceLock::new();
static PY_CLASS: OnceLock<Regex> = OnceLock::new();
static PY_IMPORT: OnceLock<Regex> = OnceLock::new();

fn ts_export_fn() -> &'static Regex {
    TS_EXPORT_FN.get_or_init(|| Regex::new(r"(?m)^(\s*)(export\s+(?:default\s+)?(?:async\s+)?)?function\s+(\w+)").unwrap())
}
fn ts_export_const() -> &'static Regex {
    TS_EXPORT_CONST.get_or_init(|| Regex::new(r"(?m)^([ \t]*)(export\s+)?(?:const|let|var)\s+(\w+)").unwrap())
}
fn ts_export_class() -> &'static Regex {
    TS_EXPORT_CLASS.get_or_init(|| Regex::new(r"(?m)^(\s*)(export\s+(?:default\s+)?)?(?:abstract\s+)?class\s+(\w+)").unwrap())
}
fn ts_export_interface() -> &'static Regex {
    TS_EXPORT_INTERFACE.get_or_init(|| Regex::new(r"(?m)^(\s*)(export\s+)?interface\s+(\w+)").unwrap())
}
fn ts_export_type() -> &'static Regex {
    TS_EXPORT_TYPE.get_or_init(|| Regex::new(r"(?m)^(\s*)(export\s+)?type\s+(\w+)").unwrap())
}
fn ts_export_enum() -> &'static Regex {
    TS_EXPORT_ENUM.get_or_init(|| Regex::new(r"(?m)^(\s*)(export\s+(?:const\s+)?)?enum\s+(\w+)").unwrap())
}
fn ts_import() -> &'static Regex {
    TS_IMPORT.get_or_init(|| Regex::new(r#"(?m)^\s*import\s+(?:(?P<default>\w+)\s*(?:,\s*)?)?(?:\{(?P<named>[^}]+)\}\s*)?(?:\*\s+as\s+(?P<namespace>\w+)\s+)?from\s+['"](?P<source>[^'"]+)['"]"#).unwrap())
}

fn rs_fn() -> &'static Regex { RS_FN.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+(?:\([^)]+\)\s+)?)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+(\w+)").unwrap()) }
fn rs_struct() -> &'static Regex { RS_STRUCT.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+(?:\([^)]+\)\s+)?)?struct\s+(\w+)").unwrap()) }
fn rs_enum() -> &'static Regex { RS_ENUM.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+(?:\([^)]+\)\s+)?)?enum\s+(\w+)").unwrap()) }
fn rs_trait() -> &'static Regex { RS_TRAIT.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+(?:\([^)]+\)\s+)?)?trait\s+(\w+)").unwrap()) }
fn rs_impl() -> &'static Regex { RS_IMPL.get_or_init(|| Regex::new(r"(?m)^(\s*)()impl(?:<[^>]+>)?\s+(?:(\w+)(?:<[^>]+>)?\s+for\s+)?(\w+)").unwrap()) }
fn rs_type() -> &'static Regex { RS_TYPE.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+)?type\s+(\w+)").unwrap()) }
fn rs_const() -> &'static Regex { RS_CONST.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+)?(?:const|static)\s+(\w+)").unwrap()) }
fn rs_mod() -> &'static Regex { RS_MOD.get_or_init(|| Regex::new(r"(?m)^(\s*)(pub\s+)?mod\s+(\w+)").unwrap()) }
fn rs_use() -> &'static Regex { RS_USE.get_or_init(|| Regex::new(r"(?m)^\s*use\s+([^;]+);").unwrap()) }

fn py_def() -> &'static Regex { PY_DEF.get_or_init(|| Regex::new(r"(?m)^([ \t]*)(?:async\s+)?def\s+(\w+)").unwrap()) }
fn py_class() -> &'static Regex { PY_CLASS.get_or_init(|| Regex::new(r"(?m)^(\s*)class\s+(\w+)").unwrap()) }
fn py_import() -> &'static Regex { PY_IMPORT.get_or_init(|| Regex::new(r"(?m)^\s*(?:from\s+(\S+)\s+)?import\s+(.+)$").unwrap()) }

pub fn language_of(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "ts" | "tsx" => "ts",
        "js" | "jsx" | "mjs" | "cjs" => "js",
        "rs" => "rs",
        "py" => "py",
        _ => "other",
    }
}

pub fn extract_symbols(content: &str, file: &Path) -> Vec<Symbol> {
    match language_of(file) {
        "ts" | "js" => extract_symbols_ts(content, file),
        "rs" => extract_symbols_rs(content, file),
        "py" => extract_symbols_py(content, file),
        _ => vec![],
    }
}

fn line_col_of(content: &str, byte_pos: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, c) in content.char_indices() {
        if i >= byte_pos { break; }
        if c == '\n' { line += 1; col = 1; } else { col += 1; }
    }
    (line, col)
}

pub fn extract_symbols_ts(content: &str, file: &Path) -> Vec<Symbol> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Symbol>, name: &str, kind: SymbolKind, byte_pos: usize, exported: bool| {
        let (line, column) = line_col_of(content, byte_pos);
        out.push(Symbol { name: name.to_string(), kind, line, column, exported, file: file.to_path_buf() });
    };

    // Functions
    for cap in ts_export_fn().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        let name = &cap[3];
        let kind = if name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && name.starts_with("use") && name.len() > 3 {
            SymbolKind::Hook
        } else if name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
            SymbolKind::Component
        } else { SymbolKind::Function };
        push(&mut out, name, kind, cap.get(3).unwrap().start(), exported);
    }
    // Consts / lets / vars
    for cap in ts_export_const().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        let name = &cap[3];
        // Heuristic: skip local scope, only care about top-level? Regex already anchors on line-start; indent captured in group 1.
        let indent = cap.get(1).map(|m| m.as_str().len()).unwrap_or(0);
        if indent > 0 { continue; }
        let kind = if name.starts_with("use") && name.len() > 3 && name.chars().nth(3).map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
            SymbolKind::Hook
        } else if name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
            SymbolKind::Const
        } else if name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            // SCREAMING_SNAKE constants (e.g. MAX_RETRIES) are not components
            SymbolKind::Const
        } else {
            SymbolKind::Component
        };
        push(&mut out, name, kind, cap.get(3).unwrap().start(), exported);
    }
    // Classes
    for cap in ts_export_class().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Class, cap.get(3).unwrap().start(), exported);
    }
    // Interfaces
    for cap in ts_export_interface().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Interface, cap.get(3).unwrap().start(), exported);
    }
    // Types
    for cap in ts_export_type().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Type, cap.get(3).unwrap().start(), exported);
    }
    // Enums
    for cap in ts_export_enum().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Enum, cap.get(3).unwrap().start(), exported);
    }

    out.sort_by_key(|s| s.line);
    out
}

pub fn extract_symbols_rs(content: &str, file: &Path) -> Vec<Symbol> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Symbol>, name: &str, kind: SymbolKind, byte_pos: usize, exported: bool| {
        let (line, column) = line_col_of(content, byte_pos);
        out.push(Symbol { name: name.to_string(), kind, line, column, exported, file: file.to_path_buf() });
    };
    for cap in rs_fn().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Function, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_struct().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Struct, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_enum().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Enum, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_trait().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Trait, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_type().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Type, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_const().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Const, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_mod().captures_iter(content) {
        let exported = cap.get(2).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        push(&mut out, &cap[3], SymbolKind::Module, cap.get(3).unwrap().start(), exported);
    }
    for cap in rs_impl().captures_iter(content) {
        // group 3 = trait (optional), group 4 = target type
        let target = cap.get(4).map(|m| m.as_str()).unwrap_or("");
        push(&mut out, target, SymbolKind::Impl, cap.get(4).unwrap().start(), true);
    }
    out.sort_by_key(|s| s.line);
    out
}

pub fn extract_symbols_py(content: &str, file: &Path) -> Vec<Symbol> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<Symbol>, name: &str, kind: SymbolKind, byte_pos: usize| {
        let (line, column) = line_col_of(content, byte_pos);
        let exported = !name.starts_with('_');
        out.push(Symbol { name: name.to_string(), kind, line, column, exported, file: file.to_path_buf() });
    };
    for cap in py_def().captures_iter(content) {
        let indent = cap.get(1).map(|m| m.as_str().len()).unwrap_or(0);
        let kind = if indent > 0 { SymbolKind::Method } else { SymbolKind::Function };
        push(&mut out, &cap[2], kind, cap.get(2).unwrap().start());
    }
    for cap in py_class().captures_iter(content) {
        push(&mut out, &cap[2], SymbolKind::Class, cap.get(2).unwrap().start());
    }
    out.sort_by_key(|s| s.line);
    out
}

pub fn extract_imports(content: &str, file: &Path) -> Vec<Import> {
    match language_of(file) {
        "ts" | "js" => extract_imports_ts(content),
        "rs" => extract_imports_rs(content),
        "py" => extract_imports_py(content),
        _ => vec![],
    }
}

pub fn extract_imports_ts(content: &str) -> Vec<Import> {
    let mut out = Vec::new();
    for cap in ts_import().captures_iter(content) {
        let source = cap.name("source").map(|m| m.as_str().to_string()).unwrap_or_default();
        let default = cap.name("default").map(|m| m.as_str().to_string());
        let named_raw = cap.name("named").map(|m| m.as_str().to_string()).unwrap_or_default();
        let namespace = cap.name("namespace").map(|m| m.as_str().to_string());
        let named: Vec<String> = named_raw.split(',').map(|s| {
            let s = s.trim();
            // handle "orig as alias"
            if let Some((_, alias)) = s.split_once(" as ") { alias.trim().to_string() } else { s.to_string() }
        }).filter(|s| !s.is_empty()).collect();
        let byte = cap.get(0).unwrap().start();
        let (line, _) = line_col_of(content, byte);
        out.push(Import { source, named, default, namespace, line });
    }
    out
}

pub fn extract_imports_rs(content: &str) -> Vec<Import> {
    let mut out = Vec::new();
    for cap in rs_use().captures_iter(content) {
        let path = cap[1].trim().to_string();
        let byte = cap.get(0).unwrap().start();
        let (line, _) = line_col_of(content, byte);
        // Rough parse of "foo::bar::{Baz, Qux}"
        let (source, named) = if let Some((base, tail)) = path.split_once("::{") {
            let named: Vec<String> = tail.trim_end_matches('}').split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (base.to_string(), named)
        } else if let Some((base, leaf)) = path.rsplit_once("::") {
            (base.to_string(), vec![leaf.to_string()])
        } else {
            (path.clone(), vec![])
        };
        out.push(Import { source, named, default: None, namespace: None, line });
    }
    out
}

pub fn extract_imports_py(content: &str) -> Vec<Import> {
    let mut out = Vec::new();
    for cap in py_import().captures_iter(content) {
        let source = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let named_raw = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        let named: Vec<String> = named_raw.split(',').map(|s| {
            let s = s.trim();
            if let Some((_, alias)) = s.split_once(" as ") { alias.trim().to_string() } else { s.to_string() }
        }).filter(|s| !s.is_empty()).collect();
        let byte = cap.get(0).unwrap().start();
        let (line, _) = line_col_of(content, byte);
        out.push(Import { source, named, default: None, namespace: None, line });
    }
    out
}

/// Given a JS/TS relative import path from `base_file`, try to resolve it to an actual file.
pub fn resolve_ts_import(base_file: &Path, import_str: &str) -> Option<PathBuf> {
    if !import_str.starts_with('.') && !import_str.starts_with('/') { return None; }
    let base = base_file.parent()?.to_path_buf();
    let target = base.join(import_str);
    let candidates = [
        target.clone(),
        target.with_extension("ts"),
        target.with_extension("tsx"),
        target.with_extension("js"),
        target.with_extension("jsx"),
        target.with_extension("mjs"),
        target.join("index.ts"),
        target.join("index.tsx"),
        target.join("index.js"),
    ];
    for c in candidates.iter() {
        if c.is_file() { return Some(c.clone()); }
    }
    None
}

/// Extract full body of a symbol by brace-matching from its line.
/// Works for TS/JS/Rust (curly-brace languages). Returns None if symbol not found.
pub fn extract_body(content: &str, symbol_name: &str) -> Option<(usize, usize, String)> {
    // Find first line with the name in a defining position
    let patterns = [
        format!(r"(?m)^[ \t]*(?:export\s+(?:default\s+)?)?(?:async\s+)?function\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:export\s+)?(?:const|let|var)\s+{}\s*[:=]", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:export\s+(?:default\s+)?)?(?:abstract\s+)?class\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:export\s+)?interface\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:export\s+)?type\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:export\s+(?:const\s+)?)?enum\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:pub\s+)?(?:async\s+)?fn\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:pub\s+)?struct\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:pub\s+)?enum\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:pub\s+)?trait\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*(?:async\s+)?def\s+{}\b", regex::escape(symbol_name)),
        format!(r"(?m)^[ \t]*class\s+{}\b", regex::escape(symbol_name)),
    ];

    let mut start_byte: Option<usize> = None;
    for p in &patterns {
        let re = Regex::new(p).ok()?;
        if let Some(m) = re.find(content) {
            start_byte = Some(m.start());
            break;
        }
    }
    let start_byte = start_byte?;

    // Move to start of the line
    let line_start = content[..start_byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    // Walk forward, brace-match
    let bytes = content.as_bytes();
    let mut depth = 0i32;
    let mut opened = false;
    let mut end_byte = content.len();
    let mut in_string: Option<u8> = None;
    let mut i = line_start;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_string {
            if c == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if c == q { in_string = None; }
            i += 1; continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_string = Some(c); i += 1; continue; }
            b'{' => { depth += 1; opened = true; }
            b'}' => {
                depth -= 1;
                if opened && depth == 0 {
                    end_byte = i + 1;
                    // Include trailing newline
                    if end_byte < bytes.len() && bytes[end_byte] == b'\n' { end_byte += 1; }
                    break;
                }
            }
            b'\n' if !opened => {
                // Python-style: no brace, use indentation. Read next non-blank line indentation, then continue until dedent.
                // Simplified: if we hit a newline before any '{', treat this as a def-with-colon block (Python).
                // Look for the colon on same starting logical line first.
                // For now, if language looks python-ish (no braces yet and file ends in .py handled outside), fall through.
                i += 1; continue;
            }
            _ => {}
        }
        i += 1;
    }
    Some((line_start, end_byte, content[line_start..end_byte].to_string()))
}

/// Collect files under a path with our standard filters, returning (path, content).
pub fn collect_source_files(root: &Path, extensions: &[String], excludes: &[String]) -> Result<Vec<(PathBuf, String)>> {
    let cfg = crate::engine::walker::WalkConfig {
        root: root.to_path_buf(),
        extensions: extensions.to_vec(),
        excludes: excludes.to_vec(),
        skip_backups: true,
        ..Default::default()
    };
    let paths = crate::engine::walker::collect_files(&cfg)?;
    let mut out = Vec::with_capacity(paths.len());
    for p in paths {
        if let Ok(c) = crate::engine::encoding::read_file_smart(&p) {
            out.push((p, c));
        }
    }
    Ok(out)
}
