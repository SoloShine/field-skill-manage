<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NInput, NSpin, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import { useProjectStore } from '@/stores/project'
import SkillCompareTable from '@/components/common/SkillCompareTable.vue'
import SkillPreviewModal from '@/components/common/SkillPreviewModal.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import type { SkillComparison } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const projectStore = useProjectStore()
const message = useMessage()

const searchText = ref('')
const previewSkill = ref<string | null>(null)

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

async function loadProjectSkills() {
  if (!projectStore.projectPath) {
    router.replace({ name: 'project' })
    return
  }
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

async function handleInstall(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.installSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(t('global.installSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.installFailed', { error: e }))
  }
}

async function handleUpdate(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.updateSkill(name, projectStore.projectPath)
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

function handlePreview(name: string) {
  previewSkill.value = name
}

watch(() => configStore.config.active_agent_id, loadProjectSkills)

onMounted(async () => {
  await configStore.loadConfig()
  await loadProjectSkills()
})
</script>

<template>
  <div class="project-detail-view">
    <div class="sticky-header">
      <div class="page-header">
        <h1>
          <NButton text size="small" @click="router.push({ name: 'project' })">&larr; {{ t('common.back') }}</NButton>
          {{ t('project.detailTitle') }}
        </h1>
      </div>
      <p class="header-path">{{ projectStore.projectPath }}</p>
      <div class="stats-bar">
        <span>{{ t('stats.total', { count: stats.total }) }}</span>
        <span class="stat-item stat-same">{{ t('stats.same', { count: stats.same }) }}</span>
        <span class="stat-item stat-outdated">{{ t('stats.outdated', { count: stats.outdated }) }}</span>
        <span class="stat-item stat-local">{{ t('stats.localOnly', { count: stats.localOnly }) }}</span>
        <span class="stat-item stat-remote">{{ t('stats.remoteOnly', { count: stats.remoteOnly }) }}</span>
      </div>
      <div class="toolbar">
        <NButton :loading="skillStore.syncing" @click="handleSync">{{ t('common.syncRemote') }}</NButton>
        <NInput v-model:value="searchText" :placeholder="t('common.search')" clearable style="width: 160px" />
      </div>
    </div>

    <NSpin :show="skillStore.loading">
      <SkillCompareTable
        v-if="filtered.length > 0"
        :comparisons="filtered"
        :target="projectStore.projectPath"
        @install="handleInstall"
        @update="handleUpdate"
        @uninstall="handleUninstall"
        @preview="handlePreview"
      />
      <EmptyState v-else :title="t('project.emptySkillTitle')" :description="t('project.emptySkillDesc')" />
    </NSpin>

    <SkillPreviewModal
      v-if="previewSkill"
      :skill-name="previewSkill"
      :target="projectStore.projectPath"
      @close="previewSkill = null"
    />
  </div>
</template>

<style scoped>
.project-detail-view {
  max-width: 1100px;
}
.sticky-header {
  position: sticky;
  top: -24px;
  z-index: 10;
  background: #f5f7fa;
  padding: 0 0 16px;
  margin: 0 -24px;
  padding-left: 24px;
  padding-right: 24px;
  border-bottom: 1px solid #eee;
}
.page-header h1 {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: 2px;
}
.header-path {
  color: #999;
  font-size: 12px;
  font-family: monospace;
  margin-bottom: 8px;
}
.stats-bar {
  margin-bottom: 10px;
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
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
