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
      class="relative shadow-2xl z-10"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        border: `3px solid ${theme.colors.interactive.primary}`,
        borderRadius: '24px',
        width: '90vw',
        maxWidth: '800px',
        height: '90vh',
        maxHeight: '900px',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px ${theme.colors.interactive.primary}20`
      }"
    >
      <!-- Header -->
      <div 
        class="flex items-center justify-between p-6 border-b"
        :style="{ borderColor: theme.colors.border.primary }"
      >
        <div class="flex items-center space-x-4">
          <div>
            <h2 
              class="text-xl font-bold"
              :style="{ color: theme.colors.text.primary }"
            >
              {{ document.title }}
            </h2>
            <div class="flex items-center space-x-3 mt-1">
              <span 
                class="text-sm font-mono"
                :style="{ color: theme.colors.text.secondary }"
              >
                {{ document.short_code }}
              </span>
              <span
                class="px-2 py-1 rounded-full text-xs font-medium"
                :style="{
                  backgroundColor: getPhaseColor(document.phase) + '20',
                  color: getPhaseColor(document.phase),
                }"
              >
                {{ document.phase }}
              </span>
              <span
                class="px-2 py-1 rounded text-xs font-medium"
                :style="{
                  backgroundColor: theme.colors.background.secondary,
                  color: theme.colors.text.secondary,
                }"
              >
                {{ document.document_type.toUpperCase() }}
              </span>
            </div>
          </div>
        </div>
        
        <div class="flex items-center space-x-3">
          <button
            v-if="!isEditing"
            @click="handleEditClick"
            class="px-6 py-3 rounded-lg transition-all font-semibold"
            :style="{
              backgroundColor: theme.colors.interactive.primary,
              color: theme.colors.text.inverse,
              border: `2px solid ${theme.colors.interactive.primary}`
            }"
            @mouseenter="handleEditButtonHover"
            @mouseleave="handleEditButtonLeave"
          >
            Edit
          </button>
          
          <template v-else>
            <button
              @click="handleCancelEdit"
              class="px-4 py-2 rounded-lg transition-all font-medium"
              :style="{
                backgroundColor: theme.colors.background.secondary,
                color: theme.colors.text.primary,
                border: `1px solid ${theme.colors.border.primary}`
              }"
              :disabled="saving"
            >
              Cancel
            </button>
            <button
              @click="handleSaveAndClose"
              class="px-6 py-3 rounded-lg transition-all font-semibold"
              :style="{
                backgroundColor: theme.colors.interactive.primary,
                color: theme.colors.text.inverse,
                border: `2px solid ${theme.colors.interactive.primary}`,
                opacity: saving ? 0.6 : 1
              }"
              :disabled="saving"
            >
              {{ saving ? 'Saving...' : 'Save' }}
            </button>
          </template>
          
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
        class="flex-1 overflow-hidden"
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
        
        <div v-else class="h-full">
          <TiptapEditor
            :content="content"
            :editable="isEditing"
            @update="handleContentUpdate"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
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
const saving = ref(false)
const error = ref<string | null>(null)
const documentContent = ref<DocumentContent | null>(null)
const isEditing = ref(props.initialEdit || false)

const loadDocument = async () => {
  if (!props.document || !props.isOpen || !currentProject.value?.path) return

  try {
    loading.value = true
    error.value = null
    isEditing.value = false // Reset to view mode when opening new document
    
    // Ensure project is loaded in the backend
    await MetisAPI.loadProject(currentProject.value.path)
    
    console.log('Reading document with short_code:', props.document.short_code)
    const docContent = await readDocument(props.document.short_code)
    documentContent.value = docContent
    content.value = docContent.content || ''
  } catch (err) {
    console.error('DocumentViewer load error:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load document'
  } finally {
    loading.value = false
  }
}

const handleSave = async () => {
  if (!props.document || !currentProject.value?.path) return

  try {
    saving.value = true
    error.value = null
    
    // Ensure project is loaded in the backend
    await MetisAPI.loadProject(currentProject.value.path)
    
    await updateDocument(props.document.short_code, content.value)
    
    // Refresh document content
    const docContent = await readDocument(props.document.short_code)
    documentContent.value = docContent
    
    emit('document-updated')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to save document'
  } finally {
    saving.value = false
  }
}

const handleEditClick = () => {
  isEditing.value = true
}

const handleCancelEdit = () => {
  // Reset content to original
  if (documentContent.value) {
    content.value = documentContent.value.content || ''
  }
  isEditing.value = false
}

const handleSaveAndClose = async () => {
  await handleSave()
  isEditing.value = false
}

const handleClose = () => {
  isEditing.value = false
  error.value = null
  emit('close')
}

const handleContentUpdate = (newContent: string) => {
  content.value = newContent
}

const handleEditButtonHover = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.interactive.primaryHover || theme.value.colors.interactive.primary
}

const handleEditButtonLeave = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.interactive.primary
}

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