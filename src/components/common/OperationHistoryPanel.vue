<script setup lang="ts">
import { NDrawer, NDrawerContent, NButton, NTag, NText, NSpace, NEmpty, NSpin, NPopconfirm, NScrollbar } from 'naive-ui'
import { ref, onMounted } from 'vue'
import type { OperationRecord, OperationType } from '@/types'
import { useSkillStore } from '@/stores/skill'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'

const { t } = useI18n()
const skillStore = useSkillStore()
const message = useMessage()

const records = ref<OperationRecord[]>([])
const loading = ref(false)
const rollbackingId = ref<string | null>(null)

function opTagType(op: OperationType): 'success' | 'warning' | 'error' {
  const map: Record<OperationType, 'success' | 'warning' | 'error'> = {
    Install: 'success',
    Update: 'warning',
    Uninstall: 'error',
  }
  return map[op]
}

function opLabel(op: OperationType): string {
  const map: Record<OperationType, string> = {
    Install: t('history.install'),
    Update: t('history.update'),
    Uninstall: t('history.uninstall'),
  }
  return map[op]
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleString()
  } catch {
    return ts
  }
}

function formatTarget(target: string): string {
  return target === 'global' ? t('history.global') : target.split(/[/\\]/).pop() || target
}

async function loadHistory() {
  loading.value = true
  try {
    records.value = await skillStore.getOperationHistory(50)
  } catch (e: any) {
    message.error(t('history.loadFailed', { error: e }))
  } finally {
    loading.value = false
  }
}

async function handleRollback(id: string) {
  rollbackingId.value = id
  try {
    await skillStore.rollbackOperation(id)
    message.success(t('history.rollbackSuccess'))
    await loadHistory()
  } catch (e: any) {
    message.error(t('history.rollbackFailed') + ': ' + e)
  } finally {
    rollbackingId.value = null
  }
}

async function handleClear() {
  try {
    await skillStore.clearHistory()
    records.value = []
    message.success(t('history.clearSuccess'))
  } catch (e: any) {
    message.error(String(e))
  }
}

onMounted(loadHistory)
</script>

<template>
  <NDrawer :show="true" :width="380" placement="right" @update:show="$emit('close')">
    <NDrawerContent :title="t('history.title')" closable>
      <template #footer>
        <NPopconfirm v-if="records.length > 0" @positive-click="handleClear">
          <template #trigger>
            <NButton size="small" type="error" ghost>{{ t('history.clear') }}</NButton>
          </template>
          {{ t('history.clearConfirm') }}
        </NPopconfirm>
      </template>

      <NSpin :show="loading">
        <NEmpty v-if="records.length === 0 && !loading" :description="t('history.empty')" />
        <NScrollbar v-else style="max-height: calc(100vh - 160px)">
          <div class="history-list">
            <div v-for="record in records" :key="record.id" class="history-item">
              <div class="history-header">
                <NTag :type="opTagType(record.operation)" size="small" round>
                  {{ opLabel(record.operation) }}
                </NTag>
                <NText depth="3" style="font-size: 11px">{{ formatTime(record.timestamp) }}</NText>
              </div>
              <div class="history-body">
                <NText strong style="font-size: 13px">{{ record.skillName }}</NText>
                <NSpace size="small" style="margin-top: 2px">
                  <NText depth="3" style="font-size: 12px">{{ formatTarget(record.target) }}</NText>
                  <template v-if="record.versionBefore || record.versionAfter">
                    <NText depth="3" style="font-size: 11px; font-family: var(--font-mono)">
                      v{{ record.versionBefore || '?' }} → v{{ record.versionAfter || '?' }}
                    </NText>
                  </template>
                </NSpace>
              </div>
              <div class="history-actions">
                <NButton
                  v-if="record.rollbackAvailable"
                  size="tiny"
                  :loading="rollbackingId === record.id"
                  @click="handleRollback(record.id)"
                >
                  {{ t('history.rollback') }}
                </NButton>
                <NText v-else depth="3" style="font-size: 11px">{{ t('history.noRollback') }}</NText>
              </div>
            </div>
          </div>
        </NScrollbar>
      </NSpin>
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped>
.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.history-item {
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border-light);
}
.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}
.history-body {
  margin-bottom: 4px;
}
.history-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
