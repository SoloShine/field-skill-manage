<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
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
  useMessage,
} from 'naive-ui'
import { useConfigStore } from '@/stores/config'
import type { AppConfig } from '@/types'

const configStore = useConfigStore()
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
    message.success('配置已保存')
  } catch (e: any) {
    message.error('保存失败: ' + e)
  }
}

async function handleReset() {
  try {
    await configStore.loadConfig()
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    message.info('已重置为当前配置')
  } catch (e: any) {
    message.error('重置失败: ' + e)
  }
}

async function handleAddCustom() {
  if (!newId.value.trim() || !newName.value.trim()) {
    message.warning('ID 和显示名称不能为空')
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
    message.success('自定义 Agent 已添加')
  } catch (e: any) {
    message.error('添加失败: ' + e)
  }
}

async function handleRemoveCustom(id: string) {
  try {
    await configStore.removeCustomAgent(id)
    Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
    message.success('已移除')
  } catch (e: any) {
    message.error('移除失败: ' + e)
  }
}

onMounted(async () => {
  await configStore.loadConfig()
  Object.assign(form, JSON.parse(JSON.stringify(configStore.config)))
})
</script>

<template>
  <div class="settings-view">
    <div class="page-header">
      <h1>设置</h1>
    </div>

    <NCard title="远端仓库" class="settings-card">
      <NForm label-placement="left" label-width="140">
        <NFormItem label="仓库地址">
          <NInput v-model:value="form.remote_url" placeholder="Git 仓库地址" />
        </NFormItem>
        <NFormItem label="缓存目录">
          <NInput v-model:value="form.cache_path" placeholder="远端仓库本地缓存目录" />
        </NFormItem>
        <NFormItem label="自动同步">
          <NSwitch v-model:value="form.auto_sync" />
        </NFormItem>
      </NForm>
    </NCard>

    <NCard title="内置 Agent 路径" class="settings-card">
      <template #header-extra>
        <NText depth="3" style="font-size: 12px">全局路径 / 项目目录模式</NText>
      </template>
      <NForm label-placement="left" label-width="140">
        <NFormItem v-for="agent in builtInAgents" :key="agent.id" :label="agent.label">
          <NSpace vertical style="width: 100%">
            <NInput
              v-model:value="form.agent_global_paths[agent.id]"
              :placeholder="'全局 skill 路径'"
              size="small"
            />
            <NInput
              v-model:value="form.agent_project_patterns[agent.id]"
              :placeholder="'项目目录模式，如 {project}/.claude/skills'"
              size="small"
            />
          </NSpace>
        </NFormItem>
      </NForm>
    </NCard>

    <NCard title="自定义 Agent" class="settings-card">
      <!-- Existing custom agents -->
      <div v-if="form.custom_agent_ids.length > 0" class="custom-agent-list">
        <div v-for="cid in form.custom_agent_ids" :key="cid" class="custom-agent-item">
          <div class="custom-agent-header">
            <NText strong>{{ form.agent_display_names[cid] || cid }}</NText>
            <NTag size="small" type="info" round>自定义</NTag>
            <NPopconfirm @positive-click="handleRemoveCustom(cid)">
              <template #trigger>
                <NButton size="tiny" type="error" ghost>移除</NButton>
              </template>
              确认移除该自定义 Agent？
            </NPopconfirm>
          </div>
          <NForm label-placement="left" label-width="100" size="small">
            <NFormItem label="全局路径">
              <NInput v-model:value="form.agent_global_paths[cid]" />
            </NFormItem>
            <NFormItem label="项目目录模式">
              <NInput v-model:value="form.agent_project_patterns[cid]" placeholder="{project}/xxx/skills" />
            </NFormItem>
          </NForm>
        </div>
      </div>
      <NText v-else depth="3" style="font-size: 13px">暂无自定义 Agent</NText>

      <NDivider style="margin: 16px 0 12px">添加自定义 Agent</NDivider>
      <NForm label-placement="left" label-width="100" size="small">
        <NFormItem label="ID (英文)">
          <NInput v-model:value="newId" placeholder="如 my-agent" />
        </NFormItem>
        <NFormItem label="显示名称">
          <NInput v-model:value="newName" placeholder="如 My Custom Agent" />
        </NFormItem>
        <NFormItem label="全局路径">
          <NInput v-model:value="newGlobalPath" placeholder="如 C:/Users/xxx/.my-agent/skills（可留空自动生成）" />
        </NFormItem>
        <NFormItem label="项目目录模式">
          <NInput v-model:value="newProjectPattern" placeholder="如 {project}/.my-agent/skills（可留空自动生成）" />
        </NFormItem>
        <NFormItem>
          <NButton type="primary" @click="handleAddCustom">添加</NButton>
        </NFormItem>
      </NForm>
    </NCard>

    <NSpace class="actions">
      <NButton type="primary" @click="handleSave">保存配置</NButton>
      <NButton @click="handleReset">重置</NButton>
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
</style>
