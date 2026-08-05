//! Vault IPC. Handlers only: no SQL, no business logic.

use crate::db;
use crate::domain::*;
use crate::error::Result;
use crate::services;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn vault_list_experiences(state: State<'_, AppState>) -> Result<Vec<ExperienceDetail>> {
    db::experience::list_details(&state.pool).await
}

#[tauri::command]
pub async fn vault_upsert_experience(
    state: State<'_, AppState>,
    input: ExperienceInput,
    skills: Option<Vec<String>>,
) -> Result<Experience> {
    let experience = db::experience::upsert(&state.pool, &input).await?;
    if let Some(skills) = skills {
        db::skill::set_experience_skills(&state.pool, experience.id, &skills).await?;
    }
    Ok(experience)
}

#[tauri::command]
pub async fn vault_delete_experience(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::experience::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn vault_upsert_role(state: State<'_, AppState>, input: RoleInput) -> Result<Role> {
    db::role::upsert(&state.pool, &input).await
}

#[tauri::command]
pub async fn vault_delete_role(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::role::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn vault_upsert_bullet(state: State<'_, AppState>, input: BulletInput) -> Result<Bullet> {
    db::bullet::upsert(&state.pool, &input).await
}

#[tauri::command]
pub async fn vault_delete_bullet(state: State<'_, AppState>, id: Uuid) -> Result<()> {
    db::bullet::delete(&state.pool, id).await
}

#[tauri::command]
pub async fn vault_set_bullet_skills(
    state: State<'_, AppState>,
    bullet_id: Uuid,
    skill_names: Vec<String>,
) -> Result<()> {
    db::skill::set_bullet_skills(&state.pool, bullet_id, &skill_names).await
}

#[tauri::command]
pub async fn vault_suggest_bullet_improvements(
    state: State<'_, AppState>,
    bullet_id: Uuid,
) -> Result<Vec<Suggestion>> {
    services::vault::suggest_improvements(&state, bullet_id).await
}

#[tauri::command]
pub async fn vault_accept_variant(state: State<'_, AppState>, variant_id: Uuid) -> Result<Bullet> {
    db::bullet::accept_variant(&state.pool, variant_id).await
}

#[tauri::command]
pub async fn vault_list_skills(state: State<'_, AppState>) -> Result<Vec<Skill>> {
    db::skill::list(&state.pool).await
}

#[tauri::command]
pub async fn vault_add_skill(state: State<'_, AppState>, name: String) -> Result<Skill> {
    db::skill::add_listed(&state.pool, &name).await
}

#[tauri::command]
pub async fn vault_set_skill_listed(
    state: State<'_, AppState>,
    id: Uuid,
    listed: bool,
) -> Result<()> {
    db::skill::set_always_list(&state.pool, id, listed).await
}

#[tauri::command]
pub async fn vault_get_profile(state: State<'_, AppState>) -> Result<Option<Profile>> {
    db::profile::get(&state.pool).await
}

#[tauri::command]
pub async fn vault_upsert_profile(state: State<'_, AppState>, profile: Profile) -> Result<Profile> {
    db::profile::upsert(&state.pool, &profile).await
}
