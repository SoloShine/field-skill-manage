<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, watch, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NInput, NSpin, NSkeleton, NBreadcrumb, NBreadcrumbItem, NText, useMessage } from 'naive-ui'
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
const statusFilter = ref<string | null>(null)
const previewSkill = ref<string | null>(null)
const searchInputRef = ref<InstanceType<typeof NInput> | null>(null)
const tableHeight = ref(400)
const viewRef = ref<HTMLElement | null>(null)
const headerRef = ref<HTMLElement | null>(null)
let resizeObserver: ResizeObserver | null = null

const operatingKeys = reactive(new Set<string>())
const firstLoaded = ref(false)

function handleSearchShortcut(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    const el = searchInputRef.value?.$el as HTMLElement | undefined
    el?.querySelector('input')?.focus()
  }
}

const filtered = computed(() => {
  let list: SkillComparison[] = skillStore.projectComparisons
  if (searchText.value) {
    const q = searchText.value.toLowerCase()
    list = list.filter((s) => s.name.toLowerCase().includes(q))
  }
  if (statusFilter.value && statusFilter.value !== 'all') {
    list = list.filter((s) => s.status === statusFilter.value)
  }
  return list
})

const projectName = computed(() => {
  if (!projectStore.projectPath) return ''
  const parts = projectStore.projectPath.replace(/\\/g, '/')
  return parts.split('/').filter(Boolean).pop() || projectStore.projectPath
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
  operatingKeys.add(name)
  try {
    await skillStore.installSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(t('global.installSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.installFailed', { error: e }))
  } finally {
    operatingKeys.delete(name)
  }
}

async function handleUpdate(name: string) {
  if (!projectStore.projectPath) return
  operatingKeys.add(name)
  try {
    await skillStore.updateSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(t('global.updateSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.updateFailed', { error: e }))
  } finally {
    operatingKeys.delete(name)
  }
}

async function handleUninstall(name: string) {
  if (!projectStore.projectPath) return
  operatingKeys.add(name)
  try {
    await skillStore.uninstallSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(t('global.uninstallSuccess', { name }))
  } catch (e: any) {
    message.error(t('global.uninstallFailed', { error: e }))
  } finally {
    operatingKeys.delete(name)
  }
}

function handlePreview(name: string) {
  previewSkill.value = name
}

async function handleBatchInstall(names: string[]) {
  for (const name of names) {
    try { await skillStore.installSkill(name, projectStore.projectPath!) } catch { /* skip */ }
  }
  await loadProjectSkills()
  message.success(t('global.syncComplete'))
}

async function handleBatchUninstall(names: string[]) {
  for (const name of names) {
    try { await skillStore.uninstallSkill(name, projectStore.projectPath!) } catch { /* skip */ }
  }
  await loadProjectSkills()
  message.success(t('global.syncComplete'))
}

function updateTableHeight() {
  if (viewRef.value && headerRef.value) {
    const viewH = viewRef.value.clientHeight
    const headerH = headerRef.value.offsetHeight
    tableHeight.value = Math.max(200, viewH - headerH - 35)
  }
}

watch(() => configStore.config.active_agent_id, loadProjectSkills)

onMounted(async () => {
  await configStore.loadConfig()
  await loadProjectSkills()
  firstLoaded.value = true

  resizeObserver = new ResizeObserver(updateTableHeight)
  if (viewRef.value) resizeObserver.observe(viewRef.value)
  if (headerRef.value) resizeObserver.observe(headerRef.value)
  updateTableHeight()

  document.addEventListener('keydown', handleSearchShortcut)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  document.removeEventListener('keydown', handleSearchShortcut)
})
</script>

<template>
  <div class="project-detail-view" ref="viewRef">
    <div class="view-header" ref="headerRef">
      <div class="page-header">
        <NBreadcrumb>
          <NBreadcrumbItem @click="router.push({ name: 'project' })">
            {{ t('project.breadcrumbProject') }}
          </NBreadcrumbItem>
          <NBreadcrumbItem>
            <NText strong>{{ projectName }}</NText>
          </NBreadcrumbItem>
        </NBreadcrumb>
      </div>
      <p class="header-path">{{ projectStore.projectPath }}</p>
      <div class="stats-bar">
        <button class="stat-chip" :class="{ active: !statusFilter || statusFilter === 'all' }" @click="statusFilter = !statusFilter || statusFilter === 'all' ? null : 'all'">{{ t('stats.total', { count: stats.total }) }}</button>
        <button class="stat-chip stat-same" :class="{ active: statusFilter === 'Same' }" @click="statusFilter = statusFilter === 'Same' ? null : 'Same'">{{ t('stats.same', { count: stats.same }) }}</button>
        <button class="stat-chip stat-outdated" :class="{ active: statusFilter === 'Outdated' }" @click="statusFilter = statusFilter === 'Outdated' ? null : 'Outdated'">{{ t('stats.outdated', { count: stats.outdated }) }}</button>
        <button class="stat-chip stat-local" :class="{ active: statusFilter === 'LocalOnly' }" @click="statusFilter = statusFilter === 'LocalOnly' ? null : 'LocalOnly'">{{ t('stats.localOnly', { count: stats.localOnly }) }}</button>
        <button class="stat-chip stat-remote" :class="{ active: statusFilter === 'RemoteOnly' }" @click="statusFilter = statusFilter === 'RemoteOnly' ? null : 'RemoteOnly'">{{ t('stats.remoteOnly', { count: stats.remoteOnly }) }}</button>
      </div>
      <div class="toolbar">
        <NButton :loading="skillStore.syncing" @click="handleSync">{{ t('common.syncRemote') }}</NButton>
        <NInput ref="searchInputRef" v-model:value="searchText" :placeholder="t('common.search')" clearable style="width: 160px" />
      </div>
    </div>

    <div v-if="!firstLoaded && skillStore.loading" class="skeleton-wrapper">
      <NSkeleton text :repeat="2" />
      <NSkeleton text style="width: 60%" />
    </div>
    <NSpin v-else :show="skillStore.loading">
      <SkillCompareTable
        v-if="filtered.length > 0"
        :comparisons="filtered"
        :target="projectStore.projectPath"
        :max-height="tableHeight"
        :operating-keys="operatingKeys"
        @install="handleInstall"
        @update="handleUpdate"
        @uninstall="handleUninstall"
        @preview="handlePreview"
        @batch-install="handleBatchInstall"
        @batch-update="(names: string[]) => { skillStore.batchUpdate(names, projectStore.projectPath!).then(() => loadProjectSkills()) }"
        @batch-uninstall="handleBatchUninstall"
      />
      <EmptyState v-else-if="skillStore.projectComparisons.length === 0" :title="t('project.emptySkillTitle')" :description="t('project.emptySkillDesc')" :action-label="t('project.emptySkillAction')" @action="handleSync" />
      <EmptyState v-else :title="t('global.noMatchTitle')" :description="t('global.noMatchDesc')" />
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
  margin-bottom: 2px;
}
.header-path {
  color: var(--color-text-muted);
  font-size: 12px;
  font-family: var(--font-mono);
  margin-bottom: 8px;
}
.stats-bar {
  margin-bottom: 10px;
  font-size: 13px;
  color: var(--color-text-secondary);
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.stat-chip {
  font-weight: 500;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: none;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
  font-family: inherit;
  line-height: 1.5;
}
.stat-chip:hover {
  opacity: 0.85;
}
.stat-chip.active {
  background: var(--color-bg-tertiary);
  border-color: currentColor;
}
.stat-same { color: var(--color-status-same); }
.stat-outdated { color: var(--color-status-outdated); }
.stat-local { color: var(--color-status-local); }
.stat-remote { color: var(--color-status-remote); }
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.skeleton-wrapper {
  padding: 20px 0;
}
</style>
