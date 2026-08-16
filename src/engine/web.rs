use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use std::time::Duration;

/// Central browser launcher. All web commands go through this.
pub struct WebSession {
    pub browser: Browser,
}

impl WebSession {
    pub fn launch(headless: bool, window_size: Option<(u32, u32)>) -> Result<Self> {
        let mut opts_builder = LaunchOptions::default_builder();
        let opts_builder = opts_builder
            .headless(headless)
            .sandbox(false) // Windows + local usage: skip sandbox to avoid perm issues
            .idle_browser_timeout(Duration::from_secs(120));

        let opts = if let Some((w, h)) = window_size {
            opts_builder.window_size(Some((w, h))).build()
                .map_err(|e| anyhow::anyhow!("Launch options: {}", e))?
        } else {
            opts_builder.build().map_err(|e| anyhow::anyhow!("Launch options: {}", e))?
        };

        let browser = Browser::new(opts).context("Failed to launch browser. Ensure Edge or Chrome is installed.")?;
        Ok(WebSession { browser })
    }

    pub fn new_tab(&self) -> Result<Arc<Tab>> {
        Ok(self.browser.new_tab()?)
    }

    /// Open URL and wait for basic load.
    pub fn open(&self, url: &str, wait_selector: Option<&str>, timeout_secs: u64) -> Result<Arc<Tab>> {
        let tab = self.new_tab()?;
        tab.set_default_timeout(Duration::from_secs(timeout_secs));
        tab.navigate_to(url).with_context(|| format!("Navigating to {}", url))?;
        tab.wait_until_navigated().context("Waiting for navigation")?;
        if let Some(sel) = wait_selector {
            tab.wait_for_element(sel).with_context(|| format!("Waiting for {}", sel))?;
        }
        Ok(tab)
    }
}

/// Parse a "1920x1080" / "1920,1080" / "1920 1080" viewport spec.
pub fn parse_viewport(s: &str) -> Result<(u32, u32)> {
    let cleaned = s.replace('x', ",").replace(' ', ",");
    let parts: Vec<&str> = cleaned.split(',').filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 { anyhow::bail!("Viewport must be WIDTHxHEIGHT, got '{}'", s); }
    Ok((parts[0].parse()?, parts[1].parse()?))
}

/// Parse a "375,768,1440" size list into individual viewport widths.
pub fn parse_size_list(s: &str) -> Result<Vec<u32>> {
    s.split(',')
        .map(|x| x.trim().parse::<u32>().map_err(|e| anyhow::anyhow!("Bad size '{}': {}", x, e)))
        .collect()
}

/// Device presets for common screen sizes
pub fn device_viewport(name: &str) -> Option<(u32, u32)> {
    match name.to_lowercase().as_str() {
        "iphone-se" | "iphone_se" => Some((375, 667)),
        "iphone-14" | "iphone_14" => Some((390, 844)),
        "iphone-14-pro-max" => Some((430, 932)),
        "ipad" => Some((820, 1180)),
        "ipad-pro" => Some((1024, 1366)),
        "pixel-7" => Some((412, 915)),
        "galaxy-s22" => Some((360, 780)),
        "desktop" => Some((1440, 900)),
        "hd" => Some((1280, 720)),
        "fhd" | "full-hd" => Some((1920, 1080)),
        "4k" | "uhd" => Some((3840, 2160)),
        _ => None,
    }
}

pub fn fmt_bytes(n: u64) -> String {
    if n < 1024 { format!("{}B", n) }
    else if n < 1024 * 1024 { format!("{:.1}K", n as f64 / 1024.0) }
    else if n < 1024 * 1024 * 1024 { format!("{:.1}M", n as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", n as f64 / 1024.0 / 1024.0 / 1024.0) }
}
