use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Semantic preview: given an intent in plain English, find files that
/// likely need to change. Extracts keywords from intent, ranks files by
/// keyword hits, shows top matches with context.
///
/// Offline mode (default): keyword extraction + weighted scoring.
/// AI mode (--ai): calls ai-ask to rank + explain (requires API key).
#[derive(Args)]
pub struct PatchPlanPreviewArgs {
    /// Describe the change you want to make
    intent: String,

    /// Path to search
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Max files to show
    #[arg(short = 'n', long, default_value = "10")]
    top: usize,

    /// Context lines around each keyword hit
    #[arg(short = 'C', long, default_value = "2")]
    context: usize,

    /// Use AI ranking (requires ai-ask configured)
    #[arg(long)]
    ai: bool,
}

pub fn run(args: PatchPlanPreviewArgs) -> Result<()> {
    println!("{}", "═══ PATCH PLAN PREVIEW ═══".cyan().bold());
    println!("  {}: {}", "Intent".dimmed(), args.intent.yellow());
    println!("  {}: {}", "Search".dimmed(), args.path.display().to_string().dimmed());
    println!();

    if args.ai {
        return run_ai_mode(&args);
    }

    run_keyword_mode(&args)
}

fn run_keyword_mode(args: &PatchPlanPreviewArgs) -> Result<()> {
    // Extract keywords from intent
    let keywords = extract_keywords(&args.intent);
    println!("  {}: {}", "Keywords".dimmed(),
        keywords.iter().map(|(w, s)| format!("{}({})", w, s)).collect::<Vec<_>>().join(", ").cyan());
    println!();

    if keywords.is_empty() {
        eprintln!("{}", "No meaningful keywords extracted from intent.".yellow());
        return Ok(());
    }

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

    // Score files by keyword hits (weighted)
    #[derive(Debug, Clone)]
    struct FileScore {
        path: PathBuf,
        score: f64,
        hits: HashMap<String, Vec<usize>>,  // keyword -> line indices
    }

    let mut scored: Vec<FileScore> = Vec::new();

    for f in &files {
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();
        let mut score = 0.0;
        let mut hits: HashMap<String, Vec<usize>> = HashMap::new();

        for (kw, weight) in &keywords {
            let re = match RegexBuilder::new(&format!(r"\b{}\b", regex::escape(kw)))
                .case_insensitive(true)
                .build() {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut kw_hits = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    kw_hits.push(i);
                }
            }
            if !kw_hits.is_empty() {
                score += (*weight as f64) * (kw_hits.len() as f64).log2().max(1.0);
                hits.insert(kw.clone(), kw_hits);
            }
        }

        // Bonus: files with MULTIPLE keywords matched score higher
        if hits.len() > 1 {
            score *= 1.0 + (hits.len() as f64 * 0.3);
        }

        if score > 0.0 {
            scored.push(FileScore { path: f.clone(), score, hits });
        }
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(args.top);

    if scored.is_empty() {
        println!("{}", "No files matched any keywords.".yellow());
        return Ok(());
    }

    println!("{}", format!("Top {} files:", scored.len()).cyan().bold());
    println!();

    for (rank, fs) in scored.iter().enumerate() {
        let content = match read_file_smart(&fs.path) { Ok(c) => c, Err(_) => continue };
        let lines: Vec<&str> = content.lines().collect();

        println!("{} {} (score: {:.1}, keywords: {})",
            format!("#{}", rank + 1).yellow().bold(),
            fs.path.display().to_string().cyan(),
            fs.score,
            fs.hits.keys().cloned().collect::<Vec<_>>().join(", ").dimmed(),
        );

        // Show a few hit contexts (max 3 per file to keep output digestible)
        let mut all_hits: Vec<(String, usize)> = fs.hits.iter()
            .flat_map(|(kw, lns)| lns.iter().map(move |l| (kw.clone(), *l)))
            .collect();
        all_hits.sort_by_key(|(_, l)| *l);
        all_hits.dedup_by_key(|(_, l)| *l);
        all_hits.truncate(3);

        for (kw, ln) in all_hits {
            let start = ln.saturating_sub(args.context);
            let end = (ln + args.context + 1).min(lines.len());
            println!("  {} {}:", "→".dimmed(), format!("line {} ({})", ln + 1, kw).green());
            for i in start..end {
                let marker = if i == ln { ">".yellow() } else { " ".normal() };
                println!("    {} {:>5} │ {}",
                    marker,
                    (i + 1).to_string().dimmed(),
                    if i == ln { lines[i].yellow().to_string() } else { lines[i].dimmed().to_string() }
                );
            }
            println!();
        }
    }

    println!();
    println!("{}", "Next steps:".cyan().bold());
    println!("  1. Review the top-ranked files above");
    println!("  2. Use `ore state <file> --at <symbol>` to inspect specific symbols");
    println!("  3. Use `ore who-calls <symbol>` to understand call sites");
    println!("  4. Write a .orepatch and apply with `patch-batch --atomic`");

    Ok(())
}

fn run_ai_mode(args: &PatchPlanPreviewArgs) -> Result<()> {
    // Delegate to ai-ask with a structured prompt
    let prompt = format!(
        "Given this intent: \"{}\"\n\n\
        Search the codebase at: {}\n\n\
        Task: List the top 5 files that likely need to change to implement this intent, \
        ranked by relevance. For each file, briefly explain WHAT needs to change. \
        Use `ore find`, `ore search-and`, `ore symbols`, `ore state --at` to explore. \
        Format your final answer as:\n\n\
        1. path/to/file.ts — brief explanation\n\
        2. ...",
        args.intent,
        args.path.display()
    );

    let ai_args = crate::commands::ai_ask::AiAskArgs {
        question: Some(prompt),
        model: None,
        no_stream: false,
        events_json: false,
        quiet: false,
        why: false,
        no_tools: false,
        auto: false,
        session: None,
        r#continue: false,
        vision: None,
    };
    crate::commands::ai_ask::run(ai_args)
}

/// Extract weighted keywords from natural language intent.
/// Weights: quoted terms (10), CamelCase/snake_case identifiers (8),
/// technical terms (5), common nouns (2). Stop words filtered.
fn extract_keywords(intent: &str) -> Vec<(String, u32)> {
    let stopwords: std::collections::HashSet<&str> = [
        "the", "a", "an", "and", "or", "but", "if", "then", "when", "where", "what",
        "how", "why", "who", "which", "that", "this", "these", "those", "is", "are",
        "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
        "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "want", "to", "of", "in", "on", "at", "by", "for", "with",
        "from", "as", "into", "onto", "up", "down", "out", "off", "over", "under",
        "not", "no", "yes", "so", "than", "also", "instead", "just", "only", "very",
        "much", "many", "more", "most", "less", "least", "any", "all", "some", "few",
        "each", "every", "same", "different", "other", "another", "such", "own",
        "use", "using", "used", "make", "makes", "made", "get", "gets", "got",
        "give", "gives", "gave", "take", "takes", "took", "come", "comes", "came",
        "go", "goes", "went", "see", "sees", "saw", "know", "knows", "knew",
        "think", "thinks", "thought", "look", "looks", "looked", "way", "ways",
    ].iter().cloned().collect();

    let mut keywords: Vec<(String, u32)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. Extract quoted terms (highest weight)
    let quoted_re = regex::Regex::new(r#""([^"]+)"|'([^']+)'"#).unwrap();
    for cap in quoted_re.captures_iter(intent) {
        let term = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str().to_string()).unwrap_or_default();
        if !term.is_empty() && seen.insert(term.to_lowercase()) {
            keywords.push((term, 10));
        }
    }

    // 2. Extract camelCase / snake_case identifiers (high weight)
    let ident_re = regex::Regex::new(r"\b([a-z]+(?:[A-Z][a-z]*)+|[a-z]+(?:_[a-z]+)+|[A-Z][a-zA-Z]*)\b").unwrap();
    for cap in ident_re.captures_iter(intent) {
        if let Some(m) = cap.get(1) {
            let word = m.as_str().to_string();
            if word.len() > 2 && seen.insert(word.to_lowercase()) {
                keywords.push((word, 8));
            }
        }
    }

    // 3. Extract single meaningful words (lower weight)
    let word_re = regex::Regex::new(r"\b([a-zA-Z]{3,})\b").unwrap();
    for cap in word_re.captures_iter(intent) {
        if let Some(m) = cap.get(1) {
            let word = m.as_str().to_string();
            let lower = word.to_lowercase();
            if stopwords.contains(lower.as_str()) { continue; }
            if seen.contains(&lower) { continue; }
            seen.insert(lower);
            keywords.push((word, 3));
        }
    }

    keywords
}
