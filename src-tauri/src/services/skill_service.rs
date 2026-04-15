use std::path::Path;

use crate::models::skill::{
    FileDiff, FileDiffStatus, InstallStatus, SkillDiff, SkillFrontmatter, SkillManifestEntry,
    SkillMeta, SkillsManifest,
};

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

/// Parse SKILL.md YAML frontmatter from a skill directory
pub fn parse_skill_frontmatter(skill_dir: &str) -> Result<SkillFrontmatter, String> {
    let skill_md_path = Path::new(skill_dir).join("SKILL.md");
    let content =
        std::fs::read_to_string(&skill_md_path).map_err(|e| format!("Read SKILL.md: {}", e))?;

    parse_frontmatter_from_string(&content)
}

/// Extract YAML frontmatter from SKILL.md content
fn parse_frontmatter_from_string(content: &str) -> Result<SkillFrontmatter, String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("No YAML frontmatter found".to_string());
    }

    let rest = &trimmed[3..];
    let end = rest
        .find("---")
        .ok_or("Unterminated YAML frontmatter")?;
    let yaml_str = &rest[..end];

    let fm = parse_simple_yaml_frontmatter(yaml_str)?;
    Ok(fm)
}

/// Minimal YAML frontmatter parser
fn parse_simple_yaml_frontmatter(yaml: &str) -> Result<SkillFrontmatter, String> {
    let mut name = String::new();
    let mut version = String::new();
    let mut description = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut license: Option<String> = None;
    let mut updated_at: Option<String> = None;

    for line in yaml.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("version:") {
            version = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("license:") {
            license = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("updated_at:") {
            updated_at = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("tags:") {
            let tag_str = val.trim();
            if tag_str.starts_with('[') && tag_str.ends_with(']') {
                let inner = &tag_str[1..tag_str.len() - 1];
                tags = inner
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
        }
    }

    Ok(SkillFrontmatter {
        name,
        version,
        description,
        tags,
        license,
        updated_at,
    })
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

    let (name, version, description, tags, license, updated_at) = if let Some(ref f) = fm {
        (
            f.name.clone(),
            f.version.clone(),
            f.description.clone(),
            f.tags.clone(),
            f.license.clone(),
            f.updated_at.clone(),
        )
    } else {
        (
            entry.name.clone(),
            entry.version.clone(),
            entry.description.clone(),
            entry.tags.clone(),
            None,
            entry.updated_at.clone(),
        )
    };

    Ok(SkillMeta {
        name,
        version,
        description,
        tags,
        path: entry.path.clone(),
        license,
        updated_at: updated_at.or_else(|| entry.updated_at.clone()),
        checksum: checksum.or_else(|| entry.checksum.clone()),
        files: None,
        install_status: Some(InstallStatus::NotInstalled),
        source_repo_id: repo_id.map(|s| s.to_string()),
    })
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
            version: String::new(),
            description: String::new(),
            tags: vec![],
            path: skill_name.to_string(),
            license: None,
            updated_at: None,
            checksum: None,
            files: None,
            install_status: Some(InstallStatus::NotInstalled),
            source_repo_id: None,
        });
    }

    let fm = parse_skill_frontmatter(skill_dir.to_string_lossy().as_ref()).ok();
    let checksum = crate::services::hash_service::aggregate_sha256(&skill_dir).ok();

    let name = fm.as_ref().map(|f| f.name.clone()).unwrap_or(skill_name.to_string());
    let version = fm.as_ref().map(|f| f.version.clone()).unwrap_or_default();
    let description = fm.as_ref().map(|f| f.description.clone()).unwrap_or_default();
    let tags = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    let license = fm.as_ref().and_then(|f| f.license.clone());
    let updated_at = fm.as_ref().and_then(|f| f.updated_at.clone());

    // Use mtime as fallback for updated_at
    let updated_at = updated_at.or_else(|| {
        skill_dir
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                dt.to_rfc3339()
            })
    });

    Ok(SkillMeta {
        name,
        version,
        description,
        tags,
        path: skill_name.to_string(),
        license,
        updated_at,
        checksum,
        files: None,
        install_status: None,
        source_repo_id: None,
    })
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
