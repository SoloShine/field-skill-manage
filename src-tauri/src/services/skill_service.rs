use std::path::Path;

use crate::models::skill::{
    DependencyEntry, DependencyStatus, FileDiff, FileDiffStatus, InstallStatus, SkillDiff,
    SkillManifestEntry, SkillMeta, SkillbaseManifest, SkillbaseResolution, SkillsManifest,
};

use crate::models::config::RepoConfig;

/// Parse skills.json from a repository root
pub fn parse_manifest(repo_path: &str) -> Result<SkillsManifest, String> {
    let manifest_path = Path::new(repo_path).join("skills.json");
    let content =
        std::fs::read_to_string(&manifest_path).map_err(|e| format!("Read skills.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse skills.json: {}", e))
}

/// Scan a directory for skill folders containing SKILL.md (fallback for repos without skills.json)
/// Checks both the root level and a `skills/` subdirectory
pub fn scan_skills_from_dirs(repo_path: &str) -> Vec<SkillManifestEntry> {
    let root = Path::new(repo_path);
    let mut entries = Vec::new();

    // Collect candidate directories: root subdirs + skills/ subdirs
    let mut scan_dirs: Vec<std::path::PathBuf> = Vec::new();

    // Root level subdirectories
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    if !name.starts_with('.') && name != "skills" {
                        scan_dirs.push(p);
                    }
                }
            }
        }
    }

    // skills/ subdirectory (e.g. anthropics/skills repo)
    let skills_subdir = root.join("skills");
    if skills_subdir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&skills_subdir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Some(name) = e.file_name().to_str() {
                        if !name.starts_with('.') {
                            scan_dirs.push(p);
                        }
                    }
                }
            }
        }
    }

    // For each candidate, check if it has SKILL.md
    for dir in &scan_dirs {
        let skill_md = dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let dir_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Try to parse frontmatter for metadata
        let fm = parse_skill_frontmatter(&dir.to_string_lossy()).ok();

        let name = fm.as_ref().map(|f| f.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| dir_name.clone());

        let path = dir.strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(dir_name.clone());

        entries.push(SkillManifestEntry {
            name,
            path,
            version: fm.as_ref().map(|f| f.version.clone()).unwrap_or_default(),
            description: fm.as_ref().map(|f| f.description.clone()).unwrap_or_default(),
            tags: fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default(),
            updated_at: fm.as_ref().and_then(|f| f.updated_at.clone()),
            checksum: None,
            license: fm.as_ref().and_then(|f| f.license.clone()),
            author: fm.as_ref().and_then(|f| f.author.clone()),
            language: fm.as_ref().and_then(|f| f.language.clone()),
            trigger: fm.as_ref().and_then(|f| f.trigger.clone()),
            security: fm.as_ref().and_then(|f| f.security.clone()),
        });
    }

    entries
}

/// Load skill entries from a repo: try skills.json first, fall back to directory scanning
pub fn load_skill_entries(repo_path: &str) -> Vec<SkillManifestEntry> {
    // Try standard skills.json manifest first
    if let Ok(manifest) = parse_manifest(repo_path) {
        return manifest.skills;
    }

    // Fallback: scan directories for SKILL.md
    let entries = scan_skills_from_dirs(repo_path);
    if !entries.is_empty() {
        eprintln!(
            "No skills.json found in '{}', discovered {} skills via directory scan",
            repo_path,
            entries.len()
        );
    }
    entries
}

/// Parse SKILL.md YAML frontmatter from a skill directory using serde_yaml
pub fn parse_skill_frontmatter(skill_dir: &str) -> Result<SkillMeta, String> {
    let skill_md_path = Path::new(skill_dir).join("SKILL.md");
    let content =
        std::fs::read_to_string(&skill_md_path).map_err(|e| format!("Read SKILL.md: {}", e))?;

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("No YAML frontmatter found".to_string());
    }

    let rest = &trimmed[3..];
    let end = rest
        .find("---")
        .ok_or("Unterminated YAML frontmatter")?;
    let yaml_str = &rest[..end];

    serde_yaml::from_str(yaml_str).map_err(|e| format!("Parse frontmatter: {}", e))
}

/// Build a SkillMeta by reading from a remote cache repository
pub fn build_remote_skill_meta(
    repo_path: &str,
    entry: &SkillManifestEntry,
    repo_id: Option<&str>,
) -> Result<SkillMeta, String> {
    let skill_dir = Path::new(repo_path).join(&entry.path);

    let fm = parse_skill_frontmatter(skill_dir.to_string_lossy().as_ref()).ok();
    let checksum = crate::services::hash_service::aggregate_sha256(&skill_dir).ok();

    let mut meta = fm.unwrap_or_else(|| SkillMeta {
        name: entry.name.clone(),
        version: entry.version.clone(),
        description: entry.description.clone(),
        tags: entry.tags.clone(),
        path: entry.path.clone(),
        ..Default::default()
    });

    meta.path = entry.path.clone();
    meta.checksum = checksum.or_else(|| entry.checksum.clone());
    meta.install_status = Some(InstallStatus::NotInstalled);
    meta.source_repo_id = repo_id.map(|s| s.to_string());

    if meta.updated_at.is_none() {
        meta.updated_at = entry.updated_at.clone();
    }

    Ok(meta)
}

/// Build SkillMeta for a locally installed skill
pub fn build_local_skill_meta(
    base_dir: &str,
    skill_name: &str,
) -> Result<SkillMeta, String> {
    let skill_dir = Path::new(base_dir).join(skill_name);

    if !skill_dir.exists() {
        return Ok(SkillMeta {
            name: skill_name.to_string(),
            path: skill_name.to_string(),
            install_status: Some(InstallStatus::NotInstalled),
            ..Default::default()
        });
    }

    let fm = parse_skill_frontmatter(skill_dir.to_string_lossy().as_ref()).ok();
    let checksum = crate::services::hash_service::aggregate_sha256(&skill_dir).ok();

    let mut meta = fm.unwrap_or_else(|| SkillMeta {
        name: skill_name.to_string(),
        path: skill_name.to_string(),
        ..Default::default()
    });

    meta.path = skill_name.to_string();
    meta.checksum = checksum;

    if meta.updated_at.is_none() {
        meta.updated_at = skill_dir
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                dt.to_rfc3339()
            });
    }

    Ok(meta)
}

/// Build comparison list pairing local and remote skills from multiple repos
pub fn build_skill_comparisons(
    local_dir: &str,
    repos: &[crate::models::config::RepoConfig],
) -> Result<Vec<crate::models::skill::SkillComparison>, String> {
    use crate::models::skill::{ComparisonStatus, SkillComparison};

    // Scan local skills (all directories)
    let local_names = list_installed_skills(local_dir)?;
    let mut local_map: std::collections::HashMap<String, SkillMeta> = std::collections::HashMap::new();
    for name in &local_names {
        if let Ok(meta) = build_local_skill_meta(local_dir, name) {
            local_map.insert(meta.name.clone(), meta);
        }
    }

    // Load remote skills from all enabled repos (don't deduplicate by name)
    let mut all_remote_skills: Vec<SkillMeta> = Vec::new();
    for repo in repos {
        if !repo.enabled {
            continue;
        }
        let entries = load_skill_entries(&repo.cache_path);
        for entry in &entries {
            if let Ok(meta) = build_remote_skill_meta(&repo.cache_path, entry, Some(&repo.id)) {
                all_remote_skills.push(meta);
            }
        }
    }

    let mut comparisons = Vec::new();

    // Track which local skills have been matched with at least one remote
    let mut local_matched = std::collections::HashSet::new();

    // For each remote skill, create a comparison entry
    for remote in &all_remote_skills {
        let local = local_map.get(&remote.name).cloned();

        if local.is_some() {
            local_matched.insert(remote.name.clone());
        }

        let status = match (&local, Some(remote)) {
            (Some(l), Some(r)) => {
                let hash_match = match (&l.checksum, &r.checksum) {
                    (Some(lc), Some(rc)) => lc.value == rc.value,
                    _ => false,
                };
                // Hash match is the primary indicator: same content = no update needed
                if hash_match {
                    ComparisonStatus::Same
                } else {
                    ComparisonStatus::Outdated
                }
            }
            _ => ComparisonStatus::RemoteOnly,
        };

        comparisons.push(SkillComparison {
            name: remote.name.clone(),
            local,
            remote: Some(remote.clone()),
            status,
            source_repo_id: remote.source_repo_id.clone(),
        });
    }

    // Add local-only skills (no remote match in any repo)
    for (name, local) in &local_map {
        if !local_matched.contains(name) {
            comparisons.push(SkillComparison {
                name: name.clone(),
                local: Some(local.clone()),
                remote: None,
                status: ComparisonStatus::LocalOnly,
                source_repo_id: None,
            });
        }
    }

    Ok(comparisons)
}

/// Install a skill by copying from remote cache to target directory
pub fn install_skill_to_dir(
    remote_repo: &str,
    skill_path: &str,
    target_dir: &str,
) -> Result<(), String> {
    let src = Path::new(remote_repo).join(skill_path);
    let dst = Path::new(target_dir).join(
        Path::new(skill_path)
            .file_name()
            .ok_or("Invalid skill path")?,
    );

    if !src.exists() {
        return Err(format!("Source skill not found: {}", src.display()));
    }

    // Remove existing target directory to avoid orphaned files
    if dst.exists() {
        std::fs::remove_dir_all(&dst).map_err(|e| format!("Remove old dir: {}", e))?;
    }

    copy_dir_recursive(&src, &dst)
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if src.is_dir() {
        std::fs::create_dir_all(dst).map_err(|e| format!("Create dir: {}", e))?;
        for entry in
            std::fs::read_dir(src).map_err(|e| format!("Read dir: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Dir entry: {}", e))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    } else {
        std::fs::copy(src, dst).map_err(|e| format!("Copy file: {}", e))?;
    }
    Ok(())
}

/// List installed skill names from a directory (all subdirectories)
pub fn list_installed_skills(dir: &str) -> Result<Vec<String>, String> {
    let path = Path::new(dir);
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();
    for entry in
        std::fs::read_dir(path).map_err(|e| format!("Read dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Dir entry: {}", e))?;
        let p = entry.path();
        // List all subdirectories (any directory could be a skill)
        if p.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                // Skip hidden directories
                if !name.starts_with('.') {
                    skills.push(name.to_string());
                }
            }
        }
    }

    Ok(skills)
}

/// Parse "X.Y.Z" into (major, minor, patch)
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let v = version.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Check if a version string satisfies a semver range
/// Supports: "*", "^X.Y.Z", "~X.Y.Z", ">=X.Y.Z", exact "X.Y.Z"
fn semver_matches(version: &str, range: &str) -> bool {
    let ver = match parse_semver(version) {
        Some(v) => v,
        None => return false,
    };

    let range = range.trim();

    if range == "*" || range.is_empty() {
        return true;
    }

    if let Some(rest) = range.strip_prefix('^') {
        let target = match parse_semver(rest) {
            Some(v) => v,
            None => return false,
        };
        if target.0 != 0 {
            ver.0 == target.0 && ver >= target
        } else if target.1 != 0 {
            ver.0 == 0 && ver.1 == target.1 && ver >= target
        } else {
            ver == target
        }
    } else if let Some(rest) = range.strip_prefix('~') {
        let target = match parse_semver(rest) {
            Some(v) => v,
            None => return false,
        };
        ver.0 == target.0 && ver.1 == target.1 && ver >= target
    } else if let Some(rest) = range.strip_prefix(">=") {
        let target = match parse_semver(rest.trim()) {
            Some(v) => v,
            None => return false,
        };
        ver >= target
    } else {
        parse_semver(range).map(|t| ver == t).unwrap_or(false)
    }
}

/// Parse "@author/name" or "author/name" into (author, name).
/// Returns ("", name) if no slash is found.
fn parse_skill_reference(reference: &str) -> (String, String) {
    let trimmed = reference.trim_start_matches('@');
    if let Some(slash_pos) = trimmed.find('/') {
        return (
            trimmed[..slash_pos].to_string(),
            trimmed[slash_pos + 1..].to_string(),
        );
    }
    (String::new(), reference.to_string())
}

/// Parse skillbase.json from a project root directory
pub fn parse_skillbase_manifest(project_path: &str) -> Result<SkillbaseManifest, String> {
    let path = Path::new(project_path).join("skillbase.json");
    if !path.exists() {
        return Err("skillbase.json not found in project root".to_string());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Read skillbase.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse skillbase.json: {}", e))
}

/// Resolve all dependencies declared in a project's skillbase.json
pub fn resolve_skillbase_dependencies(
    project_path: &str,
    repos: &[RepoConfig],
    agent_project_patterns: &std::collections::HashMap<String, String>,
    active_agent_id: &str,
) -> Result<SkillbaseResolution, String> {
    let manifest = parse_skillbase_manifest(project_path)?;

    let pattern = agent_project_patterns
        .get(active_agent_id)
        .cloned()
        .unwrap_or_else(|| "{project}/.claude/skills".to_string());
    let local_dir = pattern.replace("{project}", project_path);

    // Filter repos based on manifest.registry (exact match against RepoConfig.url)
    let filtered_repos: Vec<&RepoConfig> = if let Some(ref registry_url) = manifest.registry {
        let matched: Vec<&RepoConfig> = repos
            .iter()
            .filter(|r| r.enabled && r.url == *registry_url)
            .collect();
        if matched.is_empty() {
            // Registry specified but no matching repo — fall back to all enabled repos
            repos.iter().filter(|r| r.enabled).collect()
        } else {
            matched
        }
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    // Build index of all remote skills: (author, name) -> SkillMeta
    let mut remote_index: std::collections::HashMap<(String, String), SkillMeta> =
        std::collections::HashMap::new();
    let mut remote_by_name: std::collections::HashMap<String, SkillMeta> =
        std::collections::HashMap::new();
    for repo in &filtered_repos {
        let entries = load_skill_entries(&repo.cache_path);
        for entry in &entries {
            if let Ok(meta) = build_remote_skill_meta(&repo.cache_path, entry, Some(&repo.id)) {
                let author = meta.author.clone().unwrap_or_default();
                remote_index
                    .entry((author.clone(), meta.name.clone()))
                    .or_insert_with(|| meta.clone());
                remote_by_name
                    .entry(meta.name.clone())
                    .or_insert_with(|| meta.clone());
            }
        }
    }

    // Build index of locally installed skills: name -> SkillMeta
    let local_names = list_installed_skills(&local_dir)?;
    let mut local_map: std::collections::HashMap<String, SkillMeta> =
        std::collections::HashMap::new();
    for name in &local_names {
        if let Ok(meta) = build_local_skill_meta(&local_dir, name) {
            local_map.insert(meta.name.clone(), meta);
        }
    }

    let mut dependencies = Vec::new();
    let mut satisfied_count = 0;
    let mut missing_count = 0;
    let mut mismatch_count = 0;
    let mut outdated_count = 0;

    for (reference, version_range) in &manifest.skills {
        let (author, skill_name) = parse_skill_reference(reference);

        // Try to find remote match: first by (author, name), then by name only
        let resolved = if !author.is_empty() {
            remote_index
                .get(&(author.clone(), skill_name.clone()))
                .cloned()
                .or_else(|| remote_by_name.get(&skill_name).cloned())
        } else {
            remote_by_name.get(&skill_name).cloned()
        };

        let installed = local_map.get(&skill_name).cloned();

        let status = match (&installed, &resolved) {
            // Installed but version doesn't satisfy the declared range
            (Some(inst), _) if !semver_matches(&inst.version, version_range) => {
                DependencyStatus::VersionMismatch
            }
            // Installed and satisfies range, but a newer compatible version is available
            (Some(inst), Some(res))
                if res.version != inst.version
                    && semver_matches(&res.version, version_range) =>
            {
                DependencyStatus::Outdated
            }
            // Installed and satisfies range, no newer version available
            (Some(_), _) => DependencyStatus::Satisfied,
            // Not installed
            (None, _) => DependencyStatus::Missing,
        };

        match &status {
            DependencyStatus::Satisfied => satisfied_count += 1,
            DependencyStatus::Missing => missing_count += 1,
            DependencyStatus::VersionMismatch => mismatch_count += 1,
            DependencyStatus::Outdated => outdated_count += 1,
        }

        dependencies.push(DependencyEntry {
            reference: reference.clone(),
            author,
            skill_name,
            version_range: version_range.clone(),
            resolved,
            installed,
            status,
        });
    }

    Ok(SkillbaseResolution {
        manifest,
        dependencies,
        satisfied_count,
        missing_count,
        mismatch_count,
        outdated_count,
    })
}

/// Generate skillbase.json content from currently installed skills.
/// Picks the registry URL from the repo with the highest overlap with installed skills.
pub fn generate_skillbase_manifest(
    project_path: &str,
    project_name: &str,
    agent_project_patterns: &std::collections::HashMap<String, String>,
    active_agent_id: &str,
    repos: &[crate::models::config::RepoConfig],
) -> Result<String, String> {
    let pattern = agent_project_patterns
        .get(active_agent_id)
        .cloned()
        .unwrap_or_else(|| "{project}/.claude/skills".to_string());
    let local_dir = pattern.replace("{project}", project_path);

    let local_names = list_installed_skills(&local_dir)?;
    let mut skills = std::collections::HashMap::new();

    for name in &local_names {
        if let Ok(meta) = build_local_skill_meta(&local_dir, name) {
            let author = meta.author.as_deref().unwrap_or("local");
            let version = if meta.version.is_empty() {
                "*".to_string()
            } else {
                format!("^{}", meta.version)
            };
            skills.insert(format!("@{}/{}", author, name), version);
        }
    }

    // Find the repo with the most matching skills to use as registry
    let registry_url = if !local_names.is_empty() {
        repos
            .iter()
            .filter(|r| r.enabled)
            .filter_map(|repo| {
                let entries = load_skill_entries(&repo.cache_path);
                let remote_names: std::collections::HashSet<&str> =
                    entries.iter().map(|e| e.name.as_str()).collect();
                let match_count = local_names
                    .iter()
                    .filter(|n| remote_names.contains(n.as_str()))
                    .count();
                if match_count > 0 {
                    Some((match_count, repo.url.clone()))
                } else {
                    None
                }
            })
            .max_by_key(|(count, _)| *count)
            .map(|(_, url)| url)
    } else {
        None
    };

    let manifest = SkillbaseManifest {
        schema_version: 1,
        name: project_name.to_string(),
        version: "1.0.0".to_string(),
        skills,
        personas: std::collections::HashMap::new(),
        registry: registry_url,
        spm: None,
    };

    serde_json::to_string_pretty(&manifest).map_err(|e| format!("Serialize: {}", e))
}

/// Summary for a single project's skills
#[derive(serde::Serialize)]
pub struct ProjectSkillSummary {
    pub project_path: String,
    pub project_name: String,
    pub local_count: usize,
    pub matched_count: usize,
    pub outdated_count: usize,
    pub remote_only_count: usize,
}

/// Build skill summary for multiple projects
pub fn build_projects_overview(
    project_paths: &[String],
    repos: &[crate::models::config::RepoConfig],
    _active_id: &str,
    agent_project_patterns: &std::collections::HashMap<String, String>,
) -> Result<Vec<ProjectSkillSummary>, String> {
    let mut summaries = Vec::new();

    for pp in project_paths {
        // Resolve project skills dir from pattern
        let pattern = agent_project_patterns
            .get(_active_id)
            .cloned()
            .unwrap_or_else(|| "{project}/.claude/skills".to_string());
        let skills_dir = pattern.replace("{project}", pp);
        let path = Path::new(&skills_dir);

        let local_names = if path.exists() {
            list_installed_skills(&skills_dir)?
        } else {
            vec![]
        };

        let local_count = local_names.len();

        // Load remote names from all enabled repos
        let mut remote_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for repo in repos {
            if !repo.enabled {
                continue;
            }
            let entries = load_skill_entries(&repo.cache_path);
            for s in &entries {
                remote_names.insert(s.name.clone());
            }
        }

        let mut matched = 0usize;
        let mut outdated = 0usize;
        let mut remote_only = 0usize;

        for name in &local_names {
            if remote_names.contains(name) {
                if let Ok(comparisons) = build_skill_comparisons(&skills_dir, repos) {
                    if let Some(comp) = comparisons.iter().find(|c| &c.name == name) {
                        match comp.status {
                            crate::models::skill::ComparisonStatus::Same => matched += 1,
                            crate::models::skill::ComparisonStatus::Outdated => outdated += 1,
                            _ => matched += 1,
                        }
                    } else {
                        matched += 1;
                    }
                } else {
                    matched += 1;
                }
            }
        }

        // Count remote-only
        for rn in &remote_names {
            if !local_names.contains(rn) {
                remote_only += 1;
            }
        }

        let project_name = Path::new(pp)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| pp.clone());

        summaries.push(ProjectSkillSummary {
            project_path: pp.clone(),
            project_name,
            local_count,
            matched_count: matched,
            outdated_count: outdated,
            remote_only_count: remote_only,
        });
    }

    Ok(summaries)
}

/// Build file-level diff between a local and remote skill directory
pub fn build_skill_diff(
    local_dir: &Path,
    remote_dir: &Path,
) -> Result<SkillDiff, String> {
    let local_files = crate::services::hash_service::list_file_hashes(local_dir).unwrap_or_default();
    let remote_files = crate::services::hash_service::list_file_hashes(remote_dir).unwrap_or_default();

    let local_map: std::collections::HashMap<&str, &crate::models::skill::FileEntry> =
        local_files.iter().map(|f| (f.path.as_str(), f)).collect();
    let remote_map: std::collections::HashMap<&str, &crate::models::skill::FileEntry> =
        remote_files.iter().map(|f| (f.path.as_str(), f)).collect();

    // Collect all unique paths
    let mut all_paths: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for f in &local_files {
        all_paths.insert(f.path.as_str());
    }
    for f in &remote_files {
        all_paths.insert(f.path.as_str());
    }

    let mut files = Vec::new();
    let mut added_count = 0u32;
    let mut removed_count = 0u32;
    let mut modified_count = 0u32;
    let mut unchanged_count = 0u32;

    for path in &all_paths {
        let local_entry = local_map.get(path);
        let remote_entry = remote_map.get(path);

        let (status, local_hash, remote_hash, local_size, remote_size) =
            match (local_entry, remote_entry) {
                (Some(l), Some(r)) => {
                    // Strip "sha256:" prefix for comparison
                    let lh = l.hash.strip_prefix("sha256:").unwrap_or(&l.hash);
                    let rh = r.hash.strip_prefix("sha256:").unwrap_or(&r.hash);
                    if lh == rh {
                        (FileDiffStatus::Unchanged, Some(l.hash.clone()), Some(r.hash.clone()), Some(l.size), Some(r.size))
                    } else {
                        (FileDiffStatus::Modified, Some(l.hash.clone()), Some(r.hash.clone()), Some(l.size), Some(r.size))
                    }
                }
                (None, Some(r)) => {
                    (FileDiffStatus::Added, None, Some(r.hash.clone()), None, Some(r.size))
                }
                (Some(l), None) => {
                    (FileDiffStatus::Removed, Some(l.hash.clone()), None, Some(l.size), None)
                }
                (None, None) => continue,
            };

        match status {
            FileDiffStatus::Added => added_count += 1,
            FileDiffStatus::Removed => removed_count += 1,
            FileDiffStatus::Modified => modified_count += 1,
            FileDiffStatus::Unchanged => unchanged_count += 1,
        }

        files.push(FileDiff {
            path: path.to_string(),
            local_hash,
            remote_hash,
            local_size,
            remote_size,
            status,
        });
    }

    // Extract versions from frontmatter
    let local_version = parse_skill_frontmatter(local_dir.to_string_lossy().as_ref())
        .ok()
        .map(|f| f.version);
    let remote_version = parse_skill_frontmatter(remote_dir.to_string_lossy().as_ref())
        .ok()
        .map(|f| f.version);

    let skill_name = local_dir
        .file_name()
        .or_else(|| remote_dir.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(SkillDiff {
        skill_name,
        local_version,
        remote_version,
        files,
        added_count,
        removed_count,
        modified_count,
        unchanged_count,
    })
}
