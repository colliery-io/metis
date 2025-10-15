<template>
  <div class="vision-display flex flex-col h-full">
    <!-- Vision Header - Fixed outside container -->
    <div class="p-6 pb-4 mb-6" :style="{ backgroundColor: theme.colors.background.primary }">
      <div class="flex items-center justify-between mb-2">
        <h1 
          class="text-3xl font-bold"
          :style="{ color: theme.colors.text.primary }"
        >
          {{ vision.title }}
        </h1>
        <div class="flex items-center gap-3">
          <!-- Phase Dropdown -->
          <select
            v-model="currentPhase"
            @change="handlePhaseChange"
            class="px-3 py-2 rounded-lg border font-medium"
            :style="{
              backgroundColor: theme.colors.background.secondary,
              borderColor: theme.colors.border.primary,
              color: theme.colors.text.primary,
            }"
            :disabled="saveStatus === 'saving'"
          >
            <option
              v-for="phase in visionPhases"
              :key="phase.key"
              :value="phase.key"
            >
              {{ phase.title }}
            </option>
          </select>

          <button
            v-if="!isEditing"
            @click="handleEditClick"
            class="px-4 py-2 rounded-lg font-medium transition-all hover:shadow-md"
            :style="{
              backgroundColor: theme.colors.interactive.primary,
              color: theme.colors.text.inverse,
            }"
          >
            Edit
          </button>
          
          <template v-else>
            <span 
              v-if="saveStatus"
              class="px-3 py-2 text-sm font-medium"
              :style="{ 
                color: saveStatus === 'error' ? theme.colors.interactive.danger : theme.colors.text.secondary 
              }"
            >
              {{ saveStatusText }}
            </span>
            <button
              @click="handleStopEdit"
              class="px-4 py-2 rounded-lg font-medium transition-all hover:shadow-md"
              :style="{
                backgroundColor: theme.colors.interactive.primary,
                color: theme.colors.text.inverse,
              }"
            >
              Done
            </button>
          </template>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium"
            :style="{
              backgroundColor: getPhaseColor(currentPhase) + '20',
              color: getPhaseColor(currentPhase),
            }"
          >
            {{ vision.short_code }}
          </span>
        </div>
      </div>
      <div 
        class="text-sm"
        :style="{ color: theme.colors.text.secondary }"
      >
        Published {{ formatDate(vision.updated_at) }}
      </div>
    </div>

    <!-- Vision Content Container - Now scrollable -->
    <div 
      class="rounded-lg border flex-1 flex flex-col min-h-0 overflow-y-auto"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        borderColor: theme.colors.border.primary,
      }"
    >
      <div v-if="loading" class="text-center py-8" :style="{ color: theme.colors.text.secondary }">
        Loading document...
      </div>
      <div v-else-if="error" class="text-center py-8" :style="{ color: theme.colors.border.error }">
        Error: {{ error }}
      </div>
      <div v-else class="flex-1 flex flex-col">
        <TiptapEditor 
          :content="content"
          :editable="isEditing"
          @update="handleContentUpdate"
          class="flex-1"
          :key="vision.short_code"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { readDocument, updateDocument, transitionPhase } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'
import TiptapEditor from './TiptapEditor.vue'
import type { DocumentInfo } from '../lib/tauri-api'
import { getBoardConfig } from '../lib/board-config'

interface Props {
  vision: DocumentInfo
}

interface Emits {
  (e: 'document-updated'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { theme } = useTheme()

const content = ref('')
const loading = ref(true)
const error = ref<string | null>(null)
const isEditing = ref(false)
const originalFrontmatter = ref('')
const currentPhase = ref(props.vision.phase)
const saveStatus = ref<'saving' | 'saved' | 'error' | null>(null)

// Get vision phases from board config
const visionBoardConfig = getBoardConfig('vision')
const visionPhases = visionBoardConfig?.phases || []

// Computed
const saveStatusText = computed(() => {
  switch (saveStatus.value) {
    case 'saving': return 'Saving...'
    case 'saved': return 'Saved'
    case 'error': return 'Error saving'
    default: return ''
  }
})

// Simple debounce utility
const debounce = (func: (...args: any[]) => void, wait: number) => {
  let timeout: number
  return (...args: any[]) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

const formatDate = (timestamp: number) => {
  try {
    return new Date(timestamp * 1000).toLocaleDateString()
  } catch {
    return 'Invalid date'
  }
}

const loadContent = async () => {
  try {
    loading.value = true
    error.value = null
    
    const docContent = await readDocument(props.vision.short_code)
    
    // Extract frontmatter and content
    const fullContent = docContent.content || ''
    const lines = fullContent.split('\n')
    if (lines[0] === '---') {
      const endIndex = lines.findIndex((line, index) => index > 0 && line === '---')
      if (endIndex > 0) {
        originalFrontmatter.value = lines.slice(0, endIndex + 1).join('\n') + '\n\n'
        content.value = lines.slice(endIndex + 1).join('\n').trim()
      } else {
        originalFrontmatter.value = ''
        content.value = fullContent
      }
    } else {
      originalFrontmatter.value = ''
      content.value = fullContent
    }
    
  } catch (err) {
    console.error('Failed to load document content:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load document'
  } finally {
    loading.value = false
  }
}

const saveDocument = async () => {
  if (!isEditing.value) return

  try {
    saveStatus.value = 'saving'
    error.value = null
    
    console.log('Auto-saving document:', props.vision.short_code, 'with content length:', content.value.length)
    
    // Reconstruct the full document with original frontmatter + updated content
    const fullContent = originalFrontmatter.value + content.value
    await updateDocument(props.vision.short_code, fullContent)
    console.log('Document auto-saved successfully')
    saveStatus.value = 'saved'
    setTimeout(() => {
      saveStatus.value = null
    }, 2000)
    
    emit('document-updated')
  } catch (err) {
    console.error('Failed to auto-save document:', err)
    saveStatus.value = 'error'
    error.value = err instanceof Error ? err.message : 'Failed to save document'
    setTimeout(() => {
      saveStatus.value = null
    }, 3000)
  }
}

// Debounced save function
const debouncedSave = debounce(saveDocument, 1000)

const handleEditClick = () => {
  isEditing.value = true
}

const handleStopEdit = () => {
  isEditing.value = false
}

const handleContentUpdate = (newContent: string) => {
  console.log('Vision content update received, length:', newContent.length)
  content.value = newContent
  // Auto-save when content changes (only in edit mode)
  if (isEditing.value) {
    console.log('Vision triggering debounced save')
    debouncedSave()
  }
}

const handlePhaseChange = async () => {
  try {
    saveStatus.value = 'saving'
    console.log('Transitioning phase:', props.vision.short_code, 'to', currentPhase.value)
    await transitionPhase(props.vision.short_code, currentPhase.value)
    console.log('Phase transition successful')
    saveStatus.value = 'saved'
    setTimeout(() => {
      saveStatus.value = null
    }, 2000)
    emit('document-updated')
  } catch (err) {
    console.error('Failed to transition phase:', err)
    // Revert to original phase on error
    currentPhase.value = props.vision.phase
    saveStatus.value = 'error'
    error.value = err instanceof Error ? err.message : 'Failed to change phase'
    setTimeout(() => {
      saveStatus.value = null
    }, 3000)
  }
}

const getPhaseColor = (phase: string) => {
  switch (phase) {
    case 'draft':
      return theme.value.colors.status?.draft || theme.value.colors.interactive.secondary
    case 'review':
      return theme.value.colors.status?.active || theme.value.colors.interactive.primary
    case 'published':
      return theme.value.colors.status?.completed || '#10b981'
    default:
      return theme.value.colors.status?.draft || theme.value.colors.interactive.secondary
  }
}

onMounted(() => {
  loadContent()
})
</script>