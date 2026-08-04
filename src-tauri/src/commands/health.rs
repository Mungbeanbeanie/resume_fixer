//! The status strip's one command.

use crate::db;
use crate::domain::HealthReport;
use crate::error::Result;
use crate::llm::client::OllamaClient;
use crate::render::{tectonic, templates};
use crate::state::AppState;
use tauri::State;

/// Checks the three local services. Never fails: an unreachable dependency is the answer,
/// not an error.
#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<HealthReport> {
    let postgres = db::pool::is_ready(&state.pool).await;
    if postgres {
        // The schema is current here and nowhere earlier — the pool connects lazily, so
        // this is the first point at which the shipped templates can be written.
        if let Err(e) = db::template::sync_builtins(&state.pool, &templates::BUILTINS).await {
            tracing::warn!("could not sync the built-in templates: {e}");
        }
    }
    let (ollama, model_present) = match OllamaClient::new(&state.config.llm) {
        Ok(c) => c.health().await,
        Err(_) => (false, false),
    };
    Ok(HealthReport {
        postgres,
        ollama,
        model_present,
        tectonic: tectonic::is_available(&state.config.render.tectonic_path).await,
        model: state.config.llm.model.clone(),
    })
}
