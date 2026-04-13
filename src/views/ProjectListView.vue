<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  NCard,
  NButton,
  NSpace,
  NTag,
  NText,
  NEmpty,
  NSpin,
  NPopconfirm,
} from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import { useProjectStore } from '@/stores/project'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const projectStore = useProjectStore()

async function handleAddProject() {
  try {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      projectStore.setProjectPath(selected)
      await refresh()
    }
  } catch {
    // cancelled
  }
}

function handleOpenProject(path: string) {
  projectStore.projectPath = path
  router.push({ name: 'project-detail' })
}

async function handleRemoveProject(path: string) {
  projectStore.removeProjectPath(path)
  await refresh()
}

async function refresh() {
  if (projectStore.projectPaths.length > 0) {
    await skillStore.loadProjectsOverview(projectStore.projectPaths)
  } else {
    skillStore.projectsOverview = []
  }
}

// Refresh when agent changes
watch(() => configStore.config.active_agent_id, refresh)

onMounted(async () => {
  projectStore.loadPersisted()
  await refresh()
})
</script>

<template>
  <div class="project-list-view">
    <div class="sticky-header">
      <div class="page-header">
        <h1>项目管理</h1>
      </div>
      <div class="toolbar">
        <NSpace>
          <NButton type="primary" @click="handleAddProject">添加项目</NButton>
          <NButton :loading="skillStore.syncing" @click="skillStore.syncRemote().then(() => refresh())">
            同步远端
          </NButton>
        </NSpace>
      </div>
    </div>

    <NSpin :show="skillStore.loading">
      <div v-if="skillStore.projectsOverview.length > 0" class="project-cards">
        <NCard
          v-for="proj in skillStore.projectsOverview"
          :key="proj.project_path"
          hoverable
          class="project-card"
          @click="handleOpenProject(proj.project_path)"
        >
          <div class="card-header">
            <NText strong class="project-name">{{ proj.project_name }}</NText>
            <NSpace size="small" align="center" @click.stop>
              <NTag size="small" round>{{ proj.local_count }} 本地</NTag>
              <NTag v-if="proj.matched_count > 0" size="small" type="success" round>
                {{ proj.matched_count }} 匹配
              </NTag>
              <NTag v-if="proj.outdated_count > 0" size="small" type="warning" round>
                {{ proj.outdated_count }} 可更新
              </NTag>
              <NTag v-if="proj.remote_only_count > 0" size="small" type="default" round>
                {{ proj.remote_only_count }} 可安装
              </NTag>
            </NSpace>
          </div>
          <div class="card-path">
            <NText depth="3" style="font-size: 12px; font-family: monospace">
              {{ proj.project_path }}
            </NText>
          </div>
          <template #action>
            <NSpace justify="end">
              <NPopconfirm @positive-click="handleRemoveProject(proj.project_path)">
                <template #trigger>
                  <NButton size="tiny" type="error" ghost @click.stop>移除</NButton>
                </template>
                确认从列表中移除该项目？
              </NPopconfirm>
              <NButton size="tiny" type="primary" @click.stop="handleOpenProject(proj.project_path)">
                查看详情
              </NButton>
            </NSpace>
          </template>
        </NCard>
      </div>

      <NEmpty v-else description="暂无项目，点击「添加项目」开始管理" style="margin-top: 40px" />
    </NSpin>
  </div>
</template>

<style scoped>
.project-list-view {
  max-width: 900px;
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
  margin-bottom: 10px;
}
.page-header h1 {
  font-size: 22px;
  font-weight: 600;
}
.toolbar {
  display: flex;
  align-items: center;
}
.project-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.project-card {
  cursor: pointer;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.project-name {
  font-size: 16px;
}
.card-path {
  margin-top: 6px;
}
</style>
