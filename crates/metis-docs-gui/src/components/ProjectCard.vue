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
    class="group relative overflow-hidden rounded-lg transition-all duration-200 cursor-pointer p-3"
    :style="{
      backgroundColor: isActive 
        ? theme.colors.interactive.secondary
        : theme.colors.background.elevated,
      border: `1px solid ${isActive 
        ? theme.colors.interactive.primary
        : theme.colors.border.primary}`
    }"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div class="flex items-center justify-between">
      <div class="flex-1 min-w-0 pr-3">
        <h3
          :class="[
            'font-medium truncate transition-colors',
            isActive ? 'text-interactive-primary' : 'text-primary group-hover:text-interactive-primary'
          ]"
          style="font-size: 14px"
        >
          {{ getSidebarProjectName() }}
        </h3>
        <p
          class="text-tertiary truncate mt-1 transition-colors"
          style="font-size: 11px"
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
console.log('ProjectCard component loaded for:', props.project.path)
console.log('isSidebar:', props.isSidebar)
console.log('onClick prop:', props.onClick)
console.log('Theme name:', themeName.value)
console.log('Background elevated:', theme.value.colors.background.elevated)
console.log('Interactive secondary:', theme.value.colors.interactive.secondary)

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
  console.log('ProjectCard handleClick called', props.project.path)
  if (props.onClick) {
    console.log('Calling props.onClick')
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


const handleMouseEnter = (e: Event) => {
  if (!props.isActive) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.background.secondary
    target.style.borderColor = theme.value.colors.interactive.primary
  }
}

const handleMouseLeave = (e: Event) => {
  if (!props.isActive) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.background.elevated
    target.style.borderColor = theme.value.colors.border.primary
  }
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