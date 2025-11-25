<template>
  <div
    class="kanban-card"
    :class="[
      draggingEnabled ? 'kanbancard-drag' : 'nomoredragging',
      highlighted ? 'highlighted' : ''
    ]"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- Colored Header Bar with Short Code and Actions -->
    <div class="card-header">
      <div class="flex items-center gap-2">
        <span class="text-xs font-mono font-bold">
          {{ document.short_code }}
        </span>
        <span class="text-xs font-bold">
          {{ document.document_type.charAt(0).toUpperCase() }}
        </span>
      </div>
      
      <!-- Action Icons -->
      <div class="flex items-center gap-1">
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

    <!-- Card Content -->
    <div class="card-content">
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
  background-color: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  transition: all 0.2s ease;
  width: 100%;
  min-height: 120px;
  margin-bottom: 12px;
  cursor: default;
}

.kanban-card:hover {
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  transform: translateY(-2px);
}

.kanbancard-drag {
  cursor: move;
}

.kanbancard-drag:active {
  transform: rotate(2deg);
}

.nomoredragging {
  cursor: default;
}

.card-header {
  background-color: var(--color-interactive-primary);
  color: var(--color-text-inverse);
  padding: 8px 16px 8px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-header .text-xs {
  color: var(--color-text-inverse);
  font-size: 11px;
}

.action-button {
  padding: 4px;
  border-radius: 4px;
  transition: background-color 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}

.action-button:hover {
  background-color: rgba(0, 0, 0, 0.15);
}

.card-content {
  padding: 16px;
}

.card-title {
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
  margin: 0 0 12px 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 2rem;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.phase-badge {
  padding: 3px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.card-date {
  color: var(--color-text-tertiary);
  font-size: 11px;
  font-weight: 400;
}

/* Highlighted state from search */
.kanban-card.highlighted {
  animation: highlight-pulse 2s ease-out;
  box-shadow: 0 0 0 3px var(--color-interactive-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}

@keyframes highlight-pulse {
  0% {
    box-shadow: 0 0 0 3px var(--color-interactive-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    transform: scale(1.02);
  }
  50% {
    box-shadow: 0 0 0 6px var(--color-interactive-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    transform: scale(1.02);
  }
  100% {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    transform: scale(1);
  }
}
</style>