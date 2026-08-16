use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::engine::ai::config::AiConfig;
use crate::engine::ai::events::AiEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub engine: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBundle {
    pub query: String,
    pub source: String,        // which instance served us (or "duckduckgo")
    pub results: Vec<SearchResult>,
    pub tried: Vec<String>,    // instances tried, in order
    pub failures: Vec<(String, String)>, // (instance, reason)
}

fn client(cfg: &AiConfig) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(cfg.search_timeout_secs))
        .build()?)
}

/// Search across the failover chain until one instance returns results.
pub async fn search(query: &str, cfg: &AiConfig, tx: Option<&Sender<AiEvent>>) -> Result<SearchBundle> {
    let mut bundle = SearchBundle {
        query: query.to_string(),
        source: String::new(),
        results: Vec::new(),
        tried: Vec::new(),
        failures: Vec::new(),
    };

    let c = client(cfg)?;

    // Build instance chain: configured primary → fallbacks
    let mut instances: Vec<String> = vec![cfg.search_searxng_url.trim().trim_end_matches('/').to_string()];
    for extra in cfg.search_fallback_instances.split(',') {
        let e = extra.trim().trim_end_matches('/').to_string();
        if !e.is_empty() && !instances.contains(&e) {
            instances.push(e);
        }
    }

    for instance in &instances {
        bundle.tried.push(instance.clone());
        if let Some(t) = tx {
            let _ = t.send(AiEvent::SearchingWeb { query: query.to_string(), instance: instance.clone() });
        }
        match try_searxng(&c, instance, query, cfg).await {
            Ok(results) if !results.is_empty() => {
                bundle.source = instance.clone();
                bundle.results = truncate_results(results, cfg);
                emit_found(&bundle, tx);
                return Ok(bundle);
            }
            Ok(_) => {
                let reason = "empty results".to_string();
                bundle.failures.push((instance.clone(), reason.clone()));
                if let Some(t) = tx {
                    let _ = t.send(AiEvent::SearchFailed { instance: instance.clone(), reason });
                }
            }
            Err(e) => {
                let reason = short_err(&e);
                bundle.failures.push((instance.clone(), reason.clone()));
                if let Some(t) = tx {
                    let _ = t.send(AiEvent::SearchFailed { instance: instance.clone(), reason });
                }
            }
        }
        if let (Some(t), Some(next)) = (tx, instances.iter().skip_while(|s| *s != instance).nth(1)) {
            let _ = t.send(AiEvent::SearchFallback { from: instance.clone(), to: next.clone() });
        }
    }

    // Final fallback: DuckDuckGo HTML
    let ddg = "duckduckgo".to_string();
    bundle.tried.push(ddg.clone());
    if let Some(t) = tx {
        if let Some(last) = instances.last() {
            let _ = t.send(AiEvent::SearchFallback { from: last.clone(), to: ddg.clone() });
        }
        let _ = t.send(AiEvent::SearchingWeb { query: query.to_string(), instance: ddg.clone() });
    }
    match try_duckduckgo(&c, query, cfg).await {
        Ok(results) if !results.is_empty() => {
            bundle.source = ddg;
            bundle.results = truncate_results(results, cfg);
            emit_found(&bundle, tx);
            Ok(bundle)
        }
        Ok(_) => {
            bundle.failures.push((ddg.clone(), "empty results".to_string()));
            if let Some(t) = tx {
                let _ = t.send(AiEvent::SearchFailed { instance: ddg, reason: "empty results".to_string() });
            }
            anyhow::bail!("All search backends returned no results (tried {} instances + DuckDuckGo)", bundle.tried.len().saturating_sub(1))
        }
        Err(e) => {
            let reason = short_err(&e);
            bundle.failures.push((ddg.clone(), reason.clone()));
            if let Some(t) = tx {
                let _ = t.send(AiEvent::SearchFailed { instance: ddg, reason });
            }
            anyhow::bail!("All search backends failed. Last error: {}", e)
        }
    }
}

fn truncate_results(mut results: Vec<SearchResult>, cfg: &AiConfig) -> Vec<SearchResult> {
    results.truncate(cfg.search_max_results);
    for r in &mut results {
        if r.snippet.len() > cfg.search_max_chars_per_result {
            let cut: String = r.snippet.chars().take(cfg.search_max_chars_per_result).collect();
            r.snippet = format!("{}…", cut);
        }
    }
    results
}

fn emit_found(bundle: &SearchBundle, tx: Option<&Sender<AiEvent>>) {
    if let Some(t) = tx {
        let sources: Vec<String> = bundle.results.iter().map(|r| r.url.clone()).collect();
        let _ = t.send(AiEvent::SearchFound { count: bundle.results.len(), sources });
    }
}

fn short_err(e: &anyhow::Error) -> String {
    let s = e.to_string();
    if s.len() > 140 { format!("{}…", &s[..140]) } else { s }
}

async fn try_searxng(c: &reqwest::Client, instance: &str, query: &str, _cfg: &AiConfig) -> Result<Vec<SearchResult>> {
    let url = format!("{}/search", instance);
    let resp = c.get(&url)
        .query(&[("q", query), ("format", "json"), ("safesearch", "0")])
        .send().await
        .with_context(|| format!("connecting to {}", instance))?;

    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }
    let json: serde_json::Value = resp.json().await.context("parsing JSON (instance may have JSON disabled)")?;
    let arr = json.get("results").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let out: Vec<SearchResult> = arr.iter().filter_map(|r| {
        Some(SearchResult {
            title: r.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            url: r.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            snippet: r.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            engine: r.get("engine").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }).filter(|r| !r.url.is_empty()).collect();
    Ok(out)
}

async fn try_duckduckgo(c: &reqwest::Client, query: &str, _cfg: &AiConfig) -> Result<Vec<SearchResult>> {
    let url = "https://html.duckduckgo.com/html/";
    let resp = c.post(url)
        .form(&[("q", query)])
        .header("Accept", "text/html")
        .send().await
        .context("connecting to DuckDuckGo")?;

    if !resp.status().is_success() {
        anyhow::bail!("DDG HTTP {}", resp.status());
    }
    let html = resp.text().await.context("reading DDG response")?;
    let doc = scraper::Html::parse_document(&html);
    let result_sel = scraper::Selector::parse("div.result").unwrap();
    let title_sel = scraper::Selector::parse("a.result__a").unwrap();
    let snippet_sel = scraper::Selector::parse("a.result__snippet, .result__snippet").unwrap();

    let mut out = Vec::new();
    for r in doc.select(&result_sel) {
        let title_el = r.select(&title_sel).next();
        let (title, raw_href) = match title_el {
            Some(el) => (
                el.text().collect::<String>().trim().to_string(),
                el.value().attr("href").unwrap_or("").to_string(),
            ),
            None => continue,
        };
        let snippet = r.select(&snippet_sel).next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let final_url = decode_ddg_url(&raw_href);
        if final_url.is_empty() { continue; }
        out.push(SearchResult { title, url: final_url, snippet, engine: Some("duckduckgo".to_string()) });
    }
    Ok(out)
}

fn decode_ddg_url(raw: &str) -> String {
    // DDG wraps in //duckduckgo.com/l/?uddg=<encoded>
    if raw.contains("uddg=") {
        if let Some(idx) = raw.find("uddg=") {
            let tail = &raw[idx + 5..];
            let end = tail.find('&').unwrap_or(tail.len());
            let enc = &tail[..end];
            if let Ok(decoded) = urlencoding::decode(enc) {
                return decoded.into_owned();
            }
        }
    }
    if raw.starts_with("//") { format!("https:{}", raw) } else { raw.to_string() }
}

/// Fetch a URL and strip to article-like text (removes scripts, styles, nav).
pub async fn fetch_clean(c: &reqwest::Client, url: &str, max_chars: usize) -> Result<String> {
    let resp = c.get(url).send().await.with_context(|| format!("fetching {}", url))?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }
    let html = resp.text().await.context("reading body")?;
    let text = extract_text(&html);
    let out = if text.chars().count() > max_chars {
        let cut: String = text.chars().take(max_chars).collect();
        format!("{}\n\n[…truncated to {} chars…]", cut, max_chars)
    } else { text };
    Ok(out)
}

fn extract_text(html: &str) -> String {
    let doc = scraper::Html::parse_document(html);
    let ignore = scraper::Selector::parse("script,style,noscript,svg,nav,header,footer,aside").unwrap();
    // Collect text from all nodes, skipping any whose element ancestor chain
    // (including the direct parent) matches the ignore selector.
    let mut out = String::new();
    for node in doc.tree.nodes() {
        if let Some(text_node) = node.value().as_text() {
            let mut skip = false;
            let mut anc = node.parent();
            while let Some(anc_node) = anc {
                if let Some(anc_el) = scraper::ElementRef::wrap(anc_node) {
                    if ignore.matches(&anc_el) { skip = true; break; }
                }
                anc = anc_node.parent();
            }
            if skip { continue; }
            let trimmed = text_node.trim();
            if !trimmed.is_empty() {
                out.push_str(trimmed);
                out.push(' ');
            }
        }
    }
    // Collapse whitespace runs
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}
