//! The one outbound request this app makes: the job posting the user typed.

use crate::error::{AppError, Result};
use std::time::Duration;

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/124.0 Safari/537.36";

/// Fetches a posting page as HTML.
///
/// Rejects anything that is not an `http(s)` URL, so a `file://` path cannot be read off
/// the user's disk through the job box.
pub async fn fetch(url: &str, timeout_secs: u64) -> Result<String> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::Invalid(
            "a job link must start with http:// or https://".into(),
        ));
    }
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::Fetch(e.to_string()))?;

    // Some boards serve a stripped page, or none at all, to a request that asks for
    // nothing in particular.
    let res = client
        .get(url)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| AppError::Fetch(e.to_string()))?;
    if !res.status().is_success() {
        return Err(AppError::Fetch(format!(
            "the site answered {}",
            res.status()
        )));
    }
    res.text().await.map_err(|e| AppError::Fetch(e.to_string()))
}
