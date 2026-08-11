//! Model calls. Prompts live in `prompts/` as text; changing one means bumping
//! `PROMPT_VERSION` so stored `resumes` rows stay attributable.

pub mod client;
pub mod schemas;

use crate::error::{AppError, Result};
use client::LlmClient;
use schemas::{Improvements, ParsedJob, Selection};
use serde::de::DeserializeOwned;

/// Bump on any edit to a file in `prompts/`.
pub const PROMPT_VERSION: &str = "2026-08-06.1";

const PARSE_JOB: &str = include_str!("prompts/parse_job.txt");
const SELECT: &str = include_str!("prompts/select.txt");
const IMPROVE_BULLET: &str = include_str!("prompts/improve_bullet.txt");

/// Narrows a response to the outermost JSON object.
///
/// Ollama's `format: "json"` is reliable but not absolute; a stray fence or a leading
/// sentence should not cost a whole generation.
fn json_slice(raw: &str) -> &str {
    match (raw.find('{'), raw.rfind('}')) {
        (Some(a), Some(b)) if b > a => &raw[a..=b],
        _ => raw,
    }
}

/// Asks once, and once more if the answer does not parse. Two failures is a failed call.
async fn ask<T: DeserializeOwned>(llm: &dyn LlmClient, prompt: &str) -> Result<T> {
    let mut last = String::new();
    for attempt in 0..2 {
        let raw = llm.complete_json(prompt).await?;
        match serde_json::from_str::<T>(json_slice(&raw)) {
            Ok(v) => return Ok(v),
            Err(e) => {
                tracing::warn!("model returned unparseable JSON on attempt {attempt}: {e}");
                last = raw;
            }
        }
    }
    Err(AppError::Llm(format!(
        "the model did not return usable JSON: {}",
        last.chars().take(200).collect::<String>()
    )))
}

/// Model call #1: what does this posting ask for?
pub async fn parse_job(llm: &dyn LlmClient, job_text: &str) -> Result<ParsedJob> {
    ask(llm, &PARSE_JOB.replace("{{job_text}}", job_text)).await
}

/// Model call #2: which bullets, in which order, worded how.
///
/// `candidates` is the ID-keyed list the model is allowed to reference; anything else it
/// returns is dropped downstream.
pub async fn select(
    llm: &dyn LlmClient,
    job: &str,
    candidates: &str,
    skills: &str,
    feedback: Option<&str>,
) -> Result<Selection> {
    let prompt = SELECT
        .replace("{{job}}", job)
        .replace("{{candidates}}", candidates)
        .replace("{{skills}}", skills)
        .replace(
            "{{feedback}}",
            &feedback
                .map(|f| format!("\nTHE USER ASKED FOR THIS CHANGE:\n{f}\n"))
                .unwrap_or_default(),
        );
    ask(llm, &prompt).await
}

/// Vault "Improve": alternate phrasings for one stored bullet.
pub async fn improve_bullet(
    llm: &dyn LlmClient,
    bullet: &str,
    skills: &str,
) -> Result<Improvements> {
    let prompt = IMPROVE_BULLET
        .replace("{{bullet}}", bullet)
        .replace("{{skills}}", skills);
    ask(llm, &prompt).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::FixtureClient;

    #[tokio::test]
    async fn a_fenced_answer_still_parses() {
        let llm = FixtureClient::new(&["```json\n{\"hard_skills\":[\"Python\"]}\n```"]);
        let parsed = parse_job(&llm, "job").await.unwrap();
        assert_eq!(parsed.hard_skills, vec!["Python"]);
    }

    #[tokio::test]
    async fn a_second_chance_but_not_a_third() {
        let llm = FixtureClient::new(&["not json", "{\"hard_skills\":[\"Java\"]}"]);
        assert_eq!(parse_job(&llm, "job").await.unwrap().hard_skills, ["Java"]);

        let llm = FixtureClient::new(&["nope", "still nope", "{}"]);
        assert!(parse_job(&llm, "job").await.is_err());
    }
}
