<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, watch } from 'vue'
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
const tableHeight = ref(400)
const viewRef = ref<HTMLElement | null>(null)
const headerRef = ref<HTMLElement | null>(null)
let resizeObserver: ResizeObserver | null = null

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
    const result = skillStore.lastSyncResult
    if (result && result.fail_count > 0) {
      message.warning(t('global.syncPartial', { ok: result.success_count, fail: result.fail_count, errors: result.errors.join('; ') }))
    } else {
      message.success(t('global.syncComplete'))
    }
  } catch (e: any) {
    message.error(t('global.syncFailed', { error: e }))
  }
}

async function handleInstall(name: string, _target: string, repoId?: string) {
  try {
    await skillStore.installSkill(name, 'global', repoId)
    await skillStore.loadGlobalSkills()
    message.success(t('global.installSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.installFailed', { error: e }))
  }
}

async function handleUpdate(name: string, _target: string, repoId?: string) {
  try {
    await skillStore.updateSkill(name, 'global', repoId)
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

function handlePreview(name: string, _repoId?: string) {
  previewSkill.value = name
}

function updateTableHeight() {
  if (viewRef.value && headerRef.value) {
    const viewH = viewRef.value.clientHeight
    const headerH = headerRef.value.offsetHeight
    tableHeight.value = Math.max(200, viewH - headerH - 35)
  }
}

watch(() => configStore.config.active_agent_id, async () => {
  await skillStore.loadGlobalSkills()
})

onMounted(async () => {
  await configStore.loadConfig()
  await skillStore.loadGlobalSkills()

  resizeObserver = new ResizeObserver(updateTableHeight)
  if (viewRef.value) resizeObserver.observe(viewRef.value)
  if (headerRef.value) resizeObserver.observe(headerRef.value)
  updateTableHeight()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})
</script>

<template>
  <div class="global-view" ref="viewRef">
    <div class="view-header" ref="headerRef">
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
        :max-height="tableHeight"
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
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.view-header {
  flex-shrink: 0;
  padding: 0 0 16px;
  border-bottom: 1px solid var(--color-border);
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
  color: var(--color-text-muted);
  font-size: 12px;
  font-family: var(--font-mono);
}
.stats-bar {
  margin-bottom: 10px;
  font-size: 13px;
  color: var(--color-text-secondary);
  display: flex;
  gap: 16px;
}
.stat-item { font-weight: 500; }
.stat-same { color: var(--color-status-same); }
.stat-outdated { color: var(--color-status-outdated); }
.stat-local { color: var(--color-status-local); }
.stat-remote { color: var(--color-status-remote); }
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
</style>
