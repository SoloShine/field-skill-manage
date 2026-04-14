<script setup lang="ts">
import { onMounted, reactive, ref, computed, watchEffect } from 'vue'
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
  NRadioGroup,
  NRadioButton,
  useMessage,
} from 'naive-ui'
import { useConfigStore } from '@/stores/config'
import { useUpdateStore } from '@/stores/update'
import { useTheme, PRESET_COLORS } from '@/composables/useTheme'
import type { AppConfig, RepoConfig } from '@/types'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'
import { save, open as openDialog } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const configStore = useConfigStore()
const updateStore = useUpdateStore()
const message = useMessage()
const { isDark, toggleTheme, accentColor, setAccentColor, setCustomAccentColor, isPresetSelected, isValidHex, isPreset } = useTheme()

const customColorInput = ref('')
const themeMode = computed({
  get: () => isDark.value ? 'dark' : 'light',
  set: (val: string) => { if ((val === 'dark') !== isDark.value) toggleTheme() },
})

// Sync input with current accent color when it changes from presets
watchEffect(() => {
  if (isPreset(accentColor.value)) {
    customColorInput.value = ''
  }
})

function handleCustomColor() {
  const hex = customColorInput.value.trim()
  if (!hex) return
  if (!isValidHex(hex)) {
    message.warning(t('settings.invalidColor'))
    return
  }
  setCustomAccentColor(hex)
}

const form = reactive<AppConfig>({
  remote_url: '',
  cache_path: '',
  auto_sync: false,
  active_agent_id: 'claude',
  agent_global_paths: {},
  agent_project_patterns: {},
  agent_display_names: {},
  custom_agent_ids: [],
  repos: [],
})

// New repo form
const newRepoName = ref('')
const newRepoUrl = ref('')
const newRepoCachePath = ref('')

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

function handleAddRepo() {
  if (!newRepoUrl.value.trim()) {
    message.warning(t('settings.repoUrlRequired'))
    return
  }
  const id = 'repo_' + Date.now()
  // Derive home from claude's global path (always present, e.g. "C:/Users/xxx/.claude/skills")
  const claudePath = form.agent_global_paths['claude'] || ''
  const homeDir = claudePath.replace(/[\\/]\.claude[\\/].*$/, '')
  const defaultCache = homeDir ? `${homeDir}/.spm/cache_${id}` : ''
  const repo: RepoConfig = {
    id,
    name: newRepoName.value.trim() || t('settings.defaultRepoName'),
    url: newRepoUrl.value.trim(),
    cache_path: newRepoCachePath.value.trim() || defaultCache,
    enabled: true,
  }
  form.repos.push(repo)
  newRepoName.value = ''
  newRepoUrl.value = ''
  newRepoCachePath.value = ''
}

function handleRemoveRepo(index: number) {
  form.repos.splice(index, 1)
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

async function handleExport() {
  try {
    const filePath = await save({
      defaultPath: 'spm-config.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!filePath) return
    await invoke('export_config', { filePath })
    message.success(t('settings.exportSuccess'))
  } catch (e: any) {
    message.error(t('settings.exportFailed', { error: e }))
  }
}

async function handleImport() {
  try {
    const filePath = await openDialog({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false,
    })
    if (!filePath || typeof filePath !== 'string') return
    await invoke('import_config', { filePath })
    await configStore.loadConfig()
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    message.success(t('settings.importSuccess'))
  } catch (e: any) {
    message.error(t('settings.importFailed', { error: e }))
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
  // Version is loaded at app startup (AppLayout), only reload if missing
  if (!updateStore.currentVersion) {
    await updateStore.loadCurrentVersion()
  }
})
</script>

<template>
  <div class="settings-view">
    <div class="page-header">
      <h1>{{ t('settings.title') }}</h1>
    </div>
    <div class="settings-scroll">

    <NCard :title="t('settings.about')" class="settings-card">
      <div class="about-intro">
        <NText class="about-desc">{{ t('settings.projectDesc') }}</NText>
        <NSpace align="center" class="about-repo">
          <NText depth="3" class="about-repo-label">GitHub:</NText>
          <NButton text type="primary" @click="open('https://github.com/SoloShine/field-skill-manage')">
            SoloShine/field-skill-manage
          </NButton>
        </NSpace>
      </div>
      <NDivider style="margin: 12px 0" />
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
          <NSpace align="center">
            <NText>v{{ updateStore.updateInfo.latest_version }}</NText>
            <NTag v-if="updateStore.updateInfo.has_update" type="warning" size="small" round>
              {{ t('update.newVersionAvailable') }}
            </NTag>
            <NTag v-else type="success" size="small" round>
              {{ t('update.alreadyLatest') }}
            </NTag>
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

    <NCard :title="t('settings.appearance')" class="settings-card">
      <NForm label-placement="left" label-width="140">
        <NFormItem :label="t('settings.themeMode')">
          <NRadioGroup v-model:value="themeMode" size="small">
            <NRadioButton value="light">{{ t('settings.lightMode') }}</NRadioButton>
            <NRadioButton value="dark">{{ t('settings.darkMode') }}</NRadioButton>
          </NRadioGroup>
        </NFormItem>
        <NFormItem :label="t('settings.accentColor')">
          <div class="accent-picker">
            <div class="color-swatches">
              <button
                v-for="preset in PRESET_COLORS"
                :key="preset.primary"
                class="color-swatch"
                :class="{ active: isPresetSelected(preset) }"
                :style="{ backgroundColor: preset.primary }"
                :title="preset.name"
                @click="setAccentColor(preset)"
              />
            </div>
            <NSpace align="center" style="margin-top: 8px">
              <NInput
                v-model:value="customColorInput"
                :placeholder="t('settings.customColorPlaceholder')"
                size="small"
                style="width: 120px"
                @keyup.enter="handleCustomColor"
              />
              <NButton size="small" @click="handleCustomColor">{{ t('settings.customColor') }}</NButton>
            </NSpace>
          </div>
        </NFormItem>
      </NForm>
    </NCard>

    <NCard :title="t('settings.remoteRepo')" class="settings-card">
      <!-- Existing repos -->
      <div v-if="form.repos.length > 0" class="repo-list">
        <div v-for="(repo, index) in form.repos" :key="repo.id" class="repo-item">
          <div class="repo-header">
            <NText strong>{{ repo.name }}</NText>
            <NTag size="small" :type="repo.enabled ? 'success' : 'default'" round>
              {{ repo.enabled ? t('settings.repoEnabled') : t('settings.repoDisabled') }}
            </NTag>
            <NPopconfirm @positive-click="handleRemoveRepo(index)">
              <template #trigger>
                <NButton size="tiny" type="error" ghost>{{ t('common.remove') }}</NButton>
              </template>
              {{ t('settings.confirmRemoveRepo') }}
            </NPopconfirm>
          </div>
          <NForm label-placement="left" label-width="100" size="small">
            <NFormItem :label="t('settings.repoName')">
              <NInput v-model:value="repo.name" />
            </NFormItem>
            <NFormItem :label="t('settings.repoUrl')">
              <NInput v-model:value="repo.url" />
            </NFormItem>
            <NFormItem :label="t('settings.cacheDir')">
              <NInput v-model:value="repo.cache_path" />
            </NFormItem>
            <NFormItem :label="t('settings.repoEnabled')">
              <NSwitch v-model:value="repo.enabled" />
            </NFormItem>
          </NForm>
        </div>
      </div>
      <NText v-else depth="3" style="font-size: 13px">{{ t('settings.noRepos') }}</NText>

      <NDivider style="margin: 16px 0 12px">{{ t('settings.addRepo') }}</NDivider>
      <NForm label-placement="left" label-width="100" size="small">
        <NFormItem :label="t('settings.repoName')">
          <NInput v-model:value="newRepoName" :placeholder="t('settings.repoNamePlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.repoUrl')">
          <NInput v-model:value="newRepoUrl" :placeholder="t('settings.repoUrlPlaceholder')" />
        </NFormItem>
        <NFormItem :label="t('settings.cacheDir')">
          <NInput v-model:value="newRepoCachePath" :placeholder="t('settings.cacheDirPlaceholder')" />
        </NFormItem>
        <NFormItem>
          <NButton type="primary" @click="handleAddRepo">{{ t('common.add') }}</NButton>
        </NFormItem>
      </NForm>

      <NDivider />
      <NFormItem :label="t('settings.autoSync')" label-placement="left">
        <NSwitch v-model:value="form.auto_sync" />
      </NFormItem>
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
    </div>

    <div class="settings-footer">
      <NSpace>
        <NButton type="primary" @click="handleSave">{{ t('common.save') }}</NButton>
        <NButton @click="handleReset">{{ t('common.reset') }}</NButton>
        <NButton @click="handleExport">{{ t('settings.exportConfig') }}</NButton>
        <NButton @click="handleImport">{{ t('settings.importConfig') }}</NButton>
      </NSpace>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.page-header {
  flex-shrink: 0;
  padding: 0 0 12px;
  border-bottom: 1px solid var(--color-border);
}
.page-header h1 { font-size: 22px; font-weight: 600; }
.settings-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
}
.settings-footer {
  flex-shrink: 0;
  padding: 12px 0 0;
  border-top: 1px solid var(--color-border);
  margin-top: 8px;
}
.settings-card { margin-bottom: 16px; }
.custom-agent-list { display: flex; flex-direction: column; gap: 12px; }
.custom-agent-item {
  padding: 12px; border: 1px solid var(--color-border); border-radius: var(--radius-sm);
  background: var(--color-bg-tertiary);
}
.custom-agent-header {
  display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
}
.repo-list { display: flex; flex-direction: column; gap: 12px; }
.repo-item {
  padding: 12px; border: 1px solid var(--color-border); border-radius: var(--radius-sm);
  background: var(--color-bg-tertiary);
}
.repo-header {
  display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
}
.about-intro { margin-bottom: 4px; }
.about-desc { font-size: 13px; line-height: 1.6; display: block; margin-bottom: 8px; }
.about-repo { margin-top: 4px; }
.about-repo-label { font-size: 13px; }
.release-notes {
  font-size: 13px;
  line-height: 1.6;
  max-height: 300px;
  overflow-y: auto;
  word-break: break-word;
}
.release-notes :deep(pre) {
  background: var(--color-bg-code);
  padding: 8px;
  border-radius: 4px;
  overflow-x: auto;
}
.release-notes :deep(code) {
  background: var(--color-bg-inline-code);
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 12px;
}
.accent-picker {
  display: flex;
  flex-direction: column;
}
.color-swatches {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.2s, transform 0.15s;
  outline: none;
}
.color-swatch:hover {
  transform: scale(1.15);
}
.color-swatch.active {
  border-color: var(--color-text-primary);
}
</style>
