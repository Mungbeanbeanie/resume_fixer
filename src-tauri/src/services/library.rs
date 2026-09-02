//! Tracking an application this app did not generate a resume for.

use crate::db;
use crate::domain::*;
use crate::error::{AppError, Result};
use crate::state::AppState;
use uuid::Uuid;

/// What an uploaded PDF records instead of a model and a prompt version. Nothing generated
/// it, and the Library reads this to say so rather than printing a version that never ran.
pub const UPLOADED: &str = "uploaded";

/// Records a submission the user made on their own, with the resume they actually sent.
///
/// The PDF is stored as this application's current resume, so the Library opens it exactly
/// as it opens a generated one. It has no TeX behind it and no bullets under it: nothing
/// here was selected from the vault, so there is no provenance to write.
///
/// Rejects an entry naming no company — a tracker row nobody can identify is noise.
pub async fn track(state: &AppState, input: ManualApplication) -> Result<Uuid> {
    let company = input
        .company
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| AppError::Invalid("name the company you applied to".into()))?;

    let application = db::application::create(
        &state.pool,
        input.url.as_deref(),
        Some(company),
        input.role_title.as_deref(),
        input.notes.as_deref().unwrap_or_default(),
        JobSource::Pasted,
        &serde_json::json!({}),
        input.status,
    )
    .await?;

    if let Some(pdf) = input.pdf.filter(|p| !p.is_empty()) {
        let resume = db::resume::insert(
            &state.pool,
            application.id,
            "",
            None,
            UPLOADED,
            UPLOADED,
            None,
            0,
            &[],
        )
        .await?;

        let dest = state.resume_path(resume.id);
        if let Some(dir) = dest.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        tokio::fs::write(&dest, &pdf).await?;
        db::resume::set_pdf_path(&state.pool, resume.id, &dest.to_string_lossy()).await?;
    }

    Ok(application.id)
}
