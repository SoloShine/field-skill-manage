mod commands;
mod models;
mod services;

use commands::{config, git_sync, history, preview, skill, update, version};
use commands::config::AppState;
use std::sync::Mutex;

fn get_home() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Config file: <home>/.spm/config.json
    let home = get_home();
    let config_dir = format!("{}/.spm", home);
    let config_path = format!("{}/.spm/config.json", home);

    // Ensure config directory exists
    std::fs::create_dir_all(&config_dir).ok();

    // Load config from disk (or use defaults)
    let app_config = config::load_config_from_disk(&config_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            config: Mutex::new(app_config),
            config_path,
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            config::get_config,
            config::set_config,
            config::get_agent_types,
            config::get_all_agents,
            config::add_custom_agent,
            config::remove_custom_agent,
            config::export_config,
            config::import_config,
            // Git sync
            git_sync::sync_remote_repo,
            git_sync::get_remote_skills,
            git_sync::get_remote_skill_detail,
            // Skill
            skill::get_projects_overview,
            skill::get_global_skills,
            skill::get_project_skills,
            skill::get_project_detail,
            skill::install_skill,
            skill::update_skill,
            skill::batch_update,
            skill::uninstall_skill,
            skill::get_skillbase_resolution,
            skill::sync_skillbase_dependencies,
            skill::generate_skillbase_json,
            skill::write_skillbase_json,
            skill::scan_agent_skills,
            skill::migrate_skills,
            skill::get_migrate_skill_diff,
            skill::get_migrate_diff_content,
            // Version
            version::calculate_skill_hash,
            version::compare_skill_versions,
            version::verify_skill_integrity,
            version::get_skill_diff,
            version::get_diff_file_content,
            // History
            history::get_operation_history,
            history::rollback_operation,
            history::clear_history,
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
