use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub path: String,
    pub license: Option<String>,
    pub updated_at: Option<String>,
    pub checksum: Option<Checksum>,
    pub files: Option<Vec<FileEntry>>,
    pub install_status: Option<InstallStatus>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstallStatus {
    Installed,
    Outdated,
    NotInstalled,
    Unknown,
}

/// Entry in skills.json manifest
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifestEntry {
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub checksum: Option<Checksum>,
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

/// YAML frontmatter parsed from SKILL.md
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillFrontmatter {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
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
}
