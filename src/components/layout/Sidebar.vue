<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { NMenu, NSelect, NText } from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import { useConfigStore } from '@/stores/config'
import { computed, onMounted } from 'vue'

const router = useRouter()
const route = useRoute()
const configStore = useConfigStore()

const menuOptions: MenuOption[] = [
  { label: '全局管理', key: 'global' },
  { label: '项目管理', key: 'project' },
  { label: '设置', key: 'settings' },
]

const agentOptions = computed(() =>
  configStore.allAgents.map((a) => ({
    label: a.display_name,
    value: a.id,
  }))
)

const selectedAgent = computed({
  get: () => configStore.config.active_agent_id,
  set: (val: string) => configStore.setActiveAgent(val),
})

function handleMenuSelect(key: string) {
  router.push({ name: key })
}

onMounted(() => {
  configStore.loadAllAgents()
})
</script>

<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-title">SPM</h2>
      <p class="sidebar-subtitle">Skill Package Manager</p>
    </div>

    <div class="agent-selector">
      <NText depth="3" style="font-size: 12px; margin-bottom: 4px; display: block">
        当前 Agent
      </NText>
      <NSelect
        v-model:value="selectedAgent"
        :options="agentOptions"
        size="small"
        placeholder="选择 Agent"
      />
    </div>

    <NMenu
      :value="route.name as string"
      :options="menuOptions"
      @update:value="handleMenuSelect"
    />
  </div>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  height: 100vh;
  background: #fff;
  border-right: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
}
.sidebar-header {
  padding: 20px 16px 12px;
  border-bottom: 1px solid #f0f0f0;
}
.sidebar-title {
  font-size: 20px;
  font-weight: 700;
  color: #18a058;
  margin: 0;
}
.sidebar-subtitle {
  font-size: 11px;
  color: #999;
  margin-top: 2px;
}
.agent-selector {
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
}
</style>
