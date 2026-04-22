<script setup lang="ts">
import { NModal, NButton, NSpace, NTag, NText, NDataTable, NSpin } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { computed, h, ref } from 'vue'
import * as Diff from 'diff'
import { invoke } from '@tauri-apps/api/core'
import type { FileDiff, FileDiffStatus, SkillDiff, DiffFileContent } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  diff: SkillDiff
  target: string
  loadFileContent?: (filePath: string) => Promise<DiffFileContent>
}>()

const emit = defineEmits<{
  close: []
}>()

const selectedFile = ref<string | null>(null)
const loadingContent = ref(false)
const diffLines = ref<DiffLine[]>([])

interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  oldLineNo?: number
  newLineNo?: number
  content: string
}

const statusIcon = (status: FileDiffStatus) => {
  const map: Record<FileDiffStatus, { color: string; label: string }> = {
    Unchanged: { color: 'var(--color-status-same)', label: t('diff.unchanged') },
    Added: { color: '#10b981', label: t('diff.added') },
    Removed: { color: '#ef4444', label: t('diff.removed') },
    Modified: { color: '#f59e0b', label: t('diff.modified') },
  }
  const info = map[status]
  return h(NSpace, { size: 4, align: 'center' }, () => [
    h('span', { style: `display:inline-block;width:8px;height:8px;border-radius:50%;background:${info.color}` }),
    h(NText, { style: 'font-size: 12px' }, () => info.label),
  ])
}

function truncateHash(hash?: string): string {
  if (!hash) return '-'
  return hash.length > 16 ? hash.slice(0, 16) + '...' : hash
}

function formatSize(size?: number): string {
  if (size === undefined || size === null) return '-'
  if (size < 1024) return `${size} B`
  return `${(size / 1024).toFixed(1)} KB`
}

async function handleFileClick(row: FileDiff) {
  if (row.status === 'Unchanged') return
  if (selectedFile.value === row.path) {
    selectedFile.value = null
    diffLines.value = []
    return
  }

  selectedFile.value = row.path
  loadingContent.value = true
  try {
    const result = props.loadFileContent
      ? await props.loadFileContent(row.path)
      : await invoke<DiffFileContent>('get_diff_file_content', {
          skillName: props.diff.skillName,
          filePath: row.path,
          target: props.target,
        })
    computeDiffLines(result)
  } catch {
    diffLines.value = []
  } finally {
    loadingContent.value = false
  }
}

function computeDiffLines(result: DiffFileContent) {
  const local = result.localContent ?? ''
  const remote = result.remoteContent ?? ''
  const changes = Diff.diffLines(local, remote)

  const lines: DiffLine[] = []
  let oldLine = 1
  let newLine = 1

  for (const change of changes) {
    const textLines = change.value.replace(/\n$/, '').split('\n')

    if (change.added) {
      for (const line of textLines) {
        lines.push({ type: 'added', newLineNo: newLine++, content: line })
      }
    } else if (change.removed) {
      for (const line of textLines) {
        lines.push({ type: 'removed', oldLineNo: oldLine++, content: line })
      }
    } else {
      for (const line of textLines) {
        lines.push({ type: 'unchanged', oldLineNo: oldLine++, newLineNo: newLine++, content: line })
      }
    }
  }

  diffLines.value = lines
}

function isFileClickable(row: FileDiff): boolean {
  return row.status !== 'Unchanged'
}

const columns = computed<DataTableColumns<FileDiff>>(() => [
  {
    title: t('diff.status'),
    key: 'status',
    width: 100,
    render: (row) => statusIcon(row.status),
    sorter: (a, b) => a.status.localeCompare(b.status),
  },
  {
    title: t('diff.file'),
    key: 'path',
    render: (row) => h(NText, {
      style: `font-family: var(--font-mono); font-size: 13px; cursor: ${isFileClickable(row) ? 'pointer' : 'default'}; ${selectedFile.value === row.path ? 'color: var(--color-accent); font-weight: 600;' : ''}`,
    }, () => row.path),
  },
  {
    title: t('diff.localHash'),
    key: 'local_hash',
    width: 140,
    render: (row) => h(NText, { depth: row.localHash ? 2 : 3, style: 'font-family: var(--font-mono); font-size: 12px' }, () => truncateHash(row.localHash)),
  },
  {
    title: t('diff.remoteHash'),
    key: 'remote_hash',
    width: 140,
    render: (row) => h(NText, { depth: row.remoteHash ? 2 : 3, style: 'font-family: var(--font-mono); font-size: 12px' }, () => truncateHash(row.remoteHash)),
  },
  {
    title: t('diff.size'),
    key: 'size',
    width: 100,
    render: (row) => {
      if (row.status === 'Added') return h(NText, { style: 'color: #10b981; font-size: 12px' }, () => `+${formatSize(row.remoteSize)}`)
      if (row.status === 'Removed') return h(NText, { style: 'color: #ef4444; font-size: 12px' }, () => `-${formatSize(row.localSize)}`)
      if (row.status === 'Modified') {
        const delta = (row.remoteSize ?? 0) - (row.localSize ?? 0)
        const sign = delta >= 0 ? '+' : ''
        const color = delta >= 0 ? '#10b981' : '#ef4444'
        return h(NText, { style: `color: ${color}; font-size: 12px` }, () => `${sign}${formatSize(Math.abs(delta))}`)
      }
      return h(NText, { depth: 3, style: 'font-size: 12px' }, () => formatSize(row.localSize))
    },
  },
])

const addedLines = computed(() => diffLines.value.filter(l => l.type === 'added').length)
const removedLines = computed(() => diffLines.value.filter(l => l.type === 'removed').length)
</script>

<template>
  <NModal :show="true" @update:show="emit('close')">
    <div class="diff-dialog">
      <!-- Header -->
      <div class="diff-header">
        <span class="diff-title">{{ t('diff.title') }} — {{ diff.skillName }}</span>
        <NButton quaternary size="small" @click="emit('close')">{{ t('diff.close') }}</NButton>
      </div>

      <!-- Scrollable body -->
      <div class="diff-scroll">
        <div class="diff-summary">
          <NSpace size="small">
            <NTag v-if="diff.localVersion" size="small" type="info">{{ t('diff.localVersion') }} v{{ diff.localVersion }}</NTag>
            <NTag v-if="diff.remoteVersion" size="small" type="success">{{ t('diff.remoteVersion') }} v{{ diff.remoteVersion }}</NTag>
          </NSpace>
          <NSpace size="small" style="margin-top: 8px">
            <span v-if="diff.addedCount > 0" class="diff-badge diff-added">{{ t('diff.added') }} {{ diff.addedCount }}</span>
            <span v-if="diff.removedCount > 0" class="diff-badge diff-removed">{{ t('diff.removed') }} {{ diff.removedCount }}</span>
            <span v-if="diff.modifiedCount > 0" class="diff-badge diff-modified">{{ t('diff.modified') }} {{ diff.modifiedCount }}</span>
            <span v-if="diff.unchangedCount > 0" class="diff-badge diff-unchanged">{{ t('diff.unchanged') }} {{ diff.unchangedCount }}</span>
          </NSpace>
        </div>

        <NDataTable
          :columns="columns"
          :data="diff.files"
          :bordered="false"
          size="small"
          :max-height="selectedFile ? 180 : 300"
          virtual-scroll
          :row-props="(row: FileDiff) => isFileClickable(row) ? { style: 'cursor: pointer', onClick: () => handleFileClick(row) } : {}"
        />

        <!-- File content diff panel -->
        <div v-if="selectedFile" class="content-diff-panel">
          <div class="content-diff-header">
            <NText strong style="font-family: var(--font-mono); font-size: 13px">{{ selectedFile }}</NText>
            <NSpace size="small" align="center">
              <span v-if="addedLines > 0" class="diff-badge diff-added">+{{ addedLines }}</span>
              <span v-if="removedLines > 0" class="diff-badge diff-removed">-{{ removedLines }}</span>
              <NButton quaternary size="tiny" @click="selectedFile = null; diffLines = []">{{ t('diff.close') }}</NButton>
            </NSpace>
          </div>

          <NSpin :show="loadingContent">
            <div class="diff-viewer">
              <table v-if="diffLines.length > 0" class="diff-table">
                <tbody>
                  <tr v-for="(line, idx) in diffLines" :key="idx" :class="'diff-line-' + line.type">
                    <td class="line-no">{{ line.oldLineNo ?? '' }}</td>
                    <td class="line-no">{{ line.newLineNo ?? '' }}</td>
                    <td class="line-prefix">{{ line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' ' }}</td>
                    <td class="line-content"><pre>{{ line.content }}</pre></td>
                  </tr>
                </tbody>
              </table>
              <div v-else-if="!loadingContent" class="diff-empty">
                <NText depth="3">{{ t('diff.noContent') }}</NText>
              </div>
            </div>
          </NSpin>
        </div>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.diff-dialog {
  width: 820px;
  max-height: 85vh;
  background: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}
.diff-title {
  font-size: 16px;
  font-weight: 600;
}
.diff-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
}
.diff-summary {
  margin-bottom: 12px;
}
.diff-badge {
  display: inline-block;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}
.diff-added { background: rgba(16, 185, 129, 0.15); color: #10b981; }
.diff-removed { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
.diff-modified { background: rgba(245, 158, 11, 0.15); color: #f59e0b; }
.diff-unchanged { background: rgba(148, 163, 184, 0.15); color: #94a3b8; }

.content-diff-panel {
  margin-top: 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}
.content-diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-bg-tertiary);
  border-bottom: 1px solid var(--color-border);
}
.diff-viewer {
  max-height: 280px;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
}
.diff-table {
  width: 100%;
  border-collapse: collapse;
}
.diff-table td {
  padding: 0 8px;
  vertical-align: top;
}
.line-no {
  width: 40px;
  text-align: right;
  color: var(--color-text-muted);
  user-select: none;
  opacity: 0.6;
  padding-right: 8px;
  border-right: 1px solid var(--color-border-light);
}
.line-prefix {
  width: 16px;
  user-select: none;
  font-weight: 600;
}
.line-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

/* Line backgrounds */
.diff-line-added { background: rgba(16, 185, 129, 0.1); }
.diff-line-added .line-prefix { color: #10b981; }
.diff-line-removed { background: rgba(239, 68, 68, 0.1); }
.diff-line-removed .line-prefix { color: #ef4444; }
.diff-line-unchanged .line-prefix { color: var(--color-text-muted); }

.diff-empty {
  padding: 24px;
  text-align: center;
}
</style>
