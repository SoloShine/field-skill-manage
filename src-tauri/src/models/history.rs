use serde::{Deserialize, Serialize};

/// Type of operation performed on a skill
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

/// A single operation record in the history
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub id: String,
    pub operation: OperationType,
    pub skill_name: String,
    /// "global" or a project path
    pub target: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Source repository id
    pub repo_id: Option<String>,
    pub version_before: Option<String>,
    pub version_after: Option<String>,
    pub rollback_available: bool,
}

/// Container for all operation history
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OperationHistory {
    pub records: Vec<OperationRecord>,
}
