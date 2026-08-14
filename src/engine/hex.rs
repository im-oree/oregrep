use anyhow::Result;

/// Parse a hex pattern string like "de ad be ef" or "deadbeef" or "0xDEADBEEF" into bytes.
/// Also supports wildcards: "de ?? be ef" — `??` means "any byte" (returned as None).
/// If any wildcards are present, returns Err — use `parse_hex_pattern` for wildcard support.
pub fn parse_hex_bytes(input: &str) -> Result<Vec<u8>> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let cleaned = cleaned.strip_prefix("0x").or_else(|| cleaned.strip_prefix("0X")).unwrap_or(&cleaned).to_string();
    if cleaned.len() % 2 != 0 {
        anyhow::bail!("Hex string must have even length: {} (got {} chars)", input, cleaned.len());
    }
    let mut out = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let byte = u8::from_str_radix(&cleaned[i..i + 2], 16)
            .map_err(|_| anyhow::anyhow!("Invalid hex byte: {}", &cleaned[i..i + 2]))?;
        out.push(byte);
    }
    Ok(out)
}

/// Parse hex pattern with wildcard support. `??` = any byte.
/// Returns Vec<Option<u8>>. Some(b) = literal, None = wildcard.
pub fn parse_hex_pattern(input: &str) -> Result<Vec<Option<u8>>> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let cleaned = cleaned.strip_prefix("0x").or_else(|| cleaned.strip_prefix("0X")).unwrap_or(&cleaned).to_string();
    if cleaned.len() % 2 != 0 {
        anyhow::bail!("Hex pattern must have even length: {}", input);
    }
    let mut out = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let pair = &cleaned[i..i + 2];
        if pair == "??" || pair == "**" {
            out.push(None);
        } else {
            let b = u8::from_str_radix(pair, 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex byte in pattern: {}", pair))?;
            out.push(Some(b));
        }
    }
    Ok(out)
}

/// Find all positions where `pattern` matches in `haystack`. Supports wildcards.
pub fn find_all(haystack: &[u8], pattern: &[Option<u8>]) -> Vec<usize> {
    let mut matches = Vec::new();
    if pattern.is_empty() || haystack.len() < pattern.len() { return matches; }
    'outer: for i in 0..=haystack.len() - pattern.len() {
        for (j, pb) in pattern.iter().enumerate() {
            if let Some(b) = pb {
                if haystack[i + j] != *b { continue 'outer; }
            }
        }
        matches.push(i);
    }
    matches
}

/// Format bytes as `xxd`-style hex dump.
/// offset  hex bytes                                       ascii
pub fn format_hex_dump(bytes: &[u8], start_offset: usize, width: usize) -> String {
    let mut out = String::new();
    let width = width.max(1);
    for (i, chunk) in bytes.chunks(width).enumerate() {
        let offset = start_offset + i * width;
        out.push_str(&format!("{:08x}: ", offset));
        // Hex bytes
        for (j, b) in chunk.iter().enumerate() {
            out.push_str(&format!("{:02x}", b));
            if j % 2 == 1 { out.push(' '); }
        }
        // Pad if short
        let filled = chunk.len() * 2 + chunk.len() / 2;
        let target = width * 2 + width / 2;
        for _ in filled..target { out.push(' '); }
        out.push(' ');
        // ASCII column
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

/// Parse an offset argument: "1024", "0x400", "1k", "2m", "1g"
pub fn parse_offset(s: &str) -> Result<u64> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).map_err(|e| anyhow::anyhow!("Bad hex offset '{}': {}", s, e));
    }
    let (num, mul) = if let Some(n) = s.strip_suffix(|c: char| c == 'k' || c == 'K') { (n, 1024u64) }
        else if let Some(n) = s.strip_suffix(|c: char| c == 'm' || c == 'M') { (n, 1024 * 1024) }
        else if let Some(n) = s.strip_suffix(|c: char| c == 'g' || c == 'G') { (n, 1024 * 1024 * 1024) }
        else { (s, 1u64) };
    let val: u64 = num.parse().map_err(|e| anyhow::anyhow!("Bad offset '{}': {}", s, e))?;
    Ok(val * mul)
}
