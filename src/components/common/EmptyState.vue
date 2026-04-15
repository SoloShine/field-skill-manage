<script setup lang="ts">
import { NResult, NButton } from 'naive-ui'

defineProps<{
  title: string
  description: string
  actionLabel?: string
}>()

const emit = defineEmits<{ action: [] }>()
</script>

<template>
  <div class="empty-state-wrapper">
    <NResult status="info" :title="title" :description="description" />
    <div v-if="actionLabel" class="empty-action">
      <NButton type="primary" @click="emit('action')">{{ actionLabel }}</NButton>
    </div>
  </div>
</template>

<style scoped>
.empty-state-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.empty-action {
  margin-top: 4px;
}
</style>
