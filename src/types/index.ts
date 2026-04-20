export interface Checksum {
  algorithm: string
  value: string
}

export interface FileEntry {
  path: string
  hash: string
  size: number
  mtime: string
}

export interface TriggerInfo {
  description: string
  tags: string[]
  file_patterns: string[]
  priority?: number
}

export interface SecurityInfo {
  permissions: string[]
}

export interface CompatibilityInfo {
  min_context_tokens?: number
  requires: string[]
  models: string[]
}

export type InstallStatus = 'installed' | 'outdated' | 'not_installed' | 'unknown'

export interface SkillMeta {
  name: string
  version: string
  description: string
  tags: string[]
  path: string
  license?: string
  updated_at?: string
  checksum?: Checksum
  files?: FileEntry[]
  install_status?: InstallStatus
  source_repo_id?: string
  author?: string
  language?: string
  trigger?: TriggerInfo
  security?: SecurityInfo
  compatibility?: CompatibilityInfo
  dependencies?: Record<string, string>
  repository?: string
}

export interface AgentInfo {
  id: string
  display_name: string
}

export interface RepoConfig {
  id: string
  name: string
  url: string
  cache_path: string
  enabled: boolean
}

export interface AppConfig {
  remote_url: string
  cache_path: string
  auto_sync: boolean
  active_agent_id: string
  agent_global_paths: Record<string, string>
  agent_project_patterns: Record<string, string>
  agent_display_names: Record<string, string>
  custom_agent_ids: string[]
  repos: RepoConfig[]
}

export type ComparisonStatus = 'Same' | 'Outdated' | 'LocalOnly' | 'RemoteOnly' | 'Unknown'

export interface SkillComparison {
  name: string
  local: SkillMeta | null
  remote: SkillMeta | null
  status: ComparisonStatus
  source_repo_id?: string
}

export interface ProjectSkillSummary {
  project_path: string
  project_name: string
  local_count: number
  matched_count: number
  outdated_count: number
  remote_only_count: number
}

export interface SyncResult {
  success_count: number
  fail_count: number
  errors: string[]
}

export interface UpdateInfo {
  current_version: string
  latest_version: string
  has_update: boolean
  release_url: string
  release_notes: string | null
  published_at: string | null
  error: string | null
}

// Diff Viewer types
export type FileDiffStatus = 'Unchanged' | 'Added' | 'Removed' | 'Modified'

export interface FileDiff {
  path: string
  localHash?: string
  remoteHash?: string
  localSize?: number
  remoteSize?: number
  status: FileDiffStatus
}

export interface SkillDiff {
  skillName: string
  localVersion?: string
  remoteVersion?: string
  files: FileDiff[]
  addedCount: number
  removedCount: number
  modifiedCount: number
  unchangedCount: number
}

export interface DiffFileContent {
  localContent?: string
  remoteContent?: string
}

// Operation History types
export type OperationType = 'Install' | 'Update' | 'Uninstall'

export interface OperationRecord {
  id: string
  operation: OperationType
  skillName: string
  target: string
  timestamp: string
  repoId?: string
  versionBefore?: string
  versionAfter?: string
  rollbackAvailable: boolean
}

// Skillbase types
export interface SpmConfig {
  default_instance?: string
}

export interface SkillbaseManifest {
  schema_version: number
  name: string
  version: string
  skills: Record<string, string>
  personas: Record<string, string>
  registry?: string
  spm?: SpmConfig
}

export type DependencyStatus = 'Satisfied' | 'Missing' | 'VersionMismatch' | 'Outdated'

export interface DependencyEntry {
  reference: string
  author: string
  skillName: string
  versionRange: string
  resolved: SkillMeta | null
  installed: SkillMeta | null
  status: DependencyStatus
}

export interface SkillbaseResolution {
  manifest: SkillbaseManifest
  dependencies: DependencyEntry[]
  satisfiedCount: number
  missingCount: number
  mismatchCount: number
  outdatedCount: number
}

export interface ProjectDetailData {
  comparisons: SkillComparison[]
  skillbase: SkillbaseResolution | null
}
