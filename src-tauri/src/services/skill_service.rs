use std::path::Path;

use crate::models::skill::{
    InstallStatus, SkillFrontmatter, SkillManifestEntry, SkillMeta, SkillsManifest,
};

/// Parse skills.json from a repository root
pub fn parse_manifest(repo_path: &str) -> Result<SkillsManifest, String> {
    let manifest_path = Path::new(repo_path).join("skills.json");
    let content =
        std::fs::read_to_string(&manifest_path).map_err(|e| format!("Read skills.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse skills.json: {}", e))
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
    })
}

/// Build comparison list pairing local and remote skills
pub fn build_skill_comparisons(
    local_dir: &str,
    cache_path: &str,
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

    // Load remote skills
    let remote_skills: Vec<SkillMeta> = parse_manifest(cache_path)
        .map(|manifest| {
            manifest
                .skills
                .iter()
                .filter_map(|e| build_remote_skill_meta(cache_path, e).ok())
                .collect()
        })
        .unwrap_or_default();

    let mut remote_map: std::collections::HashMap<String, SkillMeta> = std::collections::HashMap::new();
    for skill in &remote_skills {
        remote_map.insert(skill.name.clone(), skill.clone());
    }

    // Build comparison: collect all unique names
    let mut all_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for name in local_map.keys() {
        all_names.insert(name.clone());
    }
    for name in remote_map.keys() {
        all_names.insert(name.clone());
    }

    let mut comparisons = Vec::new();

    for name in &all_names {
        let local = local_map.get(name).cloned();
        let remote = remote_map.get(name).cloned();

        let status = match (&local, &remote) {
            (Some(l), Some(r)) => {
                // Both exist: compare version and hash
                let version_match = !l.version.is_empty() && l.version == r.version;
                let hash_match = match (&l.checksum, &r.checksum) {
                    (Some(lc), Some(rc)) => lc.value == rc.value,
                    _ => false,
                };

                if version_match && hash_match {
                    ComparisonStatus::Same
                } else if version_match && !hash_match {
                    ComparisonStatus::Outdated // Same version but different content
                } else {
                    ComparisonStatus::Outdated
                }
            }
            (Some(_), None) => ComparisonStatus::LocalOnly,
            (None, Some(_)) => ComparisonStatus::RemoteOnly,
            (None, None) => ComparisonStatus::Unknown,
        };

        comparisons.push(SkillComparison {
            name: name.clone(),
            local,
            remote,
            status,
        });
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
    cache_path: &str,
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

        // Load remote names for matching
        let remote_names: std::collections::HashSet<String> = parse_manifest(cache_path)
            .map(|m| m.skills.iter().map(|s| s.name.clone()).collect())
            .unwrap_or_default();

        let mut matched = 0usize;
        let mut outdated = 0usize;
        let mut remote_only = 0usize;

        for name in &local_names {
            if remote_names.contains(name) {
                // Check if outdated by comparing versions
                if let Ok(comparisons) = build_skill_comparisons(&skills_dir, cache_path) {
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
