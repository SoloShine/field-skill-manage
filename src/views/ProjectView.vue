<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { NButton, NInput, NSpace, NSpin, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import { useProjectStore } from '@/stores/project'
import { open } from '@tauri-apps/plugin-dialog'
import SkillCompareTable from '@/components/common/SkillCompareTable.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import type { SkillComparison } from '@/types'

const skillStore = useSkillStore()
const configStore = useConfigStore()
const projectStore = useProjectStore()
const message = useMessage()

const searchText = ref('')

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
      <h1>项目 Skill 管理</h1>
    </div>

    <div class="project-selector">
      <NSpace align="center">
        <NInput :value="projectStore.projectPath" placeholder="请选择项目路径..." readonly style="width: 400px" />
        <NButton type="primary" @click="handleSelectProject">浏览...</NButton>
        <NButton :disabled="!projectStore.projectPath" :loading="skillStore.syncing" @click="handleSync">
          同步远端
        </NButton>
      </NSpace>
    </div>

    <template v-if="projectStore.projectPath">
      <div class="stats-bar">
        <span>共 {{ stats.total }} 个</span>
        <span class="stat-item stat-same">一致: {{ stats.same }}</span>
        <span class="stat-item stat-outdated">可更新: {{ stats.outdated }}</span>
        <span class="stat-item stat-local">仅本地: {{ stats.localOnly }}</span>
        <span class="stat-item stat-remote">仅远端: {{ stats.remoteOnly }}</span>
      </div>

      <div class="toolbar">
        <NInput v-model:value="searchText" placeholder="搜索..." clearable style="width: 160px" />
      </div>

      <NSpin :show="skillStore.loading">
        <SkillCompareTable
          v-if="filtered.length > 0"
          :comparisons="filtered"
          :target="projectStore.projectPath"
          @install="handleInstall"
          @update="handleUpdate"
          @uninstall="handleUninstall"
        />
        <EmptyState v-else title="暂无 Skill" description="点击「同步远端」获取可用 skill 列表" />
      </NSpin>
    </template>

    <EmptyState v-else title="请选择项目路径" description="点击上方「浏览...」按钮选择一个项目目录" />
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
