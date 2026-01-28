<template>
  <div 
    v-if="isOpen && document"
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="position: fixed; top: 0; left: 0; right: 0; bottom: 0;"
  >
    <!-- Backdrop -->
    <div 
      class="absolute inset-0 transition-opacity"
      :style="{ backgroundColor: theme.colors.background.overlay || 'rgba(0, 0, 0, 0.85)' }"
      @click="handleClose"
    />
    
    <!-- Dialog -->
    <div 
      class="relative shadow-2xl z-10 flex flex-col w-[90vw] max-w-3xl max-h-[90vh] overflow-hidden"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        border: `3px solid ${theme.colors.interactive.primary}`,
        borderRadius: '24px',
        boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px ${theme.colors.interactive.primary}20`
      }"
    >
      <!-- Header -->
      <div 
        class="flex items-center justify-between p-6 border-b"
        :style="{ borderColor: theme.colors.border.primary }"
      >
        <div class="flex-1 min-w-0">
          <div>
            <h2 
              class="text-xl font-bold"
              :style="{ color: theme.colors.text.primary }"
            >
              {{ document.title }}
            </h2>
            <div class="flex items-center justify-between mt-2" style="width: 100%;">
              <span
                class="text-sm font-mono"
                :style="{ color: theme.colors.text.secondary }"
              >
                {{ document.document_type.toUpperCase() }}: {{ document.short_code }}
              </span>
              <span
                class="px-3 py-1 rounded-full text-xs font-semibold tracking-wide"
                :style="{
                  backgroundColor: getPhaseColor(document.phase) + '20',
                  color: getPhaseColor(document.phase),
                }"
              >
                {{ document.phase }}
              </span>
            </div>
          </div>
        </div>
        
        <div class="flex items-center space-x-3">
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
            @click="toggleEditMode"
            class="px-4 py-2 text-sm font-medium rounded-lg transition-all"
            :style="{
              backgroundColor: isEditing ? theme.colors.status.active + '20' : theme.colors.interactive.primary,
              color: isEditing ? theme.colors.status.active : theme.colors.text.inverse,
              border: isEditing ? `1px solid ${theme.colors.status.active}` : 'none'
            }"
          >
            {{ isEditing ? 'Reading' : 'Edit' }}
          </button>

          <button
            @click="handleClose"
            class="font-bold transition-colors p-2 rounded-lg"
            :style="{ 
              color: theme.colors.text.secondary,
              backgroundColor: 'transparent',
              border: 'none',
              fontSize: '1.5rem',
              lineHeight: '1'
            }"
            @mouseenter="handleCloseButtonHover"
            @mouseleave="handleCloseButtonLeave"
          >
            ×
          </button>
        </div>
      </div>

      <!-- Content -->
      <div 
        class="flex-1 overflow-y-auto"
        style="min-height: 0;"
      >
        <div 
          v-if="loading"
          class="flex items-center justify-center h-full"
          :style="{ color: theme.colors.text.secondary }"
        >
          Loading document...
        </div>
        
        <div 
          v-else-if="error"
          class="p-6"
          :style="{ color: theme.colors.border.error }"
        >
          Error: {{ error }}
        </div>
        
        <div v-else class="flex flex-col h-full min-h-0">
          <TiptapEditor
            :content="content"
            :editable="isEditing"
            @update="handleContentUpdate"
            class="flex-1 min-h-0"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import TiptapEditor from './TiptapEditor.vue'
import { useProject } from '../composables/useProject'
import { useTheme } from '../composables/useTheme'
import { DocumentInfo, DocumentContent, readDocument, updateDocument, MetisAPI } from '../lib/tauri-api'

interface Props {
  isOpen: boolean
  document: DocumentInfo | null
  initialEdit?: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'document-updated'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { currentProject } = useProject()
const { theme } = useTheme()
const content = ref('')
const loading = ref(false)
const error = ref<string | null>(null)
const documentContent = ref<DocumentContent | null>(null)
const isEditing = ref(false) // Start in read mode
const saveStatus = ref<'saving' | 'saved' | 'error' | null>(null)
const originalFrontmatter = ref('')

// Computed
const saveStatusText = computed(() => {
  switch (saveStatus.value) {
    case 'saving': return 'Saving...'
    case 'saved': return 'Saved'
    case 'error': return 'Error saving'
    default: return ''
  }
})

const loadDocument = async () => {
  if (!props.document || !props.isOpen || !currentProject.value?.path) return

  try {
    loading.value = true
    error.value = null
    // Keep in edit mode when opening documents
    
    // Ensure project is loaded in the backend
    await MetisAPI.loadProject(currentProject.value.path)
    
    const docContent = await readDocument(props.document.short_code)
    documentContent.value = docContent
    
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
    // Document load error
    error.value = err instanceof Error ? err.message : 'Failed to load document'
  } finally {
    loading.value = false
  }
}

// Simple debounce utility
const debounce = (func: (...args: any[]) => void, wait: number) => {
  let timeout: number
  return (...args: any[]) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

const saveDocument = async () => {
  if (!props.document || !currentProject.value?.path) return


  try {
    saveStatus.value = 'saving'
    error.value = null
    
    // Ensure project is loaded in the backend
    await MetisAPI.loadProject(currentProject.value.path)
    
    // Reconstruct the full document with original frontmatter + updated content
    const fullContent = originalFrontmatter.value + content.value
    
    await updateDocument(props.document.short_code, fullContent)
    
    saveStatus.value = 'saved'
    setTimeout(() => {
      saveStatus.value = null
    }, 2000)
    
    emit('document-updated')
  } catch (err) {
    // Save failed
    saveStatus.value = 'error'
    error.value = err instanceof Error ? err.message : 'Failed to save document'
    setTimeout(() => {
      saveStatus.value = null
    }, 3000)
  }
}

// Debounced save function
const debouncedSave = debounce(saveDocument, 1000)

const toggleEditMode = () => {
  isEditing.value = !isEditing.value
}

const handleClose = () => {
  isEditing.value = false
  error.value = null
  emit('close')
}

const handleContentUpdate = (newContent: string) => {
  content.value = newContent
  // Auto-save when content changes (always in edit mode)
  debouncedSave()
}

// Edit button hover functions removed - no longer needed

const handleCloseButtonHover = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.background.secondary
  target.style.color = theme.value.colors.text.primary
}

const handleCloseButtonLeave = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = 'transparent'
  target.style.color = theme.value.colors.text.secondary
}

const getPhaseColor = (phase?: string) => {
  switch (phase) {
    case 'draft':
    case 'todo':
      return theme.value.colors.status?.draft || theme.value.colors.interactive.secondary
    case 'review':
    case 'doing':
    case 'active':
      return theme.value.colors.status?.active || theme.value.colors.interactive.primary
    case 'published':
    case 'completed':
      return theme.value.colors.status?.completed || '#10b981'
    case 'decided':
      return theme.value.colors.interactive.primary
    case 'superseded':
      return theme.value.colors.interactive?.danger || '#ef4444'
    default:
      return theme.value.colors.status?.draft || theme.value.colors.interactive.secondary
  }
}

// Watch for document/isOpen changes to load document
watch([() => props.document, () => props.isOpen, () => currentProject.value?.path], loadDocument, { immediate: true })
</script>