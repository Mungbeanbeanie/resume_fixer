pub mod commands;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod ingest;
pub mod llm;
pub mod pipeline;
pub mod render;
pub mod services;
pub mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = config::Config::load()?;
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            // The sqlx pool spawns its maintenance task at construction, so it has to be
            // built inside the async runtime rather than on the bare setup thread.
            let state =
                tauri::async_runtime::block_on(
                    async move { state::AppState::new(config, data_dir) },
                )?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
            commands::vault::vault_list_experiences,
            commands::vault::vault_upsert_experience,
            commands::vault::vault_delete_experience,
            commands::vault::vault_upsert_role,
            commands::vault::vault_delete_role,
            commands::vault::vault_upsert_bullet,
            commands::vault::vault_delete_bullet,
            commands::vault::vault_set_bullet_skills,
            commands::vault::vault_suggest_bullet_improvements,
            commands::vault::vault_accept_variant,
            commands::vault::vault_list_skills,
            commands::vault::vault_add_skill,
            commands::vault::vault_set_skill_listed,
            commands::vault::vault_get_profile,
            commands::vault::vault_upsert_profile,
            commands::generate::generate_ingest_job,
            commands::generate::generate_from_text,
            commands::generate::generate_discard,
            commands::generate::generate_revise,
            commands::generate::generate_rename,
            commands::generate::generate_commit,
            commands::generate::generate_export_pdf,
            commands::generate::generate_export_draft,
            commands::base::base_list_templates,
            commands::base::base_save_template,
            commands::base::base_set_active_template,
            commands::base::base_delete_template,
            commands::base::base_render,
            commands::base::base_revise,
            commands::base::base_save,
            commands::base::base_list,
            commands::base::base_delete,
            commands::base::base_export,
            commands::library::library_track_application,
            commands::library::library_list_applications,
            commands::library::library_get_application,
            commands::library::library_set_status,
            commands::library::library_update_application,
            commands::library::library_delete_application,
            commands::library::library_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
