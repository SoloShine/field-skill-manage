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
    message.error('加载失败: ' + e)
  }
}

async function handleSync() {
  try {
    await skillStore.syncRemote()
    await loadProjectSkills()
    message.success('远端同步完成')
  } catch (e: any) {
    message.error('同步失败: ' + e)
  }
}

async function handleInstall(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.installSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(`${name} 安装成功`)
  } catch (e: any) {
    message.error('安装失败: ' + e)
  }
}

async function handleUpdate(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.updateSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(`${name} 更新成功`)
  } catch (e: any) {
    message.error('更新失败: ' + e)
  }
}

async function handleUninstall(name: string) {
  if (!projectStore.projectPath) return
  try {
    await skillStore.uninstallSkill(name, projectStore.projectPath)
    await loadProjectSkills()
    message.success(`${name} 已卸载`)
  } catch (e: any) {
    message.error('卸载失败: ' + e)
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
          <NButton text size="small" @click="router.push({ name: 'project' })">&larr; 返回</NButton>
          项目详情
        </h1>
      </div>
      <p class="header-path">{{ projectStore.projectPath }}</p>
      <div class="stats-bar">
        <span>共 {{ stats.total }} 个</span>
        <span class="stat-item stat-same">一致: {{ stats.same }}</span>
        <span class="stat-item stat-outdated">可更新: {{ stats.outdated }}</span>
        <span class="stat-item stat-local">仅本地: {{ stats.localOnly }}</span>
        <span class="stat-item stat-remote">仅远端: {{ stats.remoteOnly }}</span>
      </div>
      <div class="toolbar">
        <NButton :loading="skillStore.syncing" @click="handleSync">同步远端</NButton>
        <NInput v-model:value="searchText" placeholder="搜索..." clearable style="width: 160px" />
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
      <EmptyState v-else title="暂无 Skill" description="点击「同步远端」获取可用 skill 列表" />
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
