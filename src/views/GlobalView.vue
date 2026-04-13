<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { NButton, NInput, NSpace, NSelect, NSpin, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import SkillCompareTable from '@/components/common/SkillCompareTable.vue'
import SkillPreviewModal from '@/components/common/SkillPreviewModal.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import type { SkillComparison } from '@/types'

const skillStore = useSkillStore()
const configStore = useConfigStore()
const message = useMessage()

const searchText = ref('')
const statusFilter = ref<string | null>(null)
const previewSkill = ref<string | null>(null)

const statusOptions = [
  { label: '全部', value: 'all' },
  { label: '一致', value: 'Same' },
  { label: '可更新', value: 'Outdated' },
  { label: '仅本地', value: 'LocalOnly' },
  { label: '仅远端', value: 'RemoteOnly' },
]

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
    message.success('远端同步完成')
  } catch (e: any) {
    message.error('同步失败: ' + e)
  }
}

async function handleInstall(name: string) {
  try {
    await skillStore.installSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(`${name} 安装成功`)
  } catch (e: any) {
    message.error('安装失败: ' + e)
  }
}

async function handleUpdate(name: string) {
  try {
    await skillStore.updateSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(`${name} 更新成功`)
  } catch (e: any) {
    message.error('更新失败: ' + e)
  }
}

async function handleUninstall(name: string) {
  try {
    await skillStore.uninstallSkill(name, 'global')
    await skillStore.loadGlobalSkills()
    message.success(`${name} 已卸载`)
  } catch (e: any) {
    message.error('卸载失败: ' + e)
  }
}

async function handleBatchUpdate() {
  const outdated = skillStore.globalComparisons
    .filter((s) => s.status === 'Outdated')
    .map((s) => s.name)
  if (outdated.length === 0) {
    message.info('没有需要更新的 skill')
    return
  }
  try {
    const results = await skillStore.batchUpdate(outdated, 'global')
    await skillStore.loadGlobalSkills()
    message.success(`批量更新完成: ${results.join(', ')}`)
  } catch (e: any) {
    message.error('批量更新失败: ' + e)
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
        <h1>全局 Skill 管理</h1>
        <p class="header-path">
          {{ configStore.getActiveDisplayName() }} 全局路径: {{ configStore.getGlobalPath() || '-' }}
        </p>
      </div>
      <div class="stats-bar">
        <span>共 {{ stats.total }} 个</span>
        <span class="stat-item stat-same">一致: {{ stats.same }}</span>
        <span class="stat-item stat-outdated">可更新: {{ stats.outdated }}</span>
        <span class="stat-item stat-local">仅本地: {{ stats.localOnly }}</span>
        <span class="stat-item stat-remote">仅远端: {{ stats.remoteOnly }}</span>
      </div>
      <div class="toolbar">
        <NSpace>
          <NButton type="primary" :loading="skillStore.syncing" @click="handleSync">
            同步远端
          </NButton>
          <NButton :disabled="stats.outdated === 0" @click="handleBatchUpdate">
            全部更新 ({{ stats.outdated }})
          </NButton>
        </NSpace>
        <NSpace class="filters">
          <NInput v-model:value="searchText" placeholder="搜索..." clearable style="width: 160px" />
          <NSelect v-model:value="statusFilter" :options="statusOptions" placeholder="状态" clearable style="width: 110px" />
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
      <EmptyState v-else title="暂无 Skill" description="点击「同步远端」从远程仓库拉取 skill 列表" />
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
