import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SkillComparison, ProjectSkillSummary, SyncResult, SkillDiff, OperationRecord, SkillbaseResolution } from '@/types'

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
    syncSkillbase,
    generateSkillbase,
    writeSkillbase,
  }
})
