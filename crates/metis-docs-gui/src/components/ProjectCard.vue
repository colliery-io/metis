<template>
  <!-- Project Browser Card -->
  <div
    v-if="!isSidebar"
    :class="[
      'p-4 border rounded-lg cursor-pointer transition-all',
      isSelected 
        ? 'border-blue-500 bg-blue-50' 
        : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50',
      !project.is_valid ? 'opacity-60' : ''
    ]"
    @click="handleSelect"
  >
    <div class="flex items-start justify-between">
      <div class="flex-1 min-w-0">
        <h3 class="text-lg font-medium text-gray-900 truncate">
          {{ getProjectName() }}
        </h3>
        <p class="text-sm text-gray-500 truncate mt-1" :title="project.path">
          {{ project.path }}
        </p>
      </div>
      <div class="ml-4 flex-shrink-0">
        <span class="text-2xl" :title="getStatusText()">
          {{ getStatusIcon() }}
        </span>
      </div>
    </div>
    
    <div class="mt-3">
      <span
        :class="[
          'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border',
          getStatusColor()
        ]"
      >
        {{ getStatusText() }}
      </span>
    </div>
  </div>

  <!-- Sidebar Project Card -->
  <div
    v-else
    :class="[
      'sidebar-project-card group',
      isActive ? 'sidebar-project-card-active' : 'sidebar-project-card-inactive'
    ]"
    @click="handleClick"
  >
    <div class="flex items-center justify-between">
      <div class="project-info">
        <h3
          class="project-title"
          :class="{ 'project-title-active': isActive }"
        >
          {{ getSidebarProjectName() }}
        </h3>
        <p
          class="project-path"
          :title="project.path"
        >
          {{ getSidebarProjectPath() }}
        </p>
      </div>
      
      <div class="flex items-center gap-2">
        <!-- Status indicator -->
        <div class="flex-shrink-0">
          <div
            :class="[
              'w-3 h-3 rounded-full transition-colors',
              isActive ? 'bg-interactive-primary' : 'bg-text-tertiary group-hover:bg-interactive-primary'
            ]"
          />
        </div>
        
        <!-- Remove button - always visible -->
        <button
          @click="handleRemove"
          class="w-5 h-5 flex items-center justify-center text-tertiary hover:text-interactive-danger transition-all duration-200"
          title="Remove project"
          style="font-size: 14px; background-color: transparent; border: none"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ProjectInfo } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'

interface Props {
  project: ProjectInfo
  // Browser card props
  onSelect?: (path: string) => void
  isSelected?: boolean
  // Sidebar card props  
  isActive?: boolean
  onClick?: () => void
  onRemove?: (e: Event) => void
  // Mode flag
  isSidebar?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isSelected: false,
  isSidebar: false,
  isActive: false
})

const emit = defineEmits<{
  click: []
  remove: [e: Event]
}>()

const { theme, themeName } = useTheme()

// Debug: log the current theme colors

const getStatusIcon = () => {
  if (!props.project.is_valid) return '❌'
  if (!props.project.vision_exists) return '⚠️'
  return '✅'
}

const getStatusText = () => {
  if (!props.project.is_valid) return 'Invalid Metis project'
  if (!props.project.vision_exists) return 'Missing vision document'
  return 'Valid Metis project'
}

const getStatusColor = () => {
  if (!props.project.is_valid) return 'text-red-600 bg-red-50 border-red-200'
  if (!props.project.vision_exists) return 'text-yellow-600 bg-yellow-50 border-yellow-200'
  return 'text-green-600 bg-green-50 border-green-200'
}

const getProjectName = () => {
  const parts = props.project.path.split(/[/\\]/)
  return parts[parts.length - 1] || props.project.path
}

const getSidebarProjectName = () => {
  const parts = props.project.path.split('/').filter(Boolean)
  return parts[parts.length - 1] || 'Unknown Project'
}

const getSidebarProjectPath = () => {
  const parts = props.project.path.split('/').filter(Boolean)
  const pathParts = parts.slice(0, -1)
  return pathParts.length > 2 ? `.../${pathParts.slice(-2).join('/')}` : pathParts.join('/') || '/'
}

const handleSelect = () => {
  if (props.onSelect) {
    props.onSelect(props.project.path)
  }
}

const handleClick = () => {
  if (props.onClick) {
    props.onClick()
  }
  emit('click')
}

const handleRemove = (e: Event) => {
  if (props.onRemove) {
    props.onRemove(e)
  }
  emit('remove', e)
}


</script>

<script lang="ts">
// Named exports for backwards compatibility
export default {
  name: 'ProjectCard'
}

// Export SidebarProjectCard as a separate component for clarity
export const SidebarProjectCard = {
  name: 'SidebarProjectCard'
}
</script>

<style scoped>
/* Sidebar project card styling */
.sidebar-project-card {
  position: relative;
  overflow: hidden;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
  padding: 12px;
  border: 1px solid var(--color-border-primary);
}

.sidebar-project-card-active {
  background-color: var(--color-interactive-secondary);
  border-color: var(--color-interactive-primary);
}

.sidebar-project-card-inactive {
  background-color: var(--color-background-elevated);
}

.sidebar-project-card-inactive:hover {
  background-color: var(--color-background-secondary);
  border-color: var(--color-interactive-primary);
}

.project-info {
  flex: 1;
  min-width: 0;
  padding-right: 12px;
}

.project-title {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 0.2s ease;
  margin: 0;
  color: var(--color-text-primary);
}

.project-title-active {
  color: var(--color-interactive-primary);
}

.sidebar-project-card-inactive:hover .project-title {
  color: var(--color-interactive-primary);
}

.project-path {
  color: var(--color-text-tertiary);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 2px 0 0 0;
  transition: color 0.2s ease;
}
</style>