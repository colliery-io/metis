<template>
  <div class="specification-list">
    <table v-if="documents.length > 0" class="spec-table">
      <thead>
        <tr>
          <th class="sortable" @click="toggleSort('short_code')">
            Short Code
            <span v-if="sortKey === 'short_code'" class="sort-indicator">{{ sortAsc ? '▲' : '▼' }}</span>
          </th>
          <th class="sortable" @click="toggleSort('title')">
            Title
            <span v-if="sortKey === 'title'" class="sort-indicator">{{ sortAsc ? '▲' : '▼' }}</span>
          </th>
          <th class="sortable" @click="toggleSort('parent')">
            Parent
            <span v-if="sortKey === 'parent'" class="sort-indicator">{{ sortAsc ? '▲' : '▼' }}</span>
          </th>
          <th class="sortable" @click="toggleSort('phase')">
            Phase
            <span v-if="sortKey === 'phase'" class="sort-indicator">{{ sortAsc ? '▲' : '▼' }}</span>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="doc in sortedDocuments"
          :key="doc.short_code"
          class="spec-row"
          @click="$emit('view', doc)"
        >
          <td class="short-code-cell">
            <span class="short-code">{{ doc.short_code }}</span>
          </td>
          <td class="title-cell">{{ doc.title }}</td>
          <td class="parent-cell">
            <span v-if="getParent(doc)" class="parent-badge parent-link" @click.stop="$emit('view', getParent(doc)!)">
              {{ getParent(doc)!.short_code }}: {{ getParent(doc)!.title }}
            </span>
            <span v-else class="no-parent">—</span>
          </td>
          <td class="phase-cell">
            <span
              class="phase-badge"
              :style="getPhaseStyle(doc.phase)"
            >
              {{ doc.phase }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>

    <div v-else class="empty-state">
      No specifications found. Create one to get started.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTheme } from '../composables/useTheme'
import type { DocumentInfo } from '../lib/tauri-api'

interface Props {
  documents: DocumentInfo[]
  allDocuments: DocumentInfo[]
}

const props = defineProps<Props>()

defineEmits<{
  (e: 'view', document: DocumentInfo): void
}>()

const { theme } = useTheme()

type SortKey = 'short_code' | 'title' | 'parent' | 'phase'
const sortKey = ref<SortKey>('short_code')
const sortAsc = ref(true)

const toggleSort = (key: SortKey) => {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = true
  }
}

const getParent = (doc: DocumentInfo) => {
  if (!doc.parent_id) return null
  return props.allDocuments.find(d => d.short_code === doc.parent_id) || null
}

const sortedDocuments = computed(() => {
  const docs = [...props.documents]
  const dir = sortAsc.value ? 1 : -1

  docs.sort((a, b) => {
    switch (sortKey.value) {
      case 'short_code':
        return dir * a.short_code.localeCompare(b.short_code)
      case 'title':
        return dir * a.title.localeCompare(b.title)
      case 'parent': {
        const pa = getParent(a)?.short_code || ''
        const pb = getParent(b)?.short_code || ''
        return dir * pa.localeCompare(pb)
      }
      case 'phase':
        return dir * a.phase.localeCompare(b.phase)
      default:
        return 0
    }
  })

  return docs
})

const getPhaseStyle = (phase: string) => {
  switch (phase) {
    case 'discovery':
      return {
        backgroundColor: theme.value.colors.status.draft + '20',
        color: theme.value.colors.status.draft,
      }
    case 'drafting':
      return {
        backgroundColor: theme.value.colors.interactive.primary + '20',
        color: theme.value.colors.interactive.primary,
      }
    case 'review':
      return {
        backgroundColor: theme.value.colors.status.active + '20',
        color: theme.value.colors.status.active,
      }
    case 'published':
      return {
        backgroundColor: theme.value.colors.status.completed + '20',
        color: theme.value.colors.status.completed,
      }
    default:
      return {
        backgroundColor: theme.value.colors.status.draft + '20',
        color: theme.value.colors.status.draft,
      }
  }
}
</script>

<style scoped>
.specification-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 4px;
}

.spec-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.spec-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}

.spec-table th {
  background-color: var(--color-background-secondary);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 12px 16px;
  text-align: left;
  border-bottom: 2px solid var(--color-border-primary);
  user-select: none;
}

.spec-table th.sortable {
  cursor: pointer;
  transition: color 0.15s ease;
}

.spec-table th.sortable:hover {
  color: var(--color-interactive-primary);
}

.sort-indicator {
  margin-left: 4px;
  font-size: 9px;
}

.spec-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.spec-row:hover {
  background-color: var(--color-background-secondary);
}

.spec-row td {
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border-primary);
  font-size: 14px;
  color: var(--color-text-primary);
}

.short-code-cell {
  width: 140px;
}

.short-code {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-documentType-specification);
  background: color-mix(in srgb, var(--color-documentType-specification) 12%, transparent);
  padding: 4px 8px;
  border-radius: 4px;
  letter-spacing: 0.03em;
}

.title-cell {
  font-weight: 500;
}

.parent-cell {
  width: 250px;
}

.parent-badge {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.parent-link {
  color: var(--color-interactive-primary);
  cursor: pointer;
  font-family: var(--font-mono);
  padding: 3px 8px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--color-interactive-primary) 12%, transparent);
  transition: opacity 0.15s ease;
}

.parent-link:hover {
  text-decoration: underline;
  opacity: 0.85;
}

.no-parent {
  color: var(--color-text-tertiary);
}

.phase-cell {
  width: 120px;
}

.phase-badge {
  font-family: var(--font-mono);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.empty-state {
  text-align: center;
  padding: 48px 24px;
  color: var(--color-text-secondary);
  font-size: 16px;
}

/* Custom scrollbar */
.specification-list::-webkit-scrollbar {
  width: 6px;
}

.specification-list::-webkit-scrollbar-track {
  background: var(--color-background-primary);
  border-radius: 3px;
}

.specification-list::-webkit-scrollbar-thumb {
  background: var(--color-border-primary);
  border-radius: 3px;
}

.specification-list::-webkit-scrollbar-thumb:hover {
  background: var(--color-interactive-primary);
}
</style>
