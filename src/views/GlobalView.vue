<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { NButton, NInput, NSpace, NSelect, NSpin, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import SkillCompareTable from '@/components/common/SkillCompareTable.vue'
import SkillPreviewModal from '@/components/common/SkillPreviewModal.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import type { SkillComparison } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const message = useMessage()

const searchText = ref('')
const statusFilter = ref<string | null>(null)
const previewSkill = ref<string | null>(null)

const statusOptions = computed(() => [
  { label: t('status.all'), value: 'all' },
  { label: t('status.same'), value: 'Same' },
  { label: t('status.outdated'), value: 'Outdated' },
  { label: t('status.localOnly'), value: 'LocalOnly' },
  { label: t('status.remoteOnly'), value: 'RemoteOnly' },
])

const filtered = computed(() => {
  let list: SkillComparison[] = skillStore.globalComparisons
  if (searchText.value) {
    const q = searchText.value.toLowerCase()
    list = list.filter((s) => s.name.toLowerCase().includes(q))
  }
  if (statusFilter.value && statusFilter.value !== 'all') {
    list = list.filter((s) => s.status === statusFilter.value)
  }
  return list
})

const stats = computed(() => {
  const all = skillStore.globalComparisons
  return {
    total: all.length,
    same: all.filter((s) => s.status === 'Same').length,
    outdated: all.filter((s) => s.status === 'Outdated').length,
    localOnly: all.filter((s) => s.status === 'LocalOnly').length,
    remoteOnly: all.filter((s) => s.status === 'RemoteOnly').length,
  }
})

async function handleSync() {
  try {
    await skillStore.syncRemote()
    await skillStore.loadGlobalSkills()
    message.success(t('global.syncComplete'))
  } catch (e: any) {
    message.error(t('global.syncFailed', { error: e }))
  }
}

async function handleInstall(name: string) {
  try {
    await skillStore.installSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(t('global.installSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.installFailed', { error: e }))
  }
}

async function handleUpdate(name: string) {
  try {
    await skillStore.updateSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(t('global.updateSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.updateFailed', { error: e }))
  }
}

async function handleUninstall(name: string) {
  try {
    await skillStore.uninstallSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(t('global.uninstallSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.uninstallFailed', { error: e }))
  }
}

async function handleBatchUpdate() {
  const outdated = skillStore.globalComparisons
    .filter((s) => s.status === 'Outdated')
    .map((s) => s.name)
  if (outdated.length === 0) {
    message.info(t('global.noUpdatesNeeded'))
    return
  }
  try {
    const results = await skillStore.batchUpdate(outdated, 'global')
    await skillStore.loadGlobalSkills()
    message.success(t('global.batchUpdateComplete', { names: results.join(', ') }))
  } catch (e: any) {
    message.error(t('global.batchUpdateFailed', { error: e }))
  }
}

function handlePreview(name: string) {
  previewSkill.value = name
}

watch(() => configStore.config.active_agent_id, async () => {
  await skillStore.loadGlobalSkills()
})

onMounted(async () => {
  await configStore.loadConfig()
  await skillStore.loadGlobalSkills()
})
</script>

<template>
  <div class="global-view">
    <div class="sticky-header">
      <div class="page-header">
        <h1>{{ t('global.title') }}</h1>
        <p class="header-path">
          {{ t('global.globalPath', { agent: configStore.getActiveDisplayName(), path: configStore.getGlobalPath() || '-' }) }}
        </p>
      </div>
      <div class="stats-bar">
        <span>{{ t('stats.total', { count: stats.total }) }}</span>
        <span class="stat-item stat-same">{{ t('stats.same', { count: stats.same }) }}</span>
        <span class="stat-item stat-outdated">{{ t('stats.outdated', { count: stats.outdated }) }}</span>
        <span class="stat-item stat-local">{{ t('stats.localOnly', { count: stats.localOnly }) }}</span>
        <span class="stat-item stat-remote">{{ t('stats.remoteOnly', { count: stats.remoteOnly }) }}</span>
      </div>
      <div class="toolbar">
        <NSpace>
          <NButton type="primary" :loading="skillStore.syncing" @click="handleSync">
            {{ t('common.syncRemote') }}
          </NButton>
          <NButton :disabled="stats.outdated === 0" @click="handleBatchUpdate">
            {{ t('global.updateAll', { count: stats.outdated }) }}
          </NButton>
        </NSpace>
        <NSpace class="filters">
          <NInput v-model:value="searchText" :placeholder="t('common.search')" clearable style="width: 160px" />
          <NSelect v-model:value="statusFilter" :options="statusOptions" :placeholder="t('status.status')" clearable style="width: 110px" />
        </NSpace>
      </div>
    </div>

    <NSpin :show="skillStore.loading">
      <SkillCompareTable
        v-if="filtered.length > 0"
        :comparisons="filtered"
        target="global"
        @install="handleInstall"
        @update="handleUpdate"
        @uninstall="handleUninstall"
        @preview="handlePreview"
      />
      <EmptyState v-else :title="t('global.emptyTitle')" :description="t('global.emptyDesc')" />
    </NSpin>

    <SkillPreviewModal
      v-if="previewSkill"
      :skill-name="previewSkill"
      target="global"
      @close="previewSkill = null"
    />
  </div>
</template>

<style scoped>
.global-view {
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
.page-header {
  margin-bottom: 8px;
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
  flex-wrap: wrap;
  gap: 12px;
}
</style>
