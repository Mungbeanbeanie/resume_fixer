//! Everything the IPC handlers share.

use crate::config::Config;
use crate::domain::{JobSource, RejectedRewrite};
use crate::error::Result;
use crate::llm::client::{LlmClient, OllamaClient};
use crate::pipeline::plan::ResumePlan;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

/// A generated resume that has not been committed to the library.
///
/// Drafts live here and nowhere else. Discarding one writes nothing to the database.
pub struct Draft {
    pub plan: ResumePlan,
    pub tex: String,
    pub pdf_path: PathBuf,
    pub page_count: i32,
    pub job_text: String,
    pub url: Option<String>,
    pub job_source: JobSource,
    pub parsed: serde_json::Value,
    pub company: Option<String>,
    pub role_title: Option<String>,
    pub rejected: Vec<RejectedRewrite>,
    pub feedback: Option<String>,
}

pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub llm: Box<dyn LlmClient>,
    pub data_dir: PathBuf,
    pub drafts: Mutex<HashMap<Uuid, Draft>>,
}

impl AppState {
    /// Builds the shared state. The database is connected lazily: a server that is not
    /// running yet is a status line, not a startup failure.
    pub fn new(config: Config, data_dir: PathBuf) -> Result<Self> {
        let pool = crate::db::pool::lazy(&config.database.url)?;
        Ok(AppState {
            llm: Box::new(OllamaClient::new(&config.llm)?),
            pool,
            data_dir,
            config,
            drafts: Mutex::new(HashMap::new()),
        })
    }

    pub fn draft_dir(&self, draft_id: Uuid) -> PathBuf {
        self.data_dir.join("drafts").join(draft_id.to_string())
    }

    pub fn resume_path(&self, resume_id: Uuid) -> PathBuf {
        self.data_dir
            .join("resumes")
            .join(format!("{resume_id}.pdf"))
    }
}
