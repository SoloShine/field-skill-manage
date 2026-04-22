import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SkillComparison, ProjectSkillSummary, SyncResult, SkillDiff, OperationRecord, SkillbaseResolution, ProjectDetailData, ScanAgentSkillsResult, MigrateResult, ConflictResolution } from '@/types'

export const useSkillStore = defineStore('skill', () => {
  const globalComparisons = ref<SkillComparison[]>([])
  const projectComparisons = ref<SkillComparison[]>([])
  const projectsOverview = ref<ProjectSkillSummary[]>([])
  const syncing = ref(false)
  const loading = ref(false)
  const lastSyncResult = ref<SyncResult | null>(null)
  const skillDiff = ref<SkillDiff | null>(null)
  const skillbaseResolution = ref<SkillbaseResolution | null>(null)
  const skillbaseSyncing = ref(false)

  const migrateDialogVisible = ref(false)
  const migrateScope = ref<'global' | 'project'>('global')
  const migrateProjectPath = ref<string | null>(null)
  const scanResult = ref<ScanAgentSkillsResult | null>(null)
  const migrating = ref(false)

  async function syncRemote() {
    syncing.value = true
    try {
      lastSyncResult.value = await invoke<SyncResult>('sync_remote_repo')
    } finally {
      syncing.value = false
    }
  }

  async function loadGlobalSkills() {
    loading.value = true
    try {
      globalComparisons.value = await invoke<SkillComparison[]>('get_global_skills')
    } finally {
      loading.value = false
    }
  }

  async function loadProjectSkills(projectPath: string) {
    loading.value = true
    try {
      projectComparisons.value = await invoke<SkillComparison[]>('get_project_skills', {
        projectPath,
      })
    } finally {
      loading.value = false
    }
  }

  async function loadProjectsOverview(projectPaths: string[]) {
    loading.value = true
    try {
      projectsOverview.value = await invoke<ProjectSkillSummary[]>('get_projects_overview', {
        projectPaths,
      })
    } finally {
      loading.value = false
    }
  }

  async function installSkill(name: string, target: string, repoId?: string) {
    await invoke('install_skill', { skillName: name, target, repoId: repoId || null })
  }

  async function updateSkill(name: string, target: string, repoId?: string) {
    await invoke('update_skill', { skillName: name, target, repoId: repoId || null })
  }

  async function batchUpdate(names: string[], target: string, repoId?: string) {
    return await invoke<string[]>('batch_update', { skillNames: names, target, repoId: repoId || null })
  }

  async function uninstallSkill(name: string, target: string) {
    await invoke('uninstall_skill', { skillName: name, target })
  }

  async function loadSkillDiff(name: string, target: string) {
    skillDiff.value = await invoke<SkillDiff>('get_skill_diff', { skillName: name, target })
  }

  async function getOperationHistory(limit?: number): Promise<OperationRecord[]> {
    return await invoke<OperationRecord[]>('get_operation_history', { limit: limit || null })
  }

  async function rollbackOperation(id: string) {
    await invoke('rollback_operation', { operationId: id })
  }

  async function clearHistory() {
    await invoke('clear_history')
  }

  async function loadSkillbase(projectPath: string) {
    try {
      skillbaseResolution.value = await invoke<SkillbaseResolution>('get_skillbase_resolution', {
        projectPath,
      })
    } catch {
      skillbaseResolution.value = null
    }
  }

  async function loadProjectDetail(projectPath: string) {
    loading.value = true
    try {
      const data = await invoke<ProjectDetailData>('get_project_detail', {
        projectPath,
      })
      projectComparisons.value = data.comparisons
      skillbaseResolution.value = data.skillbase
    } finally {
      loading.value = false
    }
  }

  async function syncSkillbase(projectPath: string): Promise<string[]> {
    skillbaseSyncing.value = true
    try {
      return await invoke<string[]>('sync_skillbase_dependencies', { projectPath })
    } finally {
      skillbaseSyncing.value = false
    }
  }

  async function generateSkillbase(projectPath: string): Promise<string> {
    return await invoke<string>('generate_skillbase_json', { projectPath })
  }

  async function writeSkillbase(projectPath: string, content: string): Promise<void> {
    await invoke('write_skillbase_json', { projectPath, content })
  }

  function openMigrateDialog(scope: 'global' | 'project', projectPath?: string) {
    migrateScope.value = scope
    migrateProjectPath.value = projectPath ?? null
    scanResult.value = null
    migrateDialogVisible.value = true
  }

  function closeMigrateDialog() {
    migrateDialogVisible.value = false
    scanResult.value = null
  }

  async function scanAgentSkills(agentId: string) {
    const args: Record<string, unknown> = {
      agentId,
      scope: migrateScope.value,
    }
    if (migrateScope.value === 'project' && migrateProjectPath.value) {
      args.projectPath = migrateProjectPath.value
    }
    scanResult.value = await invoke<ScanAgentSkillsResult>('scan_agent_skills', args)
  }

  async function migrateSkillsAction(
    sourceAgentId: string,
    skillNames: string[],
    conflictMap: Record<string, ConflictResolution>,
  ) {
    migrating.value = true
    try {
      const args: Record<string, unknown> = {
        sourceAgentId,
        skillNames,
        scope: migrateScope.value,
        conflictMap,
      }
      if (migrateScope.value === 'project' && migrateProjectPath.value) {
        args.projectPath = migrateProjectPath.value
      }
      return await invoke<MigrateResult>('migrate_skills', args)
    } finally {
      migrating.value = false
    }
  }

  async function loadMigrateSkillDiff(sourceAgentId: string, skillName: string) {
    const args: Record<string, unknown> = {
      sourceAgentId,
      skillName,
      scope: migrateScope.value,
    }
    if (migrateScope.value === 'project' && migrateProjectPath.value) {
      args.projectPath = migrateProjectPath.value
    }
    return await invoke<SkillDiff>('get_migrate_skill_diff', args)
  }

  async function loadMigrateDiffContent(sourceAgentId: string, skillName: string, filePath: string) {
    const args: Record<string, unknown> = {
      sourceAgentId,
      skillName,
      filePath,
      scope: migrateScope.value,
    }
    if (migrateScope.value === 'project' && migrateProjectPath.value) {
      args.projectPath = migrateProjectPath.value
    }
    return await invoke<import('@/types').DiffFileContent>('get_migrate_diff_content', args)
  }

  return {
    globalComparisons,
    projectComparisons,
    projectsOverview,
    syncing,
    loading,
    lastSyncResult,
    skillDiff,
    skillbaseResolution,
    skillbaseSyncing,
    syncRemote,
    loadGlobalSkills,
    loadProjectSkills,
    loadProjectsOverview,
    installSkill,
    updateSkill,
    batchUpdate,
    uninstallSkill,
    loadSkillDiff,
    getOperationHistory,
    rollbackOperation,
    clearHistory,
    loadSkillbase,
    loadProjectDetail,
    syncSkillbase,
    generateSkillbase,
    writeSkillbase,
    migrateDialogVisible,
    migrateScope,
    migrateProjectPath,
    scanResult,
    migrating,
    openMigrateDialog,
    closeMigrateDialog,
    scanAgentSkills,
    migrateSkillsAction,
    loadMigrateSkillDiff,
    loadMigrateDiffContent,
  }
})
