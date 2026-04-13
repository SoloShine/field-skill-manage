use tauri::State;

use crate::commands::config::AppState;
use crate::models::skill::SkillMeta;
use crate::services::{git_service, skill_service};

#[tauri::command]
pub fn sync_remote_repo(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let remote_url = config.remote_url.clone();
    let cache_path = config.cache_path.clone();
    drop(config);

    git_service::sync_repo(&remote_url, &cache_path)
}

#[tauri::command]
pub fn get_remote_skills(state: State<'_, AppState>) -> Result<Vec<SkillMeta>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let cache_path = config.cache_path.clone();
    drop(config);

    let manifest = skill_service::parse_manifest(&cache_path)?;

    let mut skills = Vec::new();
    for entry in &manifest.skills {
        match skill_service::build_remote_skill_meta(&cache_path, entry) {
            Ok(meta) => skills.push(meta),
            Err(e) => {
                // Log but don't fail the whole list
                eprintln!("Warning: Failed to parse skill {}: {}", entry.name, e);
            }
        }
    }

    Ok(skills)
}

#[tauri::command]
pub fn get_remote_skill_detail(
    state: State<'_, AppState>,
    skill_name: String,
) -> Result<SkillMeta, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let cache_path = config.cache_path.clone();
    drop(config);

    let manifest = skill_service::parse_manifest(&cache_path)?;
    let entry = manifest
        .skills
        .iter()
        .find(|s| s.name == skill_name)
        .ok_or(format!("Skill '{}' not found in manifest", skill_name))?;

    let mut meta = skill_service::build_remote_skill_meta(&cache_path, entry)?;
    let skill_dir = std::path::Path::new(&cache_path).join(&entry.path);
    meta.files = crate::services::hash_service::list_file_hashes(&skill_dir).ok();

    Ok(meta)
}
