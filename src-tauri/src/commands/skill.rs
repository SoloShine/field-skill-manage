use tauri::State;

use crate::commands::config::AppState;
use crate::models::history::OperationType;
use crate::models::config::RepoConfig;
use crate::models::skill::{DependencyStatus, SkillComparison, SkillbaseResolution, ProjectDetailData, ScanAgentSkillsResult, ConflictResolution, MigrateResult, SkillDiff};
use crate::services::{history_service, skill_service};

/// Get overview of skills for multiple projects
#[tauri::command]
pub fn get_projects_overview(
    state: State<'_, AppState>,
    project_paths: Vec<String>,
) -> Result<Vec<skill_service::ProjectSkillSummary>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    skill_service::build_projects_overview(&project_paths, &repos, &active_id, &patterns)
}

/// Get skill comparison for the active agent's global directory
#[tauri::command]
pub fn get_global_skills(state: State<'_, AppState>) -> Result<Vec<SkillComparison>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let repos = config.repos.clone();
    drop(config);

    skill_service::build_skill_comparisons(&global_path, &repos)
}

/// Get skill comparison for a specific project
#[tauri::command]
pub fn get_project_skills(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<SkillComparison>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let project_skills_dir = config.active_project_dir(&project_path);
    drop(config);

    skill_service::build_skill_comparisons(&project_skills_dir, &repos)
}

/// Get both skill comparisons and skillbase resolution in a single IPC call
#[tauri::command]
pub fn get_project_detail(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectDetailData, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    skill_service::build_project_detail(&project_path, &repos, &patterns, &active_id)
}

#[tauri::command]
pub fn install_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
    repo_id: Option<String>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let global_path = config.active_global_path();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    // Find the skill in the specified repo or search all repos
    let target_repos: Vec<_> = if let Some(ref rid) = repo_id {
        repos.iter().filter(|r| &r.id == rid).collect()
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    for repo in target_repos {
        let entries = skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            let result = skill_service::install_skill_to_dir(&repo.cache_path, &entry.path, &target_dir);

            // Record history
            let version_after = skill_service::parse_skill_frontmatter(
                &std::path::Path::new(&target_dir).join(&skill_name).to_string_lossy(),
            ).ok().map(|f| f.version);

            let _ = history_service::record_operation(
                OperationType::Install,
                &skill_name,
                &target,
                repo_id.as_deref(),
                None,
                version_after.as_deref(),
                true,
            );

            return result;
        }
    }

    Err(format!("Skill '{}' not found in any repository", skill_name))
}

#[tauri::command]
pub fn update_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
    repo_id: Option<String>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let global_path = config.active_global_path();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    // Get version before update
    let skill_dir = std::path::Path::new(&target_dir).join(&skill_name);
    let version_before = skill_service::parse_skill_frontmatter(
        &skill_dir.to_string_lossy(),
    ).ok().map(|f| f.version);

    // Find the skill in the specified repo or search all repos
    let target_repos: Vec<_> = if let Some(ref rid) = repo_id {
        repos.iter().filter(|r| &r.id == rid).collect()
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    for repo in target_repos {
        let entries = skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            let result = skill_service::install_skill_to_dir(&repo.cache_path, &entry.path, &target_dir);

            // Get version after update
            let version_after = skill_service::parse_skill_frontmatter(
                &skill_dir.to_string_lossy(),
            ).ok().map(|f| f.version);

            let _ = history_service::record_operation(
                OperationType::Update,
                &skill_name,
                &target,
                repo_id.as_deref(),
                version_before.as_deref(),
                version_after.as_deref(),
                true,
            );

            return result;
        }
    }

    Err(format!("Skill '{}' not found in any repository", skill_name))
}

#[tauri::command]
pub fn batch_update(
    state: State<'_, AppState>,
    skill_names: Vec<String>,
    target: String,
    repo_id: Option<String>,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for name in &skill_names {
        match update_skill(state.clone(), name.clone(), target.clone(), repo_id.clone()) {
            Ok(()) => results.push(format!("{}: OK", name)),
            Err(e) => results.push(format!("{}: FAILED - {}", name, e)),
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn uninstall_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    let skill_path = std::path::Path::new(&target_dir).join(&skill_name);

    // Get version before uninstall
    let version_before = skill_service::parse_skill_frontmatter(
        &skill_path.to_string_lossy(),
    ).ok().map(|f| f.version);

    if skill_path.exists() {
        std::fs::remove_dir_all(&skill_path)
            .map_err(|e| format!("Failed to remove {}: {}", skill_path.display(), e))?;

        let _ = history_service::record_operation(
            OperationType::Uninstall,
            &skill_name,
            &target,
            None,
            version_before.as_deref(),
            None,
            true,
        );

        Ok(())
    } else {
        Err(format!("Skill '{}' not found at {}", skill_name, skill_path.display()))
    }
}

/// Resolve skillbase.json dependencies for a project
#[tauri::command]
pub fn get_skillbase_resolution(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<SkillbaseResolution, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    skill_service::resolve_skillbase_dependencies(&project_path, &repos, &patterns, &active_id)
}

/// Install all missing/mismatched/outdated dependencies from skillbase.json
#[tauri::command]
pub fn sync_skillbase_dependencies(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    let target_dir = config.active_project_dir(&project_path);
    drop(config);

    let resolution = skill_service::resolve_skillbase_dependencies(
        &project_path,
        &repos,
        &patterns,
        &active_id,
    )?;

    // Filter repos based on manifest.registry (same logic as resolution)
    let filtered_repos: Vec<&RepoConfig> = if let Some(ref registry_url) = resolution.manifest.registry
    {
        let matched: Vec<&RepoConfig> = repos
            .iter()
            .filter(|r| r.enabled && r.url == *registry_url)
            .collect();
        if matched.is_empty() {
            repos.iter().filter(|r| r.enabled).collect()
        } else {
            matched
        }
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    let mut results = Vec::new();
    for dep in &resolution.dependencies {
        match dep.status {
            DependencyStatus::Missing
            | DependencyStatus::VersionMismatch
            | DependencyStatus::Outdated => {
                let mut installed = false;

                // Prefer the repo that resolved the dependency
                if let Some(ref resolved) = dep.resolved {
                    if let Some(ref repo_id) = resolved.source_repo_id {
                        if let Some(repo) = filtered_repos.iter().find(|r| r.id == *repo_id) {
                            let entries = skill_service::load_skill_entries(&repo.cache_path);
                            if let Some(entry) = entries.iter().find(|s| s.name == dep.skill_name)
                            {
                                match skill_service::install_skill_to_dir(
                                    &repo.cache_path,
                                    &entry.path,
                                    &target_dir,
                                ) {
                                    Ok(()) => {
                                        results.push(format!("{}: OK", dep.reference));
                                        installed = true;
                                    }
                                    Err(e) => {
                                        results
                                            .push(format!("{}: FAILED - {}", dep.reference, e));
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback: search all filtered repos
                if !installed {
                    for repo in &filtered_repos {
                        let entries = skill_service::load_skill_entries(&repo.cache_path);
                        if let Some(entry) = entries.iter().find(|s| s.name == dep.skill_name) {
                            match skill_service::install_skill_to_dir(
                                &repo.cache_path,
                                &entry.path,
                                &target_dir,
                            ) {
                                Ok(()) => {
                                    results.push(format!("{}: OK", dep.reference));
                                    installed = true;
                                    break;
                                }
                                Err(e) => {
                                    results.push(format!("{}: FAILED - {}", dep.reference, e));
                                }
                            }
                        }
                    }
                }

                if !installed && !results.iter().any(|r| r.starts_with(&dep.reference)) {
                    results.push(format!("{}: FAILED - not found in any repo", dep.reference));
                }
            }
            _ => results.push(format!("{}: SKIP (satisfied)", dep.reference)),
        }
    }
    Ok(results)
}

/// Generate skillbase.json content from currently installed skills
#[tauri::command]
pub fn generate_skillbase_json(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    let repos = config.repos.clone();
    drop(config);

    let project_name = std::path::Path::new(&project_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-project".to_string());

    skill_service::generate_skillbase_manifest(&project_path, &project_name, &patterns, &active_id, &repos)
}

/// Write skillbase.json content to project root
#[tauri::command]
pub fn write_skillbase_json(project_path: String, content: String) -> Result<(), String> {
    let path = std::path::Path::new(&project_path).join("skillbase.json");
    std::fs::write(&path, &content).map_err(|e| format!("Write skillbase.json: {}", e))
}

/// Scan another agent's skill directory and compare with the active agent's directory
#[tauri::command]
pub fn scan_agent_skills(
    state: State<'_, AppState>,
    agent_id: String,
    scope: String,
    project_path: Option<String>,
) -> Result<ScanAgentSkillsResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let active_id = config.active_agent_id.clone();

    if agent_id == active_id {
        return Err("Cannot migrate from the active agent to itself".to_string());
    }

    // Resolve source directory for the specified agent
    let source_dir = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        let pattern = config.agent_project_patterns.get(&agent_id).cloned().unwrap_or_default();
        pattern.replace("{project}", pp)
    } else {
        config.agent_global_paths.get(&agent_id).cloned().unwrap_or_default()
    };

    if source_dir.is_empty() {
        return Err(format!("No path configured for agent '{}'", agent_id));
    }

    // Resolve target directory for the active agent
    let target_dir = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        config.active_project_dir(pp)
    } else {
        config.active_global_path()
    };

    let display_name = config.agent_display_name(&agent_id);
    drop(config);

    let mut result = skill_service::scan_agent_skills_dir(&source_dir, &target_dir)?;
    result.agent_id = agent_id;
    result.agent_display_name = display_name;
    Ok(result)
}

/// Migrate selected skills from another agent to the active agent
#[tauri::command]
pub fn migrate_skills(
    state: State<'_, AppState>,
    source_agent_id: String,
    skill_names: Vec<String>,
    scope: String,
    project_path: Option<String>,
    conflict_map: std::collections::HashMap<String, ConflictResolution>,
) -> Result<MigrateResult, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;

    // Resolve source directory
    let source_dir = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        let pattern = config.agent_project_patterns.get(&source_agent_id).cloned().unwrap_or_default();
        pattern.replace("{project}", pp)
    } else {
        config.agent_global_paths.get(&source_agent_id).cloned().unwrap_or_default()
    };

    if source_dir.is_empty() {
        return Err(format!("No path configured for agent '{}'", source_agent_id));
    }

    // Resolve target directory
    let target_dir = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        config.active_project_dir(pp)
    } else {
        config.active_global_path()
    };

    drop(config);

    let result = skill_service::migrate_skills_to_dir(&source_dir, &target_dir, &skill_names, &conflict_map)?;

    // Record history for each migrated skill
    for name in &result.migrated {
        let _ = history_service::record_operation(
            OperationType::Install,
            name,
            &target_dir,
            Some(&format!("migrate:{}", source_agent_id)),
            None,
            None,
            false,
        );
    }

    Ok(result)
}

/// Get file-level diff between a source agent's skill and the active agent's skill
#[tauri::command]
pub fn get_migrate_skill_diff(
    state: State<'_, AppState>,
    source_agent_id: String,
    skill_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<SkillDiff, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;

    // Resolve source base directory
    let source_base = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        let pattern = config.agent_project_patterns.get(&source_agent_id).cloned().unwrap_or_default();
        pattern.replace("{project}", pp)
    } else {
        config.agent_global_paths.get(&source_agent_id).cloned().unwrap_or_default()
    };

    if source_base.is_empty() {
        return Err(format!("No path configured for agent '{}'", source_agent_id));
    }

    // Resolve target base directory
    let target_base = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        config.active_project_dir(pp)
    } else {
        config.active_global_path()
    };

    drop(config);

    let source_dir = std::path::Path::new(&source_base).join(&skill_name);
    let target_dir = std::path::Path::new(&target_base).join(&skill_name);

    eprintln!("[migrate_diff] source_dir={}", source_dir.display());
    eprintln!("[migrate_diff] target_dir={}", target_dir.display());
    eprintln!("[migrate_diff] source_exists={}, target_exists={}", source_dir.exists(), target_dir.exists());

    skill_service::build_skill_diff(&target_dir, &source_dir)
}

/// Read file content from both source and target agent directories for migration diff
#[tauri::command]
pub fn get_migrate_diff_content(
    state: State<'_, AppState>,
    source_agent_id: String,
    skill_name: String,
    file_path: String,
    scope: String,
    project_path: Option<String>,
) -> Result<crate::commands::version::DiffFileContent, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;

    let source_base = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        let pattern = config.agent_project_patterns.get(&source_agent_id).cloned().unwrap_or_default();
        pattern.replace("{project}", pp)
    } else {
        config.agent_global_paths.get(&source_agent_id).cloned().unwrap_or_default()
    };

    let target_base = if scope == "project" {
        let pp = project_path.as_ref().ok_or("project_path is required for project scope")?;
        config.active_project_dir(pp)
    } else {
        config.active_global_path()
    };

    drop(config);

    let source_file = std::path::Path::new(&source_base).join(&skill_name).join(&file_path);
    let target_file = std::path::Path::new(&target_base).join(&skill_name).join(&file_path);

    let local_content = if target_file.exists() {
        Some(std::fs::read_to_string(&target_file).map_err(|e| format!("Read target file: {}", e))?)
    } else {
        None
    };

    let remote_content = if source_file.exists() {
        Some(std::fs::read_to_string(&source_file).map_err(|e| format!("Read source file: {}", e))?)
    } else {
        None
    };

    Ok(crate::commands::version::DiffFileContent {
        local_content,
        remote_content,
    })
}
