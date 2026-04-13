<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import { NConfigProvider, NMessageProvider, zhCN, dateZhCN } from 'naive-ui'
</script>

<template>
  <NConfigProvider :locale="zhCN" :date-locale="dateZhCN">
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
