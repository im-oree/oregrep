#![allow(dead_code)] // Staged infrastructure — consumed by the Index/Database batch and future retrofits.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// A wrapper around indicatif ProgressBar with sensible ore defaults.
pub struct Progress {
    pub bar: ProgressBar,
}

impl Progress {
    /// Determinate progress with total count.
    pub fn bar(total: u64, label: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template("{prefix:.cyan} [{bar:32.green/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▊ ")
        );
        bar.set_prefix(label.to_string());
        Progress { bar }
    }

    /// Determinate progress in bytes.
    pub fn bytes(total: u64, label: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template("{prefix:.cyan} [{bar:32.green/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {eta}")
                .unwrap()
                .progress_chars("█▊ ")
        );
        bar.set_prefix(label.to_string());
        Progress { bar }
    }

    /// Indeterminate spinner.
    pub fn spinner(label: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_style(ProgressStyle::with_template("{spinner:.cyan} {prefix:.cyan} {msg}").unwrap());
        bar.set_prefix(label.to_string());
        Progress { bar }
    }

    pub fn inc(&self, n: u64) { self.bar.inc(n); }
    pub fn set(&self, n: u64) { self.bar.set_position(n); }
    pub fn message(&self, msg: &str) { self.bar.set_message(msg.to_string()); }
    pub fn finish(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }
    pub fn finish_and_clear(&self) { self.bar.finish_and_clear(); }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}
