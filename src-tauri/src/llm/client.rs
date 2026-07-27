//! The Ollama HTTP client, behind a trait so tests never need a running model.

use crate::config::Llm;
use crate::error::{AppError, Result};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub type Answer<'a> = Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;

/// One completion call. Implementors must ask the backend for JSON-only output.
pub trait LlmClient: Send + Sync {
    fn complete_json<'a>(&'a self, prompt: &'a str) -> Answer<'a>;
    fn model(&self) -> &str;
}

pub struct OllamaClient {
    http: reqwest::Client,
    endpoint: String,
    model: String,
    temperature: f32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    /// Reasoning models return their chain here and leave `response` empty when thinking
    /// is on. `ornith:35b` is one of them.
    #[serde(default)]
    thinking: Option<String>,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<Tag>,
}

#[derive(Deserialize)]
struct Tag {
    name: String,
}

impl OllamaClient {
    pub fn new(cfg: &Llm) -> Result<Self> {
        Ok(OllamaClient {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(cfg.timeout_secs))
                .build()
                .map_err(|e| AppError::Llm(e.to_string()))?,
            endpoint: cfg.endpoint.trim_end_matches('/').to_string(),
            model: cfg.model.clone(),
            temperature: cfg.temperature,
        })
    }

    /// `(server reachable, configured model pulled)`. Never errors — it feeds a status strip.
    pub async fn health(&self) -> (bool, bool) {
        let Ok(res) = self
            .http
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
        else {
            return (false, false);
        };
        let Ok(tags) = res.json::<TagsResponse>().await else {
            return (true, false);
        };
        let present = tags.models.iter().any(|m| m.name == self.model);
        (true, present)
    }
}

impl OllamaClient {
    fn request(&self, prompt: &str, think: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "format": "json",
            "options": { "temperature": self.temperature },
        });
        if !think {
            body["think"] = serde_json::Value::Bool(false);
        }
        body
    }

    async fn post(&self, body: &serde_json::Value) -> Result<GenerateResponse> {
        let res = self
            .http
            .post(format!("{}/api/generate", self.endpoint))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Llm(e.to_string()))?;
        if !res.status().is_success() {
            return Err(AppError::Llm(format!("Ollama answered {}", res.status())));
        }
        res.json::<GenerateResponse>()
            .await
            .map_err(|e| AppError::Llm(e.to_string()))
    }
}

impl LlmClient for OllamaClient {
    /// Asks with thinking turned off, because a reasoning model in JSON mode puts the
    /// object in `thinking` and leaves `response` empty. Models that reject the `think`
    /// field are asked again without it, and an answer that still arrives as reasoning is
    /// read from there rather than thrown away.
    fn complete_json<'a>(&'a self, prompt: &'a str) -> Answer<'a> {
        Box::pin(async move {
            let answer = match self.post(&self.request(prompt, false)).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::info!("retrying without the think field: {e}");
                    self.post(&self.request(prompt, true)).await?
                }
            };
            Ok(match answer.response.trim().is_empty() {
                true => answer.thinking.unwrap_or_default(),
                false => answer.response,
            })
        })
    }

    fn model(&self) -> &str {
        &self.model
    }
}

/// Returns canned answers in order. Test-only; keeps the suites off the network.
#[cfg(test)]
pub struct FixtureClient {
    answers: std::sync::Mutex<std::collections::VecDeque<String>>,
}

#[cfg(test)]
impl FixtureClient {
    pub fn new(answers: &[&str]) -> Self {
        FixtureClient {
            answers: std::sync::Mutex::new(answers.iter().map(|s| s.to_string()).collect()),
        }
    }
}

#[cfg(test)]
impl LlmClient for FixtureClient {
    fn complete_json<'a>(&'a self, _prompt: &'a str) -> Answer<'a> {
        let next = self.answers.lock().unwrap().pop_front();
        Box::pin(async move { next.ok_or_else(|| AppError::Llm("fixture exhausted".into())) })
    }

    fn model(&self) -> &str {
        "fixture"
    }
}
