<script setup lang="ts">
import { NResult } from 'naive-ui'

defineProps<{
  title: string
  description: string
}>()
</script>

<template>
  <NResult status="info" :title="title" :description="description" />
</template>
