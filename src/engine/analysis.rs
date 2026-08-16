use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_imports, extract_symbols, resolve_ts_import, Symbol};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

pub struct Graph {
    /// file → set of files it imports
    pub deps: HashMap<PathBuf, HashSet<PathBuf>>,
    /// file → set of files that import it
    pub deps_reverse: HashMap<PathBuf, HashSet<PathBuf>>,
    /// symbols per file
    pub symbols: HashMap<PathBuf, Vec<Symbol>>,
}

pub fn build_graph(root: &Path, ext_csv: Option<&str>, exc_csv: Option<&str>) -> Result<Graph> {
    let ext = ext_csv.map(parse_extensions).unwrap_or_else(||
        vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into()]);
    let exc = exc_csv.map(parse_excludes).unwrap_or_default();
    let cfg = WalkConfig {
        root: root.to_path_buf(),
        extensions: ext,
        excludes: exc,
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut deps: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut deps_reverse: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut symbols: HashMap<PathBuf, Vec<Symbol>> = HashMap::new();

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        symbols.insert(f.clone(), extract_symbols(&content, f));
        let imps = extract_imports(&content, f);
        let mut set = HashSet::new();
        for imp in &imps {
            if let Some(resolved) = resolve_ts_import(f, &imp.source) {
                let clean = clean_path(&resolved);
                set.insert(clean.clone());
                deps_reverse.entry(clean).or_default().insert(clean_path(f));
            }
        }
        deps.insert(clean_path(f), set);
    }

    Ok(Graph { deps, deps_reverse, symbols })
}

fn clean_path(p: &Path) -> PathBuf {
    match std::fs::canonicalize(p) {
        Ok(abs) => {
            let s = abs.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { abs }
        }
        Err(_) => p.to_path_buf(),
    }
}

/// Find cycles using DFS. Returns list of cycles (each is a Vec of file paths, first == last).
pub fn find_cycles(graph: &Graph) -> Vec<Vec<PathBuf>> {
    let mut cycles: Vec<Vec<PathBuf>> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut on_stack: HashSet<PathBuf> = HashSet::new();
    let mut stack: Vec<PathBuf> = Vec::new();

    let nodes: Vec<PathBuf> = graph.deps.keys().cloned().collect();
    for node in nodes {
        if !visited.contains(&node) {
            dfs_cycle(&node, graph, &mut visited, &mut on_stack, &mut stack, &mut cycles);
        }
    }
    // Dedup by cycle content (rotated forms are treated equal by min-rotated form)
    let mut seen: HashSet<String> = HashSet::new();
    let mut unique = Vec::new();
    for c in cycles {
        let key = normalize_cycle(&c);
        if seen.insert(key) { unique.push(c); }
    }
    unique
}

fn dfs_cycle(
    node: &PathBuf,
    graph: &Graph,
    visited: &mut HashSet<PathBuf>,
    on_stack: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    cycles: &mut Vec<Vec<PathBuf>>,
) {
    visited.insert(node.clone());
    on_stack.insert(node.clone());
    stack.push(node.clone());
    if let Some(children) = graph.deps.get(node) {
        for child in children {
            if !visited.contains(child) {
                dfs_cycle(child, graph, visited, on_stack, stack, cycles);
            } else if on_stack.contains(child) {
                // Found cycle: from child in stack to top
                if let Some(idx) = stack.iter().position(|n| n == child) {
                    let mut cyc: Vec<PathBuf> = stack[idx..].to_vec();
                    cyc.push(child.clone());
                    cycles.push(cyc);
                }
            }
        }
    }
    stack.pop();
    on_stack.remove(node);
}

fn normalize_cycle(cycle: &[PathBuf]) -> String {
    // Remove the trailing duplicate, rotate so smallest string is first
    let mut c: Vec<String> = cycle.iter().take(cycle.len().saturating_sub(1))
        .map(|p| p.to_string_lossy().to_string()).collect();
    if c.is_empty() { return String::new(); }
    let min_idx = (0..c.len()).min_by_key(|&i| c[i].clone()).unwrap();
    c.rotate_left(min_idx);
    c.join("→")
}

/// Cyclomatic complexity of a code chunk (rough — counts branches).
pub fn complexity_of(text: &str) -> usize {
    let mut count = 1usize;
    let patterns = ["if ", "else if", "else ", "for ", "while ", "case ",
                    " && ", " || ", " ? ", "catch ", "?.", "??"];
    for p in &patterns {
        count += text.matches(p).count();
    }
    count
}

/// Split text into per-function bodies (regex-based, TS/JS/Rust).
pub fn function_bodies(content: &str) -> Vec<(String, String, usize)> {
    // Returns (name, body, start_line)
    let mut out = Vec::new();
    let name_re = regex::Regex::new(r"(?m)^\s*(?:export\s+(?:default\s+)?)?(?:pub\s+)?(?:async\s+)?(?:function|fn)\s+(\w+)").unwrap();
    for cap in name_re.captures_iter(content) {
        let name = cap[1].to_string();
        if let Some((s, e, body)) = crate::engine::symbols::extract_body(content, &name) {
            let line = content[..s].matches('\n').count() + 1;
            let _ = e;
            out.push((name, body, line));
        }
    }
    out
}

pub fn short_path(root: &Path, p: &Path) -> String {
    let root_c = clean_path(root);
    let p_c = clean_path(p);
    match p_c.strip_prefix(&root_c) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => p.to_string_lossy().to_string(),
    }
}
