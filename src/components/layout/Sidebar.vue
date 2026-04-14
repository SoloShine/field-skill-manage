<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { NMenu, NSelect, NText, NButton, NSpin, NIcon, useMessage } from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import { useConfigStore } from '@/stores/config'
import { useUpdateStore } from '@/stores/update'
import { useI18n } from 'vue-i18n'
import { useLocale } from '@/i18n/useLocale'
import { SUPPORTED_LOCALES } from '@/i18n'
import { useTheme } from '@/composables/useTheme'
import { computed, h, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { MoonOutline, SunnyOutline, GlobeOutline, FolderOutline, SettingsOutline } from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const configStore = useConfigStore()
const updateStore = useUpdateStore()
const { t } = useI18n()
const { currentLocale, setLocale } = useLocale()
const { isDark, toggleTheme } = useTheme()
const message = useMessage()

const menuOptions = computed<MenuOption[]>(() => [
  { label: () => t('nav.global'), key: 'global', icon: () => h(NIcon, { size: 18 }, () => h(GlobeOutline)) },
  { label: () => t('nav.project'), key: 'project', icon: () => h(NIcon, { size: 18 }, () => h(FolderOutline)) },
  { label: () => t('nav.settings'), key: 'settings', icon: () => h(NIcon, { size: 18 }, () => h(SettingsOutline)) },
])

const agentOptions = computed(() =>
  configStore.allAgents.map((a) => ({
    label: a.display_name,
    value: a.id,
  }))
)

const localeOptions = SUPPORTED_LOCALES.map((l) => ({
  label: l.label,
  value: l.value,
}))

const selectedAgent = computed({
  get: () => configStore.config.active_agent_id,
  set: (val: string) => {
    const displayName = configStore.allAgents.find(a => a.id === val)?.display_name || val
    configStore.setActiveAgent(val)
    message.info(t('sidebar.agentSwitched', { name: displayName }))
  },
})

function handleMenuSelect(key: string) {
  router.push({ name: key })
}

async function handleOpenRelease() {
  if (updateStore.updateInfo?.release_url) {
    await open(updateStore.updateInfo.release_url)
  }
}

onMounted(() => {
  configStore.loadAllAgents()
  updateStore.loadCurrentVersion()
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
        {{ t('sidebar.currentAgent') }}
      </NText>
      <NSelect
        v-model:value="selectedAgent"
        :options="agentOptions"
        size="small"
        :placeholder="t('sidebar.selectAgent')"
      />
    </div>

    <NMenu
      :value="route.name as string"
      :options="menuOptions"
      @update:value="handleMenuSelect"
    />

    <div class="sidebar-footer">
      <div class="version-info">
        <NText v-if="updateStore.currentVersion" depth="3" style="font-size: 12px">
          v{{ updateStore.currentVersion }}
        </NText>
        <NButton
          v-if="updateStore.updateInfo?.has_update"
          size="tiny"
          type="warning"
          ghost
          @click="handleOpenRelease"
        >
          {{ t('update.newVersionAvailable') }}
        </NButton>
        <NSpin v-if="updateStore.checking" :size="12" />
      </div>
      <div class="footer-actions">
        <NButton quaternary size="small" @click="toggleTheme" class="theme-btn">
          <template #icon>
            <NIcon size="16">
              <SunnyOutline v-if="isDark" />
              <MoonOutline v-else />
            </NIcon>
          </template>
        </NButton>
        <NSelect
          :value="currentLocale"
          :options="localeOptions"
          size="small"
          @update:value="setLocale"
          style="flex: 1"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  height: 100vh;
  background: var(--color-bg-sidebar);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}
.sidebar-header {
  padding: 20px 16px 12px;
  border-bottom: 1px solid var(--color-border-light);
}
.sidebar-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-accent);
  margin: 0;
  letter-spacing: -0.5px;
}
.sidebar-subtitle {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 2px;
}
.agent-selector {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border-light);
}
.sidebar-footer {
  margin-top: auto;
  padding: 12px 16px;
  border-top: 1px solid var(--color-border-light);
}
.version-info {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}
.footer-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.theme-btn {
  flex-shrink: 0;
}
</style>
