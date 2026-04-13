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
}

export interface AgentInfo {
  id: string
  display_name: string
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
}

export type ComparisonStatus = 'Same' | 'Outdated' | 'LocalOnly' | 'RemoteOnly' | 'Unknown'

export interface SkillComparison {
  name: string
  local: SkillMeta | null
  remote: SkillMeta | null
  status: ComparisonStatus
}

export interface ProjectSkillSummary {
  project_path: string
  project_name: string
  local_count: number
  matched_count: number
  outdated_count: number
  remote_only_count: number
}
