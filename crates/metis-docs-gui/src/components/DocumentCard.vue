<template>
  <div
    class="overflow-hidden transition-all duration-200 shadow-lg cursor-move hover:shadow-xl document-drag"
    :style="{
      backgroundColor: theme.colors.background.elevated,
      border: 'none',
      width: '95%',
      margin: '0 auto',
      minHeight: '120px',
      boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
      borderRadius: '18px',
    }"
    :data-short-code="document.short_code"
    @click="$emit('click', document)"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- Colored Header Bar with Short Code -->
    <div 
      class="py-3 flex items-center justify-between"
      :style="{
        backgroundColor: theme.colors.interactive.primary,
        color: theme.colors.text.inverse,
        paddingLeft: '24px',
        paddingRight: '24px',
      }"
    >
      <span class="text-xs font-mono font-bold">
        {{ document.short_code }}
      </span>
      <span class="text-xs font-bold">
        {{ document.document_type.charAt(0).toUpperCase() }}
      </span>
    </div>

    <!-- Card Content -->
    <div 
      class="py-4"
      :style="{
        paddingLeft: '24px',
        paddingRight: '24px',
      }"
    >
      <!-- Title -->
      <h4 
        class="font-semibold mb-3 text-sm leading-tight"
        :style="{ 
          color: theme.colors.text.primary,
          display: '-webkit-box',
          WebkitLineClamp: 2,
          WebkitBoxOrient: 'vertical',
          overflow: 'hidden',
          minHeight: '2.5rem',
        }"
      >
        {{ document.title }}
      </h4>

      <!-- Footer with Phase and Date -->
      <div class="flex items-center justify-between">
        <!-- Phase -->
        <span
          v-if="document.phase"
          class="px-2 py-1 rounded-full text-xs font-medium"
          :style="{
            backgroundColor: phaseStyle.backgroundColor,
            color: phaseStyle.color,
            fontSize: '10px',
          }"
        >
          {{ document.phase }}
        </span>
        
        <!-- Date -->
        <div class="text-xs" :style="{ color: theme.colors.text.tertiary }">
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
}

interface Emits {
  (e: 'click', document: DocumentInfo): void
}

const props = defineProps<Props>()
defineEmits<Emits>()

const { theme } = useTheme()

const getPhaseColor = (phase?: string) => {
  switch (phase) {
    case 'draft':
    case 'todo':
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