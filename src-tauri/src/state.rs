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

/// The compiled base resume the user is looking at, before they name it. Same rule as a
/// draft: nothing is written until they save.
pub struct BaseDraft {
    /// The template this preview was built with. `revise` recompiles with this one, not
    /// with whichever template happens to be active — previewing a non-active template and
    /// then editing a line must not silently re-render the document under another layout.
    pub template_id: Uuid,
    pub template_name: String,
    pub plan: ResumePlan,
    pub tex: String,
    pub pdf_path: PathBuf,
    /// Where this preview compiles. Base and Build have one each, so a recompile of either
    /// leaves the other's PDF where its tab is still pointing.
    pub dir: PathBuf,
    pub page_count: i32,
    pub bullet_count: i32,
}

pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub llm: Box<dyn LlmClient>,
    pub data_dir: PathBuf,
    pub drafts: Mutex<HashMap<Uuid, Draft>>,
    pub base_preview: Mutex<Option<BaseDraft>>,
    /// The Build tab's preview. Its own slot because every tab stays mounted: one shared
    /// slot would mean an Apply in Base recompiling whatever Build rendered last.
    pub build_preview: Mutex<Option<BaseDraft>>,
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
            base_preview: Mutex::new(None),
            build_preview: Mutex::new(None),
        })
    }

    pub fn draft_dir(&self, draft_id: Uuid) -> PathBuf {
        self.data_dir.join("drafts").join(draft_id.to_string())
    }

    /// One reused directory: only the newest base preview is ever of interest.
    pub fn base_preview_dir(&self) -> PathBuf {
        self.data_dir.join("base").join("preview")
    }

    /// The Build tab compiles elsewhere. Both tabs stay mounted with a PDF on screen, and
    /// every compile overwrites `preview.pdf` — one directory would show each tab the
    /// other's document.
    pub fn build_preview_dir(&self) -> PathBuf {
        self.data_dir.join("base").join("build")
    }

    pub fn base_path(&self, base_id: Uuid) -> PathBuf {
        self.data_dir.join("base").join(format!("{base_id}.pdf"))
    }

    pub fn resume_path(&self, resume_id: Uuid) -> PathBuf {
        self.data_dir
            .join("resumes")
            .join(format!("{resume_id}.pdf"))
    }
}
