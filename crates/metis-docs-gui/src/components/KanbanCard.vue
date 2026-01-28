<template>
  <div
    class="kanban-card"
    :class="[
      draggingEnabled ? 'kanbancard-drag' : 'nomoredragging',
      highlighted ? 'highlighted' : '',
      `doc-type-${document.document_type}`
    ]"
    :style="{ '--accent-color': accentColor }"
    @click="$emit('view', document)"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- Left Accent Stripe (via CSS) -->

    <!-- Card Content -->
    <div class="card-content">
      <!-- Header: Short Code Badge + Actions -->
      <div class="card-header-row">
        <span class="short-code-badge">
          {{ document.short_code }}
        </span>

        <!-- Action Icons (visible on hover) -->
        <div class="action-buttons">
          <!-- View/Edit Document -->
          <button
            @click.stop="$emit('view', document)"
            class="action-button"
            title="View/edit document"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
            </svg>
          </button>

          <!-- Promote to Task Board (only for backlog items) -->
          <button
            v-if="showPromoteButton"
            @click.stop="$emit('promote', document)"
            class="action-button"
            title="Start work on this item"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>

          <!-- Archive Document -->
          <button
            @click.stop="$emit('archive', document)"
            class="action-button"
            title="Archive this document"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 2H4c-1.1 0-2 .9-2 2v2c0 1.1.9 2 2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zM4 4h16v2H4V4zm14 16H6V8h12v12zm-8-4l2-2-2-2v4z"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Title -->
      <h4 class="card-title">
        {{ document.title }}
      </h4>

      <!-- Footer with Phase and Date -->
      <div class="card-footer">
        <!-- Phase -->
        <span
          v-if="document.phase"
          class="phase-badge"
          :style="{
            backgroundColor: phaseStyle.backgroundColor,
            color: phaseStyle.color,
          }"
        >
          {{ document.phase }}
        </span>

        <!-- Date -->
        <div class="card-date">
          {{ formatDate(document.updated_at) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useTheme } from '../composables/useTheme'
import type { DocumentInfo } from '../lib/tauri-api'

interface Props {
  document: DocumentInfo
  draggingEnabled?: boolean
  boardType?: string
  highlighted?: boolean
}

interface Emits {
  (e: 'promote', document: DocumentInfo): void
  (e: 'view', document: DocumentInfo): void
  (e: 'archive', document: DocumentInfo): void
}

const props = withDefaults(defineProps<Props>(), {
  draggingEnabled: true,
  highlighted: false
})

defineEmits<Emits>()

const { theme } = useTheme()

// Show promote button for backlog items
const showPromoteButton = computed(() => {
  return props.boardType === 'backlog' && props.document.document_type === 'task'
})

// Get accent color based on document type
const accentColor = computed(() => {
  const docType = props.document.document_type
  const colors = theme.value.colors.documentType
  switch (docType) {
    case 'vision': return colors.vision
    case 'strategy': return colors.strategy
    case 'initiative': return colors.initiative
    case 'task': return colors.task
    case 'adr': return colors.adr
    default: return colors.backlog
  }
})

const getPhaseColor = (phase?: string) => {
  switch (phase) {
    case 'draft':
    case 'todo':
    case 'backlog':
      return {
        backgroundColor: theme.value.colors.status.draft + '20',
        color: theme.value.colors.status.draft,
      }
    case 'review':
    case 'doing':
    case 'active':
      return {
        backgroundColor: theme.value.colors.status.active + '20',
        color: theme.value.colors.status.active,
      }
    case 'published':
    case 'completed':
      return {
        backgroundColor: theme.value.colors.status.completed + '20',
        color: theme.value.colors.status.completed,
      }
    case 'decided':
      return {
        backgroundColor: theme.value.colors.interactive.primary + '20',
        color: theme.value.colors.interactive.primary,
      }
    case 'superseded':
      return {
        backgroundColor: theme.value.colors.interactive.danger + '20',
        color: theme.value.colors.interactive.danger,
      }
    default:
      return {
        backgroundColor: theme.value.colors.status.draft + '20',
        color: theme.value.colors.status.draft,
      }
  }
}

const formatDate = (timestamp: number) => {
  try {
    return new Date(timestamp * 1000).toLocaleDateString()
  } catch {
    return 'Invalid date'
  }
}

const phaseStyle = computed(() => getPhaseColor(props.document.phase))

const handleMouseEnter = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)'
}

const handleMouseLeave = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)'
}
</script>

<style scoped>
.kanban-card {
  --accent-color: var(--color-interactive-primary);
  position: relative;
  background: linear-gradient(
    135deg,
    var(--color-background-elevated) 0%,
    color-mix(in srgb, var(--color-background-elevated) 97%, var(--accent-color)) 100%
  );
  border: 1px solid var(--color-border-primary);
  border-left: 4px solid var(--accent-color);
  border-radius: 4px 12px 12px 4px;
  box-shadow: 0 2px 8px -2px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  width: 100%;
  min-height: 110px;
  margin-bottom: 12px;
  cursor: default;
}

.kanban-card:hover {
  box-shadow: 0 8px 20px -4px rgba(0, 0, 0, 0.12), 0 0 0 1px var(--accent-color);
  transform: translateY(-2px);
}

.kanban-card:hover .action-buttons {
  opacity: 1;
  transform: translateX(0);
}

.kanbancard-drag {
  cursor: grab;
}

.kanbancard-drag:active {
  cursor: grabbing;
  transform: rotate(1.5deg) scale(1.02);
  box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.2);
}

.nomoredragging {
  cursor: default;
}

.card-content {
  padding: 14px 16px;
}

.card-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.short-code-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  color: var(--accent-color);
  background: color-mix(in srgb, var(--accent-color) 12%, transparent);
  padding: 3px 8px;
  border-radius: 4px;
  letter-spacing: 0.03em;
}

.action-buttons {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transform: translateX(8px);
  transition: all 0.2s ease;
}

.action-button {
  padding: 5px;
  border-radius: 6px;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
}

.action-button:hover {
  background-color: var(--color-background-tertiary);
  color: var(--color-text-primary);
}

.card-title {
  font-family: var(--font-display);
  color: var(--color-text-primary);
  font-size: var(--text-sm);
  font-weight: 600;
  line-height: 1.45;
  letter-spacing: -0.01em;
  margin: 0 0 12px 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 2.2em;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.phase-badge {
  font-family: var(--font-mono);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.card-date {
  font-family: var(--font-mono);
  color: var(--color-text-tertiary);
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.02em;
}

/* Highlighted state from search */
.kanban-card.highlighted {
  animation: highlight-pulse 2s ease-out;
  box-shadow: 0 0 0 3px var(--accent-color), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}

@keyframes highlight-pulse {
  0% {
    box-shadow: 0 0 0 3px var(--accent-color), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    transform: scale(1.02);
  }
  50% {
    box-shadow: 0 0 0 6px var(--accent-color), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    transform: scale(1.02);
  }
  100% {
    box-shadow: 0 2px 8px -2px rgba(0, 0, 0, 0.08);
    transform: scale(1);
  }
}
</style>