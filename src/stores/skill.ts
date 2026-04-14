import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SkillComparison, ProjectSkillSummary, SyncResult } from '@/types'

export const useSkillStore = defineStore('skill', () => {
  const globalComparisons = ref<SkillComparison[]>([])
  const projectComparisons = ref<SkillComparison[]>([])
  const projectsOverview = ref<ProjectSkillSummary[]>([])
  const syncing = ref(false)
  const loading = ref(false)
  const lastSyncResult = ref<SyncResult | null>(null)

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

  return {
    globalComparisons,
    projectComparisons,
    projectsOverview,
    syncing,
    loading,
    lastSyncResult,
    syncRemote,
    loadGlobalSkills,
    loadProjectSkills,
    loadProjectsOverview,
    installSkill,
    updateSkill,
    batchUpdate,
    uninstallSkill,
  }
})
