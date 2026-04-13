<script setup lang="ts">
import { onMounted, reactive, ref, computed } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NButton,
  NSwitch,
  NSpace,
  NDivider,
  NPopconfirm,
  NTag,
  NText,
  NCollapse,
  NCollapseItem,
  useMessage,
} from 'naive-ui'
import { useConfigStore } from '@/stores/config'
import { useUpdateStore } from '@/stores/update'
import type { AppConfig } from '@/types'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'

const { t } = useI18n()
const configStore = useConfigStore()
const updateStore = useUpdateStore()
const message = useMessage()

const form = reactive<AppConfig>({
  remote_url: '',
  cache_path: '',
  auto_sync: false,
  active_agent_id: 'claude',
  agent_global_paths: {},
  agent_project_patterns: {},
  agent_display_names: {},
  custom_agent_ids: [],
})

// New custom agent form
const newId = ref('')
const newName = ref('')
const newGlobalPath = ref('')
const newProjectPattern = ref('')

// Built-in agent ids for display
const builtInAgents = [
  { id: 'claude', label: 'Claude Code' },
  { id: 'opencode', label: 'OpenCode' },
  { id: 'codex', label: 'Codex (OpenAI)' },
  { id: 'cursor', label: 'Cursor' },
  { id: 'windsurf', label: 'Windsurf / Cline' },
]

async function handleSave() {
  try {
    await configStore.saveConfig({ ...form })
    message.success(t('settings.saveSuccess'))
  } catch (e: any) {
    message.error(t('settings.saveFailed', { error: e }))
  }
}

async function handleReset() {
  try {
    await configStore.loadConfig()
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    message.info(t('settings.resetSuccess'))
  } catch (e: any) {
    message.error(t('settings.resetFailed', { error: e }))
  }
}

async function handleAddCustom() {
  if (!newId.value.trim() || !newName.value.trim()) {
    message.warning(t('settings.idNameRequired'))
    return
  }
  try {
    await configStore.addCustomAgent(
      newId.value.trim(),
      newName.value.trim(),
      newGlobalPath.value.trim() || `~/.${newId.value.trim()}/skills`,
      newProjectPattern.value.trim() || `{project}/.${newId.value.trim()}/skills`,
    )
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    newId.value = ''
    newName.value = ''
    newGlobalPath.value = ''
    newProjectPattern.value = ''
    message.success(t('settings.addSuccess'))
  } catch (e: any) {
    message.error(t('settings.addFailed', { error: e }))
  }
}

async function handleRemoveCustom(id: string) {
  try {
    await configStore.removeCustomAgent(id)
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    message.success(t('settings.removeSuccess'))
  } catch (e: any) {
    message.error(t('settings.removeFailed', { error: e }))
  }
}

async function handleCheckUpdate() {
  await updateStore.checkForUpdates()
  if (updateStore.updateInfo?.error) {
    if (updateStore.updateInfo.error === 'NO_RELEASES') {
      message.warning(t('update.noReleases'))
    } else {
      message.error(t('update.checkFailed', { error: updateStore.updateInfo.error }))
    }
  } else if (updateStore.updateInfo?.has_update) {
    message.info(t('update.newVersionAvailable') + ': v' + updateStore.updateInfo.latest_version)
  } else {
    message.success(t('update.alreadyLatest'))
  }
}

async function openReleasePage() {
  if (updateStore.updateInfo?.release_url) {
    await open(updateStore.updateInfo.release_url)
  }
}

const renderedNotes = computed(() => {
  return updateStore.updateInfo?.release_notes || ''
})

onMounted(async () => {
  await configStore.loadConfig()
  Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
  updateStore.loadCurrentVersion()
})
</script>

<template>
  <div class="settings-view">
    <div class="page-header">
      <h1>{{ t('settings.title') }}</h1>
    </div>

    <NCard :title="t('settings.about')" class="settings-card">
      <NForm label-placement="left" label-width="140">
        <NFormItem :label="t('settings.currentVersion')">
          <NSpace align="center">
            <NText>v{{ updateStore.currentVersion }}</NText>
            <NButton
              size="small"
              :loading="updateStore.checking"
              @click="handleCheckUpdate"
            >
              {{ t('settings.checkUpdate') }}
            </NButton>
          </NSpace>
        </NFormItem>
        <NFormItem v-if="updateStore.updateInfo && !updateStore.updateInfo.error" :label="t('settings.latestVersion')">
          <NSpace vertical>
            <NSpace align="center">
              <NText>v{{ updateStore.updateInfo.latest_version }}</NText>
              <NTag v-if="updateStore.updateInfo.has_update" type="warning" size="small" round>
                {{ t('update.newVersionAvailable') }}
              </NTag>
              <NTag v-else type="success" size="small" round>
                {{ t('update.alreadyLatest') }}
              </NTag>
            </NSpace>
            <NButton
              v-if="updateStore.updateInfo.has_update"
              type="primary"
              size="small"
              @click="openReleasePage"
            >
              {{ t('update.goToDownload') }}
            </NButton>
            <NCollapse v-if="updateStore.updateInfo.release_notes">
              <NCollapseItem :title="t('update.releaseNotes')" name="notes">
                <div class="release-notes" v-html="renderedNotes" />
              </NCollapseItem>
            </NCollapse>
          </NSpace>
        </NFormItem>
      </NForm>
    </NCard>

    <NCard :title="t('settings.remoteRepo')" class="settings-card">
      <NForm label-placement="left" label-width="140">
        <NFormItem :label="t('settings.repoUrl')">
          <NInput v-model:value="form.remote_url" :placeholder="t('settings.repoUrlPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.cacheDir')">
          <NInput v-model:value="form.cache_path" :placeholder="t('settings.cacheDirPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.autoSync')">
          <NSwitch v-model:value="form.auto_sync" />
        </NFormItem>
      </NForm>
    </NCard>

    <NCard :title="t('settings.builtinAgents')" class="settings-card">
      <template #header-extra>
        <NText depth="3" style="font-size: 12px">{{ t('settings.globalPathProjectPattern') }}</NText>
      </template>
      <NForm label-placement="left" label-width="140">
        <NFormItem v-for="agent in builtInAgents" :key="agent.id" :label="agent.label">
          <NSpace vertical style="width: 100%">
            <NInput
              v-model:value="form.agent_global_paths[agent.id]"
              :placeholder="t('settings.globalSkillPath')"
              size="small"
            />
            <NInput
              v-model:value="form.agent_project_patterns[agent.id]"
              :placeholder="t('settings.projectPatternExample')"
              size="small"
            />
          </NSpace>
        </NFormItem>
      </NForm>
    </NCard>

    <NCard :title="t('settings.customAgents')" class="settings-card">
      <!-- Existing custom agents -->
      <div v-if="form.custom_agent_ids.length > 0" class="custom-agent-list">
        <div v-for="cid in form.custom_agent_ids" :key="cid" class="custom-agent-item">
          <div class="custom-agent-header">
            <NText strong>{{ form.agent_display_names[cid] || cid }}</NText>
            <NTag size="small" type="info" round>{{ t('settings.custom') }}</NTag>
            <NPopconfirm @positive-click="handleRemoveCustom(cid)">
              <template #trigger>
                <NButton size="tiny" type="error" ghost>{{ t('common.remove') }}</NButton>
              </template>
              {{ t('settings.confirmRemoveAgent') }}
            </NPopconfirm>
          </div>
          <NForm label-placement="left" label-width="100" size="small">
            <NFormItem :label="t('settings.agentGlobalPath')">
              <NInput v-model:value="form.agent_global_paths[cid]" />
            </NFormItem>
            <NFormItem :label="t('settings.agentProjectPattern')">
              <NInput v-model:value="form.agent_project_patterns[cid]" :placeholder="t('settings.projectPatternPlaceholder')" />
            </NFormItem>
          </NForm>
        </div>
      </div>
      <NText v-else depth="3" style="font-size: 13px">{{ t('settings.noCustomAgents') }}</NText>

      <NDivider style="margin: 16px 0 12px">{{ t('settings.addCustomAgent') }}</NDivider>
      <NForm label-placement="left" label-width="100" size="small">
        <NFormItem :label="t('settings.agentId')">
          <NInput v-model:value="newId" :placeholder="t('settings.agentIdPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.agentName')">
          <NInput v-model:value="newName" :placeholder="t('settings.agentNamePlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.agentGlobalPath')">
          <NInput v-model:value="newGlobalPath" :placeholder="t('settings.agentGlobalPathPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.agentProjectPattern')">
          <NInput v-model:value="newProjectPattern" :placeholder="t('settings.agentProjectPatternPlaceholder')" />
        </NFormItem>
        <NFormItem>
          <NButton type="primary" @click="handleAddCustom">{{ t('common.add') }}</NButton>
        </NFormItem>
      </NForm>
    </NCard>

    <NSpace class="actions">
      <NButton type="primary" @click="handleSave">{{ t('common.save') }}</NButton>
      <NButton @click="handleReset">{{ t('common.reset') }}</NButton>
    </NSpace>
  </div>
</template>

<style scoped>
.settings-view { max-width: 750px; }
.page-header {
  position: sticky; top: -24px; z-index: 10;
  background: #f5f7fa; padding: 0 0 12px;
  margin: 0 -24px; padding-left: 24px; padding-right: 24px;
}
.page-header h1 { font-size: 22px; font-weight: 600; }
.settings-card { margin-bottom: 16px; }
.custom-agent-list { display: flex; flex-direction: column; gap: 12px; }
.custom-agent-item {
  padding: 12px; border: 1px solid #eee; border-radius: 6px;
  background: #fafafa;
}
.custom-agent-header {
  display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
}
.actions { margin-top: 16px; }
.release-notes {
  font-size: 13px;
  line-height: 1.6;
  max-height: 300px;
  overflow-y: auto;
  word-break: break-word;
}
.release-notes :deep(pre) {
  background: #f5f5f5;
  padding: 8px;
  border-radius: 4px;
  overflow-x: auto;
}
.release-notes :deep(code) {
  background: #f0f0f0;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 12px;
}
</style>
