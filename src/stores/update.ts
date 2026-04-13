import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { UpdateInfo } from '@/types'

export const useUpdateStore = defineStore('update', () => {
  const updateInfo = ref<UpdateInfo | null>(null)
  const checking = ref(false)
  const currentVersion = ref('')

  async function loadCurrentVersion() {
    currentVersion.value = await invoke<string>('get_current_version')
  }

  async function checkForUpdates() {
    checking.value = true
    try {
      updateInfo.value = await invoke<UpdateInfo>('check_for_updates')
    } finally {
      checking.value = false
    }
  }

  return { updateInfo, checking, currentVersion, loadCurrentVersion, checkForUpdates }
})
