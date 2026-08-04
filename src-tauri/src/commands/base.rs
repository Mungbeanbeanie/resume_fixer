//! Base resume and template IPC. Handlers only: no SQL, no business logic.

use super::generate::safe_filename;
use crate::db;
use crate::domain::*;
use crate::error::{AppError, Result};
use crate::render::tex::{self, RenderInput};
use crate::services;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn base_list_templates(state: State<'_, AppState>) -> Result<Vec<Template>> {
    db::template::list(&state.pool).await
}

/// Stores a pasted or edited template.
///
/// Rejects a source Tera cannot parse. A broken template saved here would break every
/// later generation, and the error would surface far from the paste that caused it.
#[tauri::command]
pub async fn base_save_template(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> Result<Template> {
    tex::render(&source, &RenderInput::default())
        .map_err(|e| AppError::Invalid(format!("that template does not compile: {e}")))?;
    db::template::upsert(&state.pool, &name, &source).await
}

#[tauri::command]
pub async fn base_set_active_template(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::template::set_active(&state.pool, id).await
}

#[tauri::command]
pub async fn base_delete_template(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::template::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn base_render(
    state: State<'_, AppState>,
    template_id: Option<Uuid>,
) -> Result<BasePreview> {
    services::base::render(&state, template_id).await
}

#[tauri::command]
pub async fn base_save(state: State<'_, AppState>, name: String) -> Result<BaseResume> {
    services::base::save(&state, name).await
}

#[tauri::command]
pub async fn base_list(state: State<'_, AppState>) -> Result<Vec<BaseResume>> {
    db::base_resume::list(&state.pool).await
}

#[tauri::command]
pub async fn base_delete(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    services::base::delete(&state, id).await
}

/// Copies a saved base resume into the configured output directory.
#[tauri::command]
pub async fn base_export(state: State<'_, AppState>, id: Uuid, filename: String) -> Result<String> {
    let base = db::base_resume::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("base resume".into()))?;
    let source = base
        .pdf_path
        .ok_or_else(|| AppError::NotFound("PDF for this base resume".into()))?;
    let dest = state.config.output_dir()?.join(safe_filename(&filename));
    tokio::fs::copy(&source, &dest).await?;
    Ok(dest.to_string_lossy().to_string())
}
