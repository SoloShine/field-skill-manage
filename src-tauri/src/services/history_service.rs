use crate::models::history::{OperationHistory, OperationRecord, OperationType};
use crate::models::config::RepoConfig;
use std::path::Path;

fn get_history_path(home: &str) -> String {
    format!("{}/.spm/history.json", home)
}

fn get_home() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

pub fn load_history() -> OperationHistory {
    let home = get_home();
    let path = get_history_path(&home);
    let path = Path::new(&path);
    if !path.exists() {
        return OperationHistory::default();
    }
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => OperationHistory::default(),
    }
}

pub fn save_history(history: &OperationHistory) -> Result<(), String> {
    let home = get_home();
    let path = get_history_path(&home);
    let content = serde_json::to_string_pretty(history)
        .map_err(|e| format!("Serialize history: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Write history: {}", e))
}

pub fn record_operation(
    operation: OperationType,
    skill_name: &str,
    target: &str,
    repo_id: Option<&str>,
    version_before: Option<&str>,
    version_after: Option<&str>,
    rollback_available: bool,
) -> Result<OperationRecord, String> {
    let mut history = load_history();

    let id = format!(
        "{:x}",
        chrono::Utc::now().timestamp_millis()
    );

    let record = OperationRecord {
        id,
        operation,
        skill_name: skill_name.to_string(),
        target: target.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        repo_id: repo_id.map(|s| s.to_string()),
        version_before: version_before.map(|s| s.to_string()),
        version_after: version_after.map(|s| s.to_string()),
        rollback_available,
    };

    history.records.insert(0, record.clone());
    // Keep max 200 records
    history.records.truncate(200);

    save_history(&history)?;
    Ok(record)
}

pub fn rollback_operation(
    operation_id: &str,
    repos: &[RepoConfig],
    global_path: &str,
    agent_project_patterns: &std::collections::HashMap<String, String>,
    active_agent_id: &str,
) -> Result<(), String> {
    let mut history = load_history();

    let record = history.records.iter().find(|r| r.id == operation_id)
        .ok_or_else(|| format!("Operation {} not found", operation_id))?
        .clone();

    if !record.rollback_available {
        return Err("This operation cannot be rolled back".to_string());
    }

    let target_dir = if record.target == "global" {
        global_path.to_string()
    } else {
        let pattern = agent_project_patterns
            .get(active_agent_id)
            .cloned()
            .unwrap_or_else(|| "{project}/.claude/skills".to_string());
        pattern.replace("{project}", &record.target)
    };

    let skill_dir = Path::new(&target_dir).join(&record.skill_name);

    match record.operation {
        OperationType::Install => {
            // Rollback install = uninstall
            if skill_dir.exists() {
                std::fs::remove_dir_all(&skill_dir)
                    .map_err(|e| format!("Remove skill: {}", e))?;
            }
        }
        OperationType::Uninstall => {
            // Rollback uninstall = re-install from source repo
            let repo_id = record.repo_id.as_deref();
            let target_repos: Vec<&RepoConfig> = if let Some(rid) = repo_id {
                repos.iter().filter(|r| r.id == rid).collect()
            } else {
                repos.iter().filter(|r| r.enabled).collect()
            };

            let mut found = false;
            for repo in target_repos {
                let entries = crate::services::skill_service::load_skill_entries(&repo.cache_path);
                if let Some(entry) = entries.iter().find(|s| s.name == record.skill_name) {
                    crate::services::skill_service::install_skill_to_dir(
                        &repo.cache_path,
                        &entry.path,
                        &target_dir,
                    )?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("Cannot rollback: skill '{}' not found in any repository", record.skill_name));
            }
        }
        OperationType::Update => {
            // Rollback update = re-install previous version if available in repo
            // Since we don't keep backups, we try to reinstall from the same repo
            let repo_id = record.repo_id.as_deref();
            let target_repos: Vec<&RepoConfig> = if let Some(rid) = repo_id {
                repos.iter().filter(|r| r.id == rid).collect()
            } else {
                repos.iter().filter(|r| r.enabled).collect()
            };

            // For updates, we can only rollback if the repo still has the old version
            // This is a best-effort rollback
            let mut found = false;
            for repo in target_repos {
                let entries = crate::services::skill_service::load_skill_entries(&repo.cache_path);
                if let Some(entry) = entries.iter().find(|s| s.name == record.skill_name) {
                    crate::services::skill_service::install_skill_to_dir(
                        &repo.cache_path,
                        &entry.path,
                        &target_dir,
                    )?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("Cannot rollback: skill '{}' not found in any repository", record.skill_name));
            }
        }
    }

    // Mark as non-rollbackable after rollback
    if let Some(r) = history.records.iter_mut().find(|r| r.id == operation_id) {
        r.rollback_available = false;
    }
    save_history(&history)?;

    Ok(())
}
