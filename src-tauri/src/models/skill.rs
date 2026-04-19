use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriggerInfo {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecurityInfo {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompatibilityInfo {
    #[serde(default)]
    pub min_context_tokens: Option<u32>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkillMeta {
    // 通用元数据字段（来自 SKILL.md frontmatter 或 skills.json）
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub trigger: Option<TriggerInfo>,
    #[serde(default)]
    pub security: Option<SecurityInfo>,
    #[serde(default)]
    pub compatibility: Option<CompatibilityInfo>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,

    // 运行时填充字段（service 层填入）
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub checksum: Option<Checksum>,
    #[serde(default)]
    pub files: Option<Vec<FileEntry>>,
    #[serde(default)]
    pub install_status: Option<InstallStatus>,
    #[serde(default)]
    pub source_repo_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Checksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub mtime: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum InstallStatus {
    #[default]
    Unknown,
    Installed,
    Outdated,
    NotInstalled,
}

/// Entry in skills.json manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifestEntry {
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub checksum: Option<Checksum>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub trigger: Option<TriggerInfo>,
    #[serde(default)]
    pub security: Option<SecurityInfo>,
}

/// Root of skills.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillsManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub skills: Vec<SkillManifestEntry>,
}

/// File-level diff status for comparing local vs remote skill
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileDiffStatus {
    Unchanged,
    Added,
    Removed,
    Modified,
}

/// Single file diff between local and remote versions
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub local_hash: Option<String>,
    pub remote_hash: Option<String>,
    pub local_size: Option<u64>,
    pub remote_size: Option<u64>,
    pub status: FileDiffStatus,
}

/// Complete diff result for a skill
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiff {
    pub skill_name: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub files: Vec<FileDiff>,
    pub added_count: u32,
    pub removed_count: u32,
    pub modified_count: u32,
    pub unchanged_count: u32,
}

/// Comparison status between local and remote
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComparisonStatus {
    /// Local and remote version match, hash matches
    Same,
    /// Version differs or hash mismatch
    Outdated,
    /// Only exists locally, no remote match
    LocalOnly,
    /// Only exists remotely, not installed locally
    RemoteOnly,
    /// Cannot determine (missing metadata)
    Unknown,
}

/// Paired local/remote skill for comparison view
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillComparison {
    pub name: String,
    /// Local skill info (None if not installed)
    pub local: Option<SkillMeta>,
    /// Remote skill info (None if not in remote repo)
    pub remote: Option<SkillMeta>,
    /// Comparison result
    pub status: ComparisonStatus,
    /// Which repo this skill comes from (for remote skills)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repo_id: Option<String>,
}
