import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useProjectStore = defineStore('project', () => {
  const projectPath = ref('')
  const projectPaths = ref<string[]>([])

  function setProjectPath(path: string) {
    projectPath.value = path
    if (path && !projectPaths.value.includes(path)) {
      projectPaths.value.push(path)
      persist()
    }
  }

  function removeProjectPath(path: string) {
    projectPaths.value = projectPaths.value.filter((p) => p !== path)
    if (projectPath.value === path) {
      projectPath.value = ''
    }
    persist()
  }

  function persist() {
    try {
      localStorage.setItem('spm_project_paths', JSON.stringify(projectPaths.value))
    } catch {
      // ignore
    }
  }

  function loadPersisted() {
    try {
      const saved = localStorage.getItem('spm_project_paths')
      if (saved) {
        projectPaths.value = JSON.parse(saved)
      }
    } catch {
      // ignore
    }
  }

  return { projectPath, projectPaths, setProjectPath, removeProjectPath, loadPersisted }
})
