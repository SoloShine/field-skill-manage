<script setup lang="ts">
import { onMounted, ref, computed, watch, onUnmounted } from 'vue'
import { NButton, NInput, NSpace, NSpin, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import { useProjectStore } from '@/stores/project'
import { open } from '@tauri-apps/plugin-dialog'
import SkillCompareTable from '@/components/common/SkillCompareTable.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import type { SkillComparison } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const projectStore = useProjectStore()
const message = useMessage()

const searchText = ref('')

// Sticky header: compute table max-height from window
const winHeight = ref(window.innerHeight)
function onResize() { winHeight.value = window.innerHeight }
window.addEventListener('resize', onResize)
onUnmounted(() => window.removeEventListener('resize', onResize))
const tableMaxHeight = computed(() => Math.max(300, winHeight.value - 280))

const filtered = computed(() => {
  let list: SkillComparison[] = skillStore.projectComparisons
  if (searchText.value) {
    const q = searchText.value.toLowerCase()
    list = list.filter((s) => s.name.toLowerCase().includes(q))
  }
  return list
})

const stats = computed(() => {
  const all = skillStore.projectComparisons
  return {
    total: all.length,
    same: all.filter((s) => s.status === 'Same').length,
    outdated: all.filter((s) => s.status === 'Outdated').length,
    localOnly: all.filter((s) => s.status === 'LocalOnly').length,
    remoteOnly: all.filter((s) => s.status === 'RemoteOnly').length,
  }
})

async function handleSelectProject() {
  try {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      projectStore.setProjectPath(selected)
    }
  } catch {
    // cancelled
  }
}

async function loadProjectSkills() {
  if (!projectStore.projectPath) return
  try {
    await skillStore.loadProjectSkills(projectStore.projectPath)
  } catch (e: any) {
    message.error(t('project.loadFailed', { error: e }))
  }
}

async function handleSync() {
  try {
    await skillStore.syncRemote()
    await loadProjectSkills()
    message.success(t('global.syncComplete'))
  } catch (e: any) {
    message.error(t('global.syncFailed', { error: e }))
  }
}

async function handleInstall(name: string, _target: string, repoId?: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.installSkill(name, projectStore.projectPath, repoId)
    await loadProjectSkills()
    message.success(t('global.installSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.installFailed', { error: e }))
  }
}

async function handleUpdate(name: string, _target: string, repoId?: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.updateSkill(name, projectStore.projectPath, repoId)
    await loadProjectSkills()
    message.success(t('global.updateSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.updateFailed', { error: e }))
  }
}

async function handleUninstall(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.uninstallSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(t('global.uninstallSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.uninstallFailed', { error: e }))
  }
}

watch(() => projectStore.projectPath, loadProjectSkills)

onMounted(async () => {
  await configStore.loadConfig()
  if (projectStore.projectPath) {
    await loadProjectSkills()
  }
})
</script>

<template>
  <div class="project-view">
    <div class="page-header">
      <h1>{{ t('project.skillTitle') }}</h1>
    </div>

    <div class="project-selector">
      <NSpace align="center">
        <NInput :value="projectStore.projectPath" :placeholder="t('project.selectPathPlaceholder')" readonly style="width: 400px" />
        <NButton type="primary" @click="handleSelectProject">{{ t('common.browse') }}</NButton>
        <NButton :disabled="!projectStore.projectPath" :loading="skillStore.syncing" @click="handleSync">
          {{ t('common.syncRemote') }}
        </NButton>
      </NSpace>
    </div>

    <template v-if="projectStore.projectPath">
      <div class="stats-bar">
        <span>{{ t('stats.total', { count: stats.total }) }}</span>
        <span class="stat-item stat-same">{{ t('stats.same', { count: stats.same }) }}</span>
        <span class="stat-item stat-outdated">{{ t('stats.outdated', { count: stats.outdated }) }}</span>
        <span class="stat-item stat-local">{{ t('stats.localOnly', { count: stats.localOnly }) }}</span>
        <span class="stat-item stat-remote">{{ t('stats.remoteOnly', { count: stats.remoteOnly }) }}</span>
      </div>

      <div class="toolbar">
        <NInput v-model:value="searchText" :placeholder="t('common.search')" clearable style="width: 160px" />
      </div>

      <NSpin :show="skillStore.loading">
        <SkillCompareTable
          v-if="filtered.length > 0"
          :comparisons="filtered"
          :target="projectStore.projectPath"
          :max-height="tableMaxHeight"
          @install="handleInstall"
          @update="handleUpdate"
          @uninstall="handleUninstall"
        />
        <EmptyState v-else :title="t('project.emptySkillTitle')" :description="t('project.emptySkillDesc')" />
      </NSpin>
    </template>

    <EmptyState v-else :title="t('project.selectPathTitle')" :description="t('project.selectPathDesc')" />
  </div>
</template>

<style scoped>
.project-view {
  max-width: 1100px;
}
.page-header {
  margin-bottom: 12px;
}
.page-header h1 {
  font-size: 22px;
  font-weight: 600;
}
.project-selector {
  margin-bottom: 16px;
  padding: 16px;
  background: #fff;
  border-radius: 8px;
  border: 1px solid #e8e8e8;
}
.stats-bar {
  margin-bottom: 12px;
  font-size: 13px;
  color: #666;
  display: flex;
  gap: 16px;
}
.stat-item { font-weight: 500; }
.stat-same { color: #18a058; }
.stat-outdated { color: #f0a020; }
.stat-local { color: #2080f0; }
.stat-remote { color: #999; }
.toolbar {
  margin-bottom: 16px;
}
</style>
