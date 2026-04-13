mod commands;
mod models;
mod services;

use commands::{config, git_sync, preview, skill, update, version};
use commands::config::AppState;
use models::config::AppConfig;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_config = AppConfig::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            config: Mutex::new(app_config),
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            config::get_config,
            config::set_config,
            config::get_agent_types,
            config::get_all_agents,
            config::add_custom_agent,
            config::remove_custom_agent,
            // Git sync
            git_sync::sync_remote_repo,
            git_sync::get_remote_skills,
            git_sync::get_remote_skill_detail,
            // Skill
            skill::get_projects_overview,
            skill::get_global_skills,
            skill::get_project_skills,
            skill::install_skill,
            skill::update_skill,
            skill::batch_update,
            skill::uninstall_skill,
            // Version
            version::calculate_skill_hash,
            version::compare_skill_versions,
            version::verify_skill_integrity,
            // Preview
            preview::get_skill_file_tree,
            preview::read_skill_file,
            // Update
            update::check_for_updates,
            update::get_current_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
