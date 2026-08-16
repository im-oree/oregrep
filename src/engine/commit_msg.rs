use anyhow::Result;
use std::collections::HashMap;

use crate::engine::git::git;

#[derive(Debug, Clone)]
pub struct FileChange {
    pub status: String,     // "A" added, "M" modified, "D" deleted, "R" renamed
    pub path: String,
    pub added: usize,
    pub removed: usize,
}

#[derive(Debug, Clone)]
pub struct DiffAnalysis {
    pub files: Vec<FileChange>,
    pub total_added: usize,
    pub total_removed: usize,
    pub buckets: HashMap<String, Vec<String>>,  // category -> file paths
    pub new_symbols: Vec<String>,               // exported symbols added
    pub removed_symbols: Vec<String>,           // exported symbols removed
    pub new_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub renamed_files: Vec<(String, String)>,
    pub is_config_only: bool,
    pub is_test_only: bool,
    pub is_docs_only: bool,
    pub is_deps_change: bool,
    pub touches_readme: bool,
}

pub fn analyze_diff(staged: bool) -> Result<DiffAnalysis> {
    // Use --numstat to get add/remove counts
    let args: Vec<&str> = if staged {
        vec!["diff", "--cached", "--numstat"]
    } else {
        vec!["diff", "--numstat", "HEAD"]
    };
    let numstat = git(&args)?;

    let name_args: Vec<&str> = if staged {
        vec!["diff", "--cached", "--name-status"]
    } else {
        vec!["diff", "--name-status", "HEAD"]
    };
    let namestat = git(&name_args)?;

    let mut files: HashMap<String, FileChange> = HashMap::new();
    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 { continue; }
        let added: usize = parts[0].parse().unwrap_or(0);
        let removed: usize = parts[1].parse().unwrap_or(0);
        let path = parts[2].to_string();
        files.insert(path.clone(), FileChange { status: "M".to_string(), path, added, removed });
    }
    for line in namestat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() { continue; }
        let status = parts[0].to_string();
        let path = if status.starts_with('R') && parts.len() >= 3 {
            parts[2].to_string()
        } else if parts.len() >= 2 {
            parts[1].to_string()
        } else { continue };
        if let Some(fc) = files.get_mut(&path) {
            fc.status = status.chars().next().unwrap_or('M').to_string();
        } else {
            files.insert(path.clone(), FileChange {
                status: status.chars().next().unwrap_or('M').to_string(),
                path: path.clone(),
                added: 0,
                removed: 0,
            });
        }
    }

    let mut all_files: Vec<FileChange> = files.into_values().collect();
    all_files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut a = DiffAnalysis {
        files: all_files.clone(),
        total_added: all_files.iter().map(|f| f.added).sum(),
        total_removed: all_files.iter().map(|f| f.removed).sum(),
        buckets: HashMap::new(),
        new_symbols: vec![],
        removed_symbols: vec![],
        new_files: vec![],
        deleted_files: vec![],
        renamed_files: vec![],
        is_config_only: true,
        is_test_only: true,
        is_docs_only: true,
        is_deps_change: false,
        touches_readme: false,
    };

    for f in &all_files {
        let cat = categorize(&f.path);
        a.buckets.entry(cat.to_string()).or_default().push(f.path.clone());

        if f.status == "A" { a.new_files.push(f.path.clone()); }
        if f.status == "D" { a.deleted_files.push(f.path.clone()); }

        let is_test = f.path.contains(".test.") || f.path.contains(".spec.") || f.path.contains("__tests__") || f.path.starts_with("test/");
        let is_docs = f.path.to_lowercase().ends_with(".md") || f.path.starts_with("docs/");
        let is_config = matches!(cat, "config");
        let is_deps = f.path == "package.json" || f.path == "package-lock.json"
                   || f.path == "Cargo.toml" || f.path == "Cargo.lock"
                   || f.path == "yarn.lock" || f.path == "pnpm-lock.yaml"
                   || f.path == "requirements.txt" || f.path == "poetry.lock"
                   || f.path == "Pipfile" || f.path == "Pipfile.lock";

        if !is_test { a.is_test_only = false; }
        if !is_docs { a.is_docs_only = false; }
        if !is_config { a.is_config_only = false; }
        if is_deps { a.is_deps_change = true; }
        if f.path.to_lowercase().starts_with("readme") { a.touches_readme = true; }
    }
    if all_files.is_empty() {
        a.is_test_only = false;
        a.is_docs_only = false;
        a.is_config_only = false;
    }

    // Diff patch text for symbol extraction
    let patch_args: Vec<&str> = if staged {
        vec!["diff", "--cached", "-U0"]
    } else {
        vec!["diff", "-U0", "HEAD"]
    };
    if let Ok(patch) = git(&patch_args) {
        let (news, olds) = extract_symbols_from_patch(&patch);
        a.new_symbols = news;
        a.removed_symbols = olds;
    }

    Ok(a)
}

fn categorize(path: &str) -> &'static str {
    let p = path.to_lowercase();
    if p.contains("/components/") || p.contains("\\components\\") { "component" }
    else if p.contains("/hooks/") || p.contains("\\hooks\\") || p.contains("/use") { "hook" }
    else if p.contains("/store") || p.contains("\\store") { "store" }
    else if p.contains("/routes/") || p.contains("\\routes\\") { "route" }
    else if p.contains("/pages/") || p.contains("\\pages\\") { "page" }
    else if p.contains("/lib/") || p.contains("\\lib\\") { "lib" }
    else if p.contains("/util") || p.contains("\\util") || p.contains("/helper") { "util" }
    else if p.contains(".test.") || p.contains(".spec.") || p.contains("__tests__") { "test" }
    else if p.contains("/services/") || p.contains("\\services\\") { "service" }
    else if p.contains("/api/") || p.contains("\\api\\") { "api" }
    else if p.ends_with(".md") || p.contains("/docs/") { "docs" }
    else if p == "package.json" || p == "cargo.toml" || p == "cargo.lock" || p == "package-lock.json"
         || p == "yarn.lock" || p == "pnpm-lock.yaml" { "deps" }
    else if p.ends_with(".json") || p.ends_with(".toml") || p.ends_with(".yaml") || p.ends_with(".yml")
         || p.starts_with(".env") || p.starts_with(".git") || p == ".gitignore"
         || p.contains("config") || p.ends_with("rc") { "config" }
    else if p.contains("/commands/") || p.contains("\\commands\\") { "command" }
    else if p.contains("/engine/") || p.contains("\\engine\\") { "engine" }
    else if p.ends_with(".css") || p.ends_with(".scss") { "style" }
    else if p.ends_with(".ts") || p.ends_with(".tsx") || p.ends_with(".js") || p.ends_with(".jsx") || p.ends_with(".rs") || p.ends_with(".py") { "code" }
    else { "other" }
}

fn extract_symbols_from_patch(patch: &str) -> (Vec<String>, Vec<String>) {
    // Look for added/removed lines that define exports
    let add_re = regex::Regex::new(r"^\+\s*(?:export\s+(?:default\s+)?(?:async\s+)?)?(?:function|const|class|interface|type|enum)\s+(\w+)").unwrap();
    let rm_re = regex::Regex::new(r"^-\s*(?:export\s+(?:default\s+)?(?:async\s+)?)?(?:function|const|class|interface|type|enum)\s+(\w+)").unwrap();
    let rs_add_re = regex::Regex::new(r"^\+\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type|mod)\s+(\w+)").unwrap();
    let rs_rm_re = regex::Regex::new(r"^-\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|type|mod)\s+(\w+)").unwrap();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    for line in patch.lines() {
        if let Some(cap) = add_re.captures(line).or_else(|| rs_add_re.captures(line)) {
            let n = cap[1].to_string();
            if !added.contains(&n) { added.push(n); }
        }
        if let Some(cap) = rm_re.captures(line).or_else(|| rs_rm_re.captures(line)) {
            let n = cap[1].to_string();
            if !removed.contains(&n) { removed.push(n); }
        }
    }
    (added, removed)
}

pub fn detect_convention() -> String {
    // Look at last 20 commits — if >50% start with `feat:` / `fix:` / etc, use conventional
    let out = git(&["log", "-n", "20", "--pretty=format:%s"]).unwrap_or_default();
    let re = regex::Regex::new(r"^(feat|fix|chore|docs|style|refactor|test|perf|build|ci|revert)(\([^)]+\))?!?:").unwrap();
    let total = out.lines().count();
    let matches = out.lines().filter(|l| re.is_match(l)).count();
    if total > 0 && matches * 2 >= total { "conventional".to_string() } else { "simple".to_string() }
}

pub fn compose_message(a: &DiffAnalysis, style: &str, with_body: bool) -> String {
    let subject = compose_subject(a, style);
    if !with_body { return subject; }
    let body = compose_body(a);
    if body.is_empty() { subject } else { format!("{}\n\n{}", subject, body) }
}

fn compose_subject(a: &DiffAnalysis, style: &str) -> String {
    // Detect kind
    let (kind, scope) = infer_kind_scope(a);
    let action = infer_action(a);
    let what = infer_what(a);

    match style {
        "conventional" => {
            let scope_str = if !scope.is_empty() { format!("({})", scope) } else { String::new() };
            format!("{}{}: {} {}", kind, scope_str, action, what).trim_end().to_string()
        }
        _ => {
            // Simple English
            if scope.is_empty() { format!("{} {}", capitalize(&action), what) }
            else { format!("{} {} in {}", capitalize(&action), what, scope) }
        }
    }
}

fn compose_body(a: &DiffAnalysis) -> String {
    let mut lines = Vec::new();

    // Per-bucket bullets
    let mut buckets: Vec<(&String, &Vec<String>)> = a.buckets.iter().collect();
    buckets.sort_by(|x, y| y.1.len().cmp(&x.1.len()));

    for (cat, files) in buckets.iter().take(6) {
        if files.is_empty() { continue; }
        let sample: Vec<String> = files.iter().take(3).map(|p| short_name(p)).collect();
        let more = if files.len() > 3 { format!(" (+{} more)", files.len() - 3) } else { String::new() };
        lines.push(format!("- {}: {}{}", cat, sample.join(", "), more));
    }

    // New symbols
    if !a.new_symbols.is_empty() {
        let sym_str: Vec<String> = a.new_symbols.iter().take(6).map(|s| s.clone()).collect();
        let more = if a.new_symbols.len() > 6 { format!(" (+{} more)", a.new_symbols.len() - 6) } else { String::new() };
        lines.push(format!("- new symbols: {}{}", sym_str.join(", "), more));
    }
    if !a.removed_symbols.is_empty() {
        let sym_str: Vec<String> = a.removed_symbols.iter().take(6).map(|s| s.clone()).collect();
        let more = if a.removed_symbols.len() > 6 { format!(" (+{} more)", a.removed_symbols.len() - 6) } else { String::new() };
        lines.push(format!("- removed symbols: {}{}", sym_str.join(", "), more));
    }
    if !a.renamed_files.is_empty() {
        for (o, n) in a.renamed_files.iter().take(3) {
            lines.push(format!("- renamed: {} → {}", short_name(o), short_name(n)));
        }
    }

    // Stats footer
    lines.push(String::new());
    lines.push(format!("{} files changed, +{} -{}", a.files.len(), a.total_added, a.total_removed));

    lines.join("\n")
}

fn infer_kind_scope(a: &DiffAnalysis) -> (String, String) {
    if a.is_docs_only { return ("docs".to_string(), String::new()); }
    if a.is_test_only { return ("test".to_string(), String::new()); }
    if a.is_deps_change && a.files.len() <= 3 { return ("chore".to_string(), "deps".to_string()); }
    if a.is_config_only { return ("chore".to_string(), "config".to_string()); }

    // Kind
    let kind = if !a.new_symbols.is_empty() && a.new_files.len() >= a.deleted_files.len() { "feat" }
        else if a.total_removed > a.total_added * 2 { "refactor" }
        else if a.deleted_files.len() > a.new_files.len() { "chore" }
        else if a.total_removed > 0 && a.total_added > 0 { "refactor" }
        else if !a.new_files.is_empty() { "feat" }
        else { "fix" };

    // Scope from most common bucket
    let mut buckets: Vec<(&String, &Vec<String>)> = a.buckets.iter().collect();
    buckets.sort_by(|x, y| y.1.len().cmp(&x.1.len()));
    let scope = buckets.first().map(|(k, _)| (*k).clone()).unwrap_or_default();
    let scope = if scope == "other" || scope == "code" { String::new() } else { scope };
    (kind.to_string(), scope)
}

fn infer_action(a: &DiffAnalysis) -> String {
    if !a.deleted_files.is_empty() && a.new_files.is_empty() && a.new_symbols.is_empty() { "remove".to_string() }
    else if !a.new_files.is_empty() && a.deleted_files.is_empty() && !a.new_symbols.is_empty() { "add".to_string() }
    else if a.total_removed > a.total_added * 2 { "clean up".to_string() }
    else if !a.new_symbols.is_empty() && !a.removed_symbols.is_empty() { "rework".to_string() }
    else if !a.new_symbols.is_empty() { "add".to_string() }
    else if a.total_added > a.total_removed * 3 { "extend".to_string() }
    else { "update".to_string() }
}

fn infer_what(a: &DiffAnalysis) -> String {
    // Prefer symbol names when small set
    if !a.new_symbols.is_empty() && a.new_symbols.len() <= 3 {
        return a.new_symbols.join(", ");
    }
    if a.new_symbols.len() > 3 {
        return format!("{} symbols", a.new_symbols.len());
    }
    if !a.removed_symbols.is_empty() && a.removed_symbols.len() <= 3 {
        return format!("{} (removed)", a.removed_symbols.join(", "));
    }
    // Use bucket names
    let mut buckets: Vec<(&String, &Vec<String>)> = a.buckets.iter().collect();
    buckets.sort_by(|x, y| y.1.len().cmp(&x.1.len()));
    match buckets.first() {
        Some((cat, files)) if files.len() == 1 => short_name(&files[0]),
        Some((cat, files)) => format!("{} {}{}", files.len(), cat, if files.len() > 1 { "s" } else { "" }),
        None => "no-op".to_string(),
    }
}

fn short_name(path: &str) -> String {
    let p = path.replace('\\', "/");
    let last = p.rsplit('/').next().unwrap_or(&p);
    // Strip common extensions
    let stem = last.rsplitn(2, '.').last().unwrap_or(last);
    stem.to_string()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
