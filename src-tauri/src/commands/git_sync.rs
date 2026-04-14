use tauri::State;

use crate::commands::config::AppState;
use crate::models::skill::SkillMeta;
use crate::services::{git_service, skill_service};

/// Sync result with partial failure info
#[derive(serde::Serialize)]
pub struct SyncResult {
    pub success_count: usize,
    pub fail_count: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
pub fn sync_remote_repo(state: State<'_, AppState>) -> Result<SyncResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos: Vec<_> = config.repos.iter().filter(|r| r.enabled).cloned().collect();
    drop(config);

    if repos.is_empty() {
        return Err("No enabled repositories to sync".to_string());
    }

    let mut errors: Vec<String> = Vec::new();
    let mut success_count = 0usize;
    for repo in &repos {
        if let Err(e) = git_service::sync_repo(&repo.url, &repo.cache_path) {
            eprintln!("Warning: Failed to sync repo '{}': {}", repo.name, e);
            errors.push(format!("{}: {}", repo.name, e));
        } else {
            success_count += 1;
        }
    }

    // If ALL repos failed, return error
    if success_count == 0 {
        return Err(format!("All repos failed to sync: {}", errors.join("; ")));
    }

    Ok(SyncResult {
        success_count,
        fail_count: errors.len(),
        errors,
    })
}

#[tauri::command]
pub fn get_remote_skills(state: State<'_, AppState>) -> Result<Vec<SkillMeta>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    drop(config);

    let mut skills = Vec::new();
    for repo in &repos {
        if !repo.enabled {
            continue;
        }
        let entries = skill_service::load_skill_entries(&repo.cache_path);
        for entry in &entries {
            match skill_service::build_remote_skill_meta(&repo.cache_path, entry, Some(&repo.id)) {
                Ok(meta) => skills.push(meta),
                Err(e) => {
                    eprintln!("Warning: Failed to parse skill {} from {}: {}", entry.name, repo.name, e);
                }
            }
        }
    }

    Ok(skills)
}

#[tauri::command]
pub fn get_remote_skill_detail(
    state: State<'_, AppState>,
    skill_name: String,
    repo_id: Option<String>,
) -> Result<SkillMeta, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    drop(config);

    // If repo_id specified, search only that repo; otherwise search all
    let target_repos: Vec<_> = if let Some(ref rid) = repo_id {
        repos.iter().filter(|r| &r.id == rid).collect()
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    for repo in target_repos {
        let entries = skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            let mut meta = skill_service::build_remote_skill_meta(&repo.cache_path, entry, Some(&repo.id))?;
            let skill_dir = std::path::Path::new(&repo.cache_path).join(&entry.path);
            meta.files = crate::services::hash_service::list_file_hashes(&skill_dir).ok();
            return Ok(meta);
        }
    }

    Err(format!("Skill '{}' not found in any repository", skill_name))
}
