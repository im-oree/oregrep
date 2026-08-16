use anyhow::{Context, Result};
use serde_json::Value;

/// Navigate a JSON Value by a path like "foo.bar[0].baz" or "foo/bar/0/baz"
pub fn nav_get<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() { return Some(v); }
    let parts = tokenize_path(path);
    let mut cur = v;
    for p in parts {
        match cur {
            Value::Object(m) => cur = m.get(&p)?,
            Value::Array(a) => {
                let idx: usize = p.parse().ok()?;
                cur = a.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

/// Set a value at a path, creating intermediate objects as needed.
pub fn nav_set(v: &mut Value, path: &str, new_val: Value) -> Result<()> {
    if path.is_empty() { *v = new_val; return Ok(()); }
    let parts = tokenize_path(path);
    set_recurse(v, &parts, 0, new_val)
}

fn set_recurse(v: &mut Value, parts: &[String], idx: usize, new_val: Value) -> Result<()> {
    if idx == parts.len() - 1 {
        let key = &parts[idx];
        match v {
            Value::Object(m) => { m.insert(key.clone(), new_val); Ok(()) }
            Value::Array(a) => {
                let i: usize = key.parse().context("Array index must be integer")?;
                if i >= a.len() { a.resize(i + 1, Value::Null); }
                a[i] = new_val;
                Ok(())
            }
            _ => anyhow::bail!("Cannot set '{}' on non-container", key),
        }
    } else {
        let key = &parts[idx];
        let next = match v {
            Value::Object(m) => m.entry(key.clone()).or_insert(Value::Object(serde_json::Map::new())),
            Value::Array(a) => {
                let i: usize = key.parse().context("Array index must be integer")?;
                if i >= a.len() { a.resize(i + 1, Value::Object(serde_json::Map::new())); }
                &mut a[i]
            }
            _ => anyhow::bail!("Cannot traverse '{}' on non-container", key),
        };
        set_recurse(next, parts, idx + 1, new_val)
    }
}

/// Parse "foo.bar[0].baz" or "foo/bar/0/baz" into ["foo","bar","0","baz"]
fn tokenize_path(path: &str) -> Vec<String> {
    let normalized = path.replace('/', ".");
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in normalized.chars() {
        match c {
            '.' => { if !cur.is_empty() { out.push(std::mem::take(&mut cur)); } }
            '[' => { if !cur.is_empty() { out.push(std::mem::take(&mut cur)); } }
            ']' => { if !cur.is_empty() { out.push(std::mem::take(&mut cur)); } }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}

/// Merge b into a. Objects deep-merge, arrays concat (or replace if `replace_arrays`),
/// scalars overwrite.
pub fn deep_merge(a: &mut Value, b: Value, replace_arrays: bool) {
    match (a, b) {
        (Value::Object(am), Value::Object(bm)) => {
            for (k, bv) in bm {
                match am.get_mut(&k) {
                    Some(av) => deep_merge(av, bv, replace_arrays),
                    None => { am.insert(k, bv); }
                }
            }
        }
        (Value::Array(aa), Value::Array(ba)) => {
            if replace_arrays { *aa = ba; }
            else { aa.extend(ba); }
        }
        (a, b) => { *a = b; }
    }
}

/// Try to parse a CLI value string as JSON literal; fall back to string.
/// Examples: 42 → number, true → bool, "hello" → string (keeps quotes), hello → string
pub fn parse_cli_value(s: &str) -> Value {
    // Try JSON first (numbers, bools, null, arrays, objects, quoted strings)
    if let Ok(v) = serde_json::from_str::<Value>(s) { return v; }
    // Fallback: bare string
    Value::String(s.to_string())
}

/// Print a JSON Value as a scalar-friendly string.
/// Objects/arrays become pretty JSON. Scalars become raw text.
pub fn value_to_display(v: &Value, pretty: bool) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => if pretty { serde_json::to_string_pretty(v).unwrap_or_default() }
             else { serde_json::to_string(v).unwrap_or_default() },
    }
}
