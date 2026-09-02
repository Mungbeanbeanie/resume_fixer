//! Library IPC.

use crate::db;
use crate::domain::*;
use crate::error::Result;
use crate::services;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn library_track_application(
    state: State<'_, AppState>,
    input: ManualApplication,
) -> Result<Uuid> {
    services::library::track(&state, input).await
}

#[tauri::command]
pub async fn library_list_applications(
    state: State<'_, AppState>,
    filter: Option<ApplicationStatus>,
) -> Result<Vec<ApplicationSummary>> {
    db::application::list_summaries(&state.pool, filter).await
}

#[tauri::command]
pub async fn library_get_application(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ApplicationDetail> {
    db::application::get_detail(&state.pool, id).await
}

#[tauri::command]
pub async fn library_set_status(
    state: State<'_, AppState>,
    id: Uuid,
    status: ApplicationStatus,
) -> Result<()> {
    db::application::set_status(&state.pool, id, status).await
}

#[tauri::command]
pub async fn library_update_application(
    state: State<'_, AppState>,
    id: Uuid,
    patch: ApplicationPatch,
) -> Result<()> {
    db::application::update(&state.pool, id, &patch).await
}

#[tauri::command]
pub async fn library_delete_application(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::application::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn library_stats(state: State<'_, AppState>) -> Result<StatusStats> {
    db::application::stats(&state.pool).await
}
