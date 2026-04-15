<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import { NConfigProvider, NMessageProvider, darkTheme } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { useLocale } from '@/i18n/useLocale'
import { useUpdateStore } from '@/stores/update'
import { useTheme } from '@/composables/useTheme'
import { computed, onMounted } from 'vue'

const { naiveLocale, naiveDateLocale } = useLocale()
const updateStore = useUpdateStore()
const { isDark, accentColor } = useTheme()

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    primaryColor: accentColor.value.primary,
    primaryColorHover: accentColor.value.hover,
    primaryColorPressed: accentColor.value.pressed,
    primaryColorSuppl: accentColor.value.primary,
    borderRadius: '8px',
    borderRadiusSmall: '6px',
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    fontFamilyMono: "'JetBrains Mono', Consolas, Monaco, monospace",
  },
}))

const theme = computed(() => isDark.value ? darkTheme : undefined)

onMounted(async () => {
  await updateStore.loadCurrentVersion()
  await updateStore.checkForUpdates()
})
</script>

<template>
  <NConfigProvider :locale="naiveLocale" :date-locale="naiveDateLocale" :theme="theme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <div class="app-layout">
        <Sidebar />
        <main class="main-content">
          <router-view v-slot="{ Component }">
            <transition name="fade-slide">
              <component :is="Component" :key="$route.fullPath" />
            </transition>
          </router-view>
        </main>
      </div>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 24px;
  display: flex;
  flex-direction: column;
  position: relative;
}
</style>
