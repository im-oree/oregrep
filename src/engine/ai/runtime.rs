use anyhow::Result;

/// Build a fresh multi-thread tokio runtime scoped to a single AI command.
/// Keeps the rest of ore fully synchronous.
pub fn build_runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()?)
}
