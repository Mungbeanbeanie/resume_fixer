//! Job posting ingest: fetch the URL, extract the description, or fall back to paste.

pub mod extract;
pub mod fetch;

use crate::config::Ingest;
use crate::domain::{IngestResult, JobSource};
use crate::error::Result;

/// Fetches and extracts a posting.
///
/// A failure at either step is not an error the user has to act on: the result comes back
/// with `needs_paste`, and the UI shows the paste box. Only a malformed URL is rejected.
/// The failure carries its reason so the user can tell a login wall from a timeout — every
/// cause used to arrive as the same sentence.
pub async fn run(url: &str, cfg: &Ingest) -> Result<IngestResult> {
    let html = match fetch::fetch(url, cfg.timeout_secs).await {
        Ok(html) => html,
        Err(e @ crate::error::AppError::Invalid(_)) => return Err(e),
        Err(e) => {
            tracing::info!("fetch failed for {url}: {e}");
            return Ok(needs_paste(e.to_string()));
        }
    };
    match extract::extract(&html, cfg.min_chars) {
        Ok(text) => Ok(IngestResult {
            text,
            source: JobSource::Fetched,
            needs_paste: false,
            reason: None,
        }),
        Err(e) => {
            tracing::info!("extract failed for {url}: {e}");
            Ok(needs_paste(e.to_string()))
        }
    }
}

fn needs_paste(reason: String) -> IngestResult {
    IngestResult {
        text: String::new(),
        source: JobSource::Pasted,
        needs_paste: true,
        reason: Some(reason),
    }
}
