import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, AgentInfo } from '@/types'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    remote_url: '',
    cache_path: '',
    auto_sync: false,
    active_agent_id: 'claude',
    agent_global_paths: {},
    agent_project_patterns: {},
    agent_display_names: {},
    custom_agent_ids: [],
  })
  const allAgents = ref<AgentInfo[]>([])
  const loading = ref(false)

  async function loadConfig() {
    loading.value = true
    try {
      config.value = await invoke<AppConfig>('get_config')
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(newConfig: AppConfig) {
    await invoke('set_config', { config: newConfig })
    config.value = newConfig
  }

  async function loadAllAgents() {
    allAgents.value = await invoke<AgentInfo[]>('get_all_agents')
  }

  async function setActiveAgent(id: string) {
    const newConfig = { ...config.value, active_agent_id: id }
    await saveConfig(newConfig)
  }

  async function addCustomAgent(id: string, displayName: string, globalPath: string, projectPattern: string) {
    await invoke('add_custom_agent', {
      id, displayName, globalPath, projectPattern,
    })
    await loadConfig()
    await loadAllAgents()
  }

  async function removeCustomAgent(id: string) {
    await invoke('remove_custom_agent', { id })
    await loadConfig()
    await loadAllAgents()
  }

  function getGlobalPath(): string {
    return config.value.agent_global_paths[config.value.active_agent_id] || ''
  }

  function getProjectPattern(): string {
    return config.value.agent_project_patterns[config.value.active_agent_id] || '{project}/.claude/skills'
  }

  function getActiveDisplayName(): string {
    const id = config.value.active_agent_id
    return config.value.agent_display_names[id]
      || allAgents.value.find(a => a.id === id)?.display_name
      || id
  }

  return {
    config,
    allAgents,
    loading,
    loadConfig,
    saveConfig,
    loadAllAgents,
    setActiveAgent,
    addCustomAgent,
    removeCustomAgent,
    getGlobalPath,
    getProjectPattern,
    getActiveDisplayName,
  }
})
