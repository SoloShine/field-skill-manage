<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import { NConfigProvider, NMessageProvider } from 'naive-ui'
import { useLocale } from '@/i18n/useLocale'

const { naiveLocale, naiveDateLocale } = useLocale()
</script>

<template>
  <NConfigProvider :locale="naiveLocale" :date-locale="naiveDateLocale">
    <NMessageProvider>
      <div class="app-layout">
        <Sidebar />
        <main class="main-content">
          <RouterView />
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
  padding: 24px;
}
</style>
