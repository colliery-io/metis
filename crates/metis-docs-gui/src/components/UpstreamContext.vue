<template>
  <div v-if="isVisible" class="upstream-context">
    <!-- Header with toggle -->
    <button class="upstream-header" @click="isExpanded = !isExpanded">
      <div class="header-left">
        <svg
          :class="['expand-icon', { expanded: isExpanded }]"
          width="12" height="12" viewBox="0 0 24 24"
          fill="none" stroke="currentColor" stroke-width="2.5"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
        <span class="header-title">Upstream Context</span>
        <span v-if="upstreamDocuments.length > 0" class="header-count">{{ upstreamDocuments.length }}</span>
      </div>
      <div v-if="isExpanded" class="header-right">
        <button
          class="scope-toggle"
          :class="{ active: browseAll }"
          @click.stop="browseAll = !browseAll"
          title="Browse all workspaces"
        >
          {{ browseAll ? 'All Workspaces' : 'Parent Chain' }}
        </button>
      </div>
    </button>

    <!-- Content -->
    <Transition name="slide">
      <div v-if="isExpanded" class="upstream-body">
        <!-- Empty State -->
        <div v-if="upstreamDocuments.length === 0" class="empty-state">
          <p v-if="browseAll">No other workspaces found</p>
          <p v-else>No upstream references</p>
        </div>

        <!-- Document List -->
        <div v-else class="upstream-list">
          <!-- Group by workspace -->
          <div
            v-for="group in groupedDocuments"
            :key="group.workspace"
            class="workspace-group"
          >
            <div class="workspace-label">
              <span class="workspace-badge">{{ group.workspace }}</span>
              <span v-if="group.team" class="team-badge">{{ group.team }}</span>
            </div>

            <div
              v-for="doc in group.documents"
              :key="doc.short_code"
              class="upstream-doc"
              @click="$emit('view-document', doc)"
            >
              <div class="doc-header">
                <span class="doc-type-badge" :class="doc.document_type">
                  {{ doc.document_type.charAt(0).toUpperCase() }}
                </span>
                <span class="doc-short-code">{{ doc.short_code }}</span>
                <span class="doc-phase" :class="doc.phase">{{ doc.phase }}</span>
              </div>
              <div class="doc-title">{{ doc.title }}</div>
              <!-- Progress bar for initiatives with tasks -->
              <div v-if="doc.document_type === 'initiative' && doc.progress" class="progress-container">
                <div class="progress-bar">
                  <div
                    class="progress-fill"
                    :style="{ width: `${doc.progress.percent}%` }"
                  />
                </div>
                <span class="progress-label">{{ doc.progress.completed }}/{{ doc.progress.total }} tasks</span>
              </div>
              <div class="doc-meta">
                <span class="read-only-badge">Read-only</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { DocumentInfo } from '../lib/tauri-api'
import { MetisAPI } from '../lib/tauri-api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted } from 'vue'

interface UpstreamDocument extends DocumentInfo {
  workspace?: string
  team?: string
  is_remote?: boolean
  progress?: { completed: number; total: number; percent: number }
}

interface WorkspaceGroup {
  workspace: string
  team?: string
  documents: UpstreamDocument[]
}

defineEmits<{
  (e: 'view-document', doc: UpstreamDocument): void
}>()

const isVisible = ref(false)
const isExpanded = ref(false)
const browseAll = ref(false)
const allDocuments = ref<DocumentInfo[]>([])
let unlistenSyncCompleted: UnlistenFn | null = null

// Upstream documents are those from other workspaces (prefixed differently)
// For now, we detect them by looking at documents that appear after sync
// that have different short_code prefixes
const upstreamDocuments = computed<UpstreamDocument[]>(() => {
  if (!isVisible.value) return []

  // Get all documents and identify those from remote workspaces
  // Remote documents have short codes that don't match the local project prefix
  // For now, show documents that have references in parent chains
  const docs = allDocuments.value

  // Group documents by their short_code prefix
  const prefixGroups = new Map<string, DocumentInfo[]>()
  for (const doc of docs) {
    const prefix = doc.short_code.split('-')[0]
    if (!prefixGroups.has(prefix)) {
      prefixGroups.set(prefix, [])
    }
    prefixGroups.get(prefix)!.push(doc)
  }

  // If only one prefix, no upstream documents exist
  if (prefixGroups.size <= 1) return []

  // Find the "local" prefix (the one with the most documents, or from config)
  let localPrefix = ''
  let maxCount = 0
  for (const [prefix, docs] of prefixGroups) {
    if (docs.length > maxCount) {
      maxCount = docs.length
      localPrefix = prefix
    }
  }

  // All documents from non-local prefixes are "upstream"
  const remoteDocuments: UpstreamDocument[] = []
  for (const [prefix, docs] of prefixGroups) {
    if (prefix === localPrefix) continue

    for (const doc of docs) {
      const enriched: UpstreamDocument = {
        ...doc,
        workspace: prefix.toLowerCase(),
        is_remote: true,
      }

      // Calculate progress for initiatives
      if (doc.document_type === 'initiative') {
        const childTasks = allDocuments.value.filter(
          d => d.document_type === 'task' && d.initiative_id === doc.short_code
        )
        if (childTasks.length > 0) {
          const completed = childTasks.filter(t => t.phase === 'completed').length
          enriched.progress = {
            completed,
            total: childTasks.length,
            percent: Math.round((completed / childTasks.length) * 100),
          }
        }
      }

      remoteDocuments.push(enriched)
    }
  }

  if (browseAll.value) {
    return remoteDocuments
  }

  // In "Parent Chain" mode, only show documents that are referenced by local docs
  const localDocs = prefixGroups.get(localPrefix) || []
  const referencedCodes = new Set<string>()
  for (const doc of localDocs) {
    if (doc.initiative_id && !doc.initiative_id.startsWith(localPrefix)) {
      referencedCodes.add(doc.initiative_id)
    }
  }

  // Walk up parent chains
  const visited = new Set<string>()
  const queue = [...referencedCodes]
  while (queue.length > 0) {
    const code = queue.shift()!
    if (visited.has(code)) continue
    visited.add(code)
    const doc = allDocuments.value.find(d => d.short_code === code)
    if (doc?.initiative_id) {
      queue.push(doc.initiative_id)
    }
  }

  return remoteDocuments.filter(d => visited.has(d.short_code))
})

const groupedDocuments = computed<WorkspaceGroup[]>(() => {
  const groups = new Map<string, WorkspaceGroup>()
  for (const doc of upstreamDocuments.value) {
    const ws = doc.workspace || 'unknown'
    if (!groups.has(ws)) {
      groups.set(ws, { workspace: ws, team: doc.team, documents: [] })
    }
    groups.get(ws)!.documents.push(doc)
  }
  return Array.from(groups.values())
})

const loadDocuments = async () => {
  try {
    allDocuments.value = await MetisAPI.listDocuments()
  } catch {
    // Silently fail
  }
}

const checkVisibility = async () => {
  try {
    isVisible.value = await MetisAPI.isUpstreamConfigured()
    if (isVisible.value) {
      await loadDocuments()
    }
  } catch {
    isVisible.value = false
  }
}

onMounted(async () => {
  await checkVisibility()

  unlistenSyncCompleted = await listen('sync-completed', async () => {
    await loadDocuments()
  })
})

onUnmounted(() => {
  if (unlistenSyncCompleted) unlistenSyncCompleted()
})

// Expose for parent to trigger refresh
defineExpose({ checkVisibility })

// Watch browseAll to reload if needed
watch(browseAll, () => {
  // Reactivity handles this through the computed
})
</script>

<style scoped>
.upstream-context {
  border-top: 1px solid var(--color-border-primary);
  background: var(--color-background-secondary);
}

/* Header */
.upstream-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background 0.15s ease;
}

.upstream-header:hover {
  background: var(--color-background-tertiary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.expand-icon {
  transition: transform 0.2s ease;
  color: var(--color-text-tertiary);
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.header-title {
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.header-count {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-inverse);
  background: var(--color-interactive-primary);
  padding: 1px 6px;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
}

.header-right {
  display: flex;
  align-items: center;
}

.scope-toggle {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  padding: 3px 8px;
  border: 1px solid var(--color-border-primary);
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.scope-toggle:hover {
  background: var(--color-background-tertiary);
  color: var(--color-text-secondary);
}

.scope-toggle.active {
  background: var(--color-interactive-primary);
  color: var(--color-text-inverse);
  border-color: var(--color-interactive-primary);
}

/* Body */
.upstream-body {
  padding: 0 16px 12px;
  max-height: 300px;
  overflow-y: auto;
}

.empty-state {
  padding: 16px;
  text-align: center;
}

.empty-state p {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin: 0;
}

/* Workspace Groups */
.workspace-group {
  margin-bottom: 8px;
}

.workspace-label {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  margin-bottom: 4px;
}

.workspace-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  color: var(--color-interactive-primary);
  background: rgba(var(--color-interactive-primary-rgb, 59, 130, 246), 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.team-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  padding: 2px 6px;
  border: 1px solid var(--color-border-primary);
  border-radius: 4px;
}

/* Document Cards */
.upstream-doc {
  padding: 8px 10px;
  background: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  margin-bottom: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.upstream-doc:hover {
  border-color: var(--color-border-secondary);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
}

.doc-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.doc-type-badge {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: var(--color-text-inverse);
  flex-shrink: 0;
}

.doc-type-badge.vision { background: var(--color-document-vision); }
.doc-type-badge.strategy { background: var(--color-document-strategy); }
.doc-type-badge.initiative { background: var(--color-document-initiative); }
.doc-type-badge.task { background: var(--color-document-task); }
.doc-type-badge.adr { background: var(--color-document-adr); }

.doc-short-code {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.doc-phase {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: 3px;
  margin-left: auto;
}

.doc-phase.published,
.doc-phase.active,
.doc-phase.decided,
.doc-phase.ready {
  color: var(--color-interactive-primary);
  background: rgba(var(--color-interactive-primary-rgb, 59, 130, 246), 0.1);
}

.doc-phase.completed {
  color: var(--color-text-tertiary);
  background: var(--color-background-tertiary);
}

.doc-phase.draft,
.doc-phase.todo,
.doc-phase.discovery,
.doc-phase.shaping {
  color: var(--color-text-tertiary);
  background: var(--color-background-secondary);
}

.doc-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  line-height: 1.3;
}

/* Progress Bar */
.progress-container {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: var(--color-background-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-interactive-primary);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-label {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--color-text-tertiary);
  white-space: nowrap;
}

/* Meta */
.doc-meta {
  display: flex;
  align-items: center;
  gap: 6px;
}

.read-only-badge {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  background: var(--color-background-tertiary);
  padding: 1px 5px;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

/* Slide transition */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.slide-enter-from,
.slide-leave-to {
  max-height: 0;
  opacity: 0;
  padding-top: 0;
  padding-bottom: 0;
}
</style>
