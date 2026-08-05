//! Generate IPC.

use crate::domain::{BulletEdit, GenerationResult, IngestResult, JobSource};
use crate::error::{AppError, Result};
use crate::ingest;
use crate::services;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn generate_ingest_job(state: State<'_, AppState>, url: String) -> Result<IngestResult> {
    ingest::run(&url, &state.config.ingest).await
}

#[tauri::command]
pub async fn generate_from_text(
    state: State<'_, AppState>,
    job_text: String,
    url: Option<String>,
    fetched: Option<bool>,
    feedback: Option<String>,
) -> Result<GenerationResult> {
    let source = if fetched.unwrap_or(false) {
        JobSource::Fetched
    } else {
        JobSource::Pasted
    };
    services::generate::generate(&state, &job_text, url, source, feedback).await
}

#[tauri::command]
pub async fn generate_discard(state: State<'_, AppState>, draft_id: Uuid) -> Result<()> {
    services::generate::discard(&state, draft_id).await
}

#[tauri::command]
pub async fn generate_revise(
    state: State<'_, AppState>,
    draft_id: Uuid,
    edits: Vec<BulletEdit>,
) -> Result<GenerationResult> {
    services::generate::revise(&state, draft_id, edits).await
}

#[tauri::command]
pub async fn generate_commit(
    state: State<'_, AppState>,
    draft_id: Uuid,
    applied: bool,
) -> Result<Uuid> {
    services::generate::commit(&state, draft_id, applied).await
}

/// Copies an uncommitted draft's PDF into the configured output directory.
///
/// A draft is a real file on disk already; downloading one must not require committing it
/// to the library first.
#[tauri::command]
pub async fn generate_export_draft(
    state: State<'_, AppState>,
    draft_id: Uuid,
    filename: String,
) -> Result<String> {
    let source = {
        let drafts = state.drafts.lock().await;
        drafts
            .get(&draft_id)
            .ok_or_else(|| AppError::NotFound("draft".into()))?
            .pdf_path
            .clone()
    };
    let dest = state.config.output_dir()?.join(safe_filename(&filename));
    tokio::fs::copy(&source, &dest).await?;
    Ok(dest.to_string_lossy().to_string())
}

/// Strips anything that would let a name escape the output directory.
pub(crate) fn safe_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' {
                '-'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim_matches(['.', ' ']).to_string();
    if cleaned.is_empty() {
        "resume.pdf".to_string()
    } else if cleaned.ends_with(".pdf") {
        cleaned
    } else {
        format!("{cleaned}.pdf")
    }
}

/// Copies a stored PDF to a destination the user picked.
///
/// A bare filename lands in the configured output directory; an absolute path is honored
/// as given.
#[tauri::command]
pub async fn generate_export_pdf(
    state: State<'_, AppState>,
    resume_id: Uuid,
    dest_path: String,
) -> Result<String> {
    let resume = crate::db::resume::get(&state.pool, resume_id)
        .await?
        .ok_or_else(|| AppError::NotFound("resume".into()))?;
    let source = resume
        .pdf_path
        .ok_or_else(|| AppError::NotFound("PDF for this resume".into()))?;

    let dest = std::path::PathBuf::from(&dest_path);
    let dest = if dest.is_absolute() {
        dest
    } else {
        state.config.output_dir()?.join(dest)
    };
    tokio::fs::copy(&source, &dest).await?;
    Ok(dest.to_string_lossy().to_string())
}
