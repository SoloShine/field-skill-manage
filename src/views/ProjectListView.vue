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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
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
        <h1>{{ t('project.title') }}</h1>
      </div>
      <div class="toolbar">
        <NSpace>
          <NButton type="primary" @click="handleAddProject">{{ t('project.addProject') }}</NButton>
          <NButton :loading="skillStore.syncing" @click="skillStore.syncRemote().then(() => refresh())">
            {{ t('common.syncRemote') }}
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
              <NTag size="small" round>{{ t('stats.local', { count: proj.local_count }) }}</NTag>
              <NTag v-if="proj.matched_count > 0" size="small" type="success" round>
                {{ t('stats.matched', { count: proj.matched_count }) }}
              </NTag>
              <NTag v-if="proj.outdated_count > 0" size="small" type="warning" round>
                {{ t('stats.updatable', { count: proj.outdated_count }) }}
              </NTag>
              <NTag v-if="proj.remote_only_count > 0" size="small" type="default" round>
                {{ t('stats.installable', { count: proj.remote_only_count }) }}
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
                  <NButton size="tiny" type="error" ghost @click.stop>{{ t('common.remove') }}</NButton>
                </template>
                {{ t('project.confirmRemove') }}
              </NPopconfirm>
              <NButton size="tiny" type="primary" @click.stop="handleOpenProject(proj.project_path)">
                {{ t('project.viewDetail') }}
              </NButton>
            </NSpace>
          </template>
        </NCard>
      </div>

      <NEmpty v-else :description="t('project.emptyDesc')" style="margin-top: 40px" />
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
