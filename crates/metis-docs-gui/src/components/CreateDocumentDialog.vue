<template>
  <div 
    v-if="isOpen"
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
      class="relative shadow-2xl p-8 z-10"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        border: `3px solid ${theme.colors.interactive.primary}`,
        borderRadius: '24px',
        width: '500px',
        maxWidth: '90vw',
        maxHeight: '90vh',
        overflowY: 'auto',
        boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px ${theme.colors.interactive.primary}20`
      }"
    >
      <div class="flex items-center justify-between mb-6">
        <h2 
          class="text-xl font-bold"
          :style="{ color: theme.colors.text.primary }"
        >
          Create New {{ getDocumentTypeLabel(boardType) }}
        </h2>
        <button
          @click="handleClose"
          class="font-bold transition-colors p-6 rounded-lg"
          :style="{ 
            color: theme.colors.text.secondary,
            backgroundColor: 'transparent',
            border: 'none',
            fontSize: '2rem',
            lineHeight: '1'
          }"
          @mouseenter="handleCloseButtonHover"
          @mouseleave="handleCloseButtonLeave"
        >
          ×
        </button>
      </div>

      <form @submit="handleSubmit" class="space-y-6">
        <div>
          <label for="title" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Title *
          </label>
          <input
            type="text"
            id="title"
            v-model="title"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 text-lg font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :placeholder="`Enter ${getDocumentTypeLabel(boardType).toLowerCase()} title...`"
            :disabled="loading"
          />
        </div>

        <div>
          <label for="description" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Description
          </label>
          <textarea
            id="description"
            v-model="description"
            rows="3"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 resize-none font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            placeholder="Brief description of what this accomplishes..."
            :disabled="loading"
          />
        </div>

        <!-- Parent Selection -->
        <div v-if="boardTypeRequiresParent(boardType)">
          <label for="parent" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Parent {{ getParentTypeLabel() }} *
          </label>
          <select
            id="parent"
            v-model="parentId"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :disabled="loading || loadingParents"
          >
            <option value="">{{ loadingParents ? 'Loading parents...' : `Select parent ${getParentTypeLabel().toLowerCase()}...` }}</option>
            <option 
              v-for="parent in availableParents" 
              :key="parent.short_code" 
              :value="parent.short_code"
            >
              {{ parent.short_code }}: {{ parent.title }}
            </option>
          </select>
          <div 
            v-if="availableParents.length === 0 && !loadingParents" 
            class="text-sm mt-1" 
            :style="{ color: theme.colors.text.secondary }"
          >
            No available {{ getParentTypeLabel().toLowerCase() }}s found. Create one first.
          </div>
        </div>

        <div v-if="boardType === 'initiative'">
          <label for="complexity" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Complexity
          </label>
          <select
            id="complexity"
            v-model="complexity"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :disabled="loading"
          >
            <option value="xs">XS - Very Simple (1-2 days)</option>
            <option value="s">S - Simple (3-5 days)</option>
            <option value="m">M - Medium (1-2 weeks)</option>
            <option value="l">L - Large (3-4 weeks)</option>
            <option value="xl">XL - Very Large (1-2 months)</option>
          </select>
        </div>

        <div v-if="boardType === 'strategy'">
          <label for="riskLevel" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Risk Level
          </label>
          <select
            id="riskLevel"
            v-model="riskLevel"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :disabled="loading"
          >
            <option value="low">Low - Well understood, minimal uncertainty</option>
            <option value="medium">Medium - Some unknowns, manageable risk</option>
            <option value="high">High - Significant unknowns, requires exploration</option>
          </select>
        </div>

        <div v-if="boardType === 'backlog'">
          <label for="ticketType" class="block text-sm font-semibold mb-2" :style="{ color: theme.colors.text.primary }">
            Ticket Type
          </label>
          <select
            id="ticketType"
            v-model="ticketType"
            class="w-full px-4 py-3 rounded-lg focus:outline-none focus:ring-2 font-medium transition-all"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `2px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary
            }"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :disabled="loading"
          >
            <option value="feature">Feature - New functionality or enhancement</option>
            <option value="bug">Bug - Something that needs to be fixed</option>
            <option value="tech-debt">Tech Debt - Code improvement or refactoring</option>
            <option value="general">General - Documentation, process, or other work</option>
          </select>
        </div>

        <div v-if="error" class="mb-4 text-sm break-words" :style="{ color: theme.colors.border.error || '#ef4444' }">
          {{ error }}
        </div>

        <div class="flex justify-center pt-6">
          <button
            type="submit"
            class="px-12 py-5 rounded-2xl transition-all font-semibold text-xl"
            :style="{
              backgroundColor: loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) ? theme.colors.background.tertiary : theme.colors.interactive.primary,
              color: loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) ? theme.colors.text.tertiary : theme.colors.text.inverse,
              border: `2px solid ${loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) ? theme.colors.border.secondary : theme.colors.interactive.primary}`,
              cursor: loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) ? 'not-allowed' : 'pointer',
              opacity: loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) ? 0.6 : 1
            }"
            @mouseenter="handleSubmitButtonHover"
            @mouseleave="handleSubmitButtonLeave"
            :disabled="loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId)"
          >
            {{ loading ? 'Creating...' : `Create ${getDocumentTypeLabel(boardType)}` }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { createDocument, getAvailableParents, getProjectConfig, type CreateDocumentRequest, type ParentOption } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'
import type { BoardType } from '../types/board'

interface Props {
  isOpen: boolean
  boardType: BoardType
}

interface Emits {
  (e: 'close'): void
  (e: 'document-created'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { theme } = useTheme()

const title = ref('')
const description = ref('')
const complexity = ref('m')
const riskLevel = ref('medium')
const ticketType = ref('feature')
const parentId = ref('')
const loading = ref(false)
const error = ref<string | null>(null)
const availableParents = ref<ParentOption[]>([])
const loadingParents = ref(false)
const projectConfig = ref<any>(null)

// Helper function to check if a board type requires a parent
const boardTypeRequiresParent = (boardType: BoardType) => {
  if (!projectConfig.value) return false  // Don't show parent dropdown until config is loaded
  
  if (boardType === 'strategy') return false  // Strategies need Vision parent, which is implied
  if (boardType === 'initiative') {
    // Only need strategy parents if strategies are enabled
    return projectConfig.value.strategies_enabled ?? false
  }
  if (boardType === 'task') {
    // Need initiative parents if initiatives are enabled, otherwise no parent needed (direct config)
    return projectConfig.value.initiatives_enabled ?? false
  }
  return false
}

const getDocumentTypeLabel = (type: BoardType) => {
  switch (type) {
    case 'vision':
      return 'Vision'
    case 'strategy':
      return 'Strategy'
    case 'initiative':
      return 'Initiative'
    case 'task':
      return 'Task'
    case 'adr':
      return 'ADR (Architectural Decision Record)'
    case 'backlog':
      return 'Backlog Item'
    default:
      return 'Document'
  }
}

const loadProjectConfig = async () => {
  try {
    projectConfig.value = await getProjectConfig()
    console.log('Project config loaded:', projectConfig.value)
  } catch (err) {
    console.error('Failed to load project config:', err)
    // Default to full configuration if we can't load it
    projectConfig.value = { strategies_enabled: true, initiatives_enabled: true }
  }
}

const loadAvailableParents = async (boardType: BoardType) => {
  // Check if this board type needs parents based on configuration
  if (!boardTypeRequiresParent(boardType)) return
  
  loadingParents.value = true
  try {
    // boardType is guaranteed to need parents here
    const childType = boardType
    console.log('Loading available parents for child type:', childType)
    const parents = await getAvailableParents(childType)
    console.log('Received parents:', parents)
    availableParents.value = parents
  } catch (err) {
    console.error('Failed to load available parents:', err)
    error.value = 'Failed to load available parent documents'
  } finally {
    loadingParents.value = false
  }
}

const getParentTypeLabel = () => {
  switch (props.boardType) {
    case 'strategy':
      return 'Vision'
    case 'initiative':
      return 'Strategy'
    case 'task':
      return 'Initiative'
    default:
      return 'Parent'
  }
}

// Load available parents when dialog opens and board type changes
watch([() => props.isOpen, () => props.boardType], async ([isOpen, boardType]) => {
  console.log('CreateDocumentDialog watcher triggered:', { isOpen, boardType })
  try {
    if (isOpen) {
      // First load the project configuration
      if (!projectConfig.value) {
        await loadProjectConfig()
      }
      
      // Then check if this board type needs parents based on config
      if (boardTypeRequiresParent(boardType)) {
        console.log('Conditions met, loading parents for:', boardType)
        await loadAvailableParents(boardType)
      } else {
        console.log('No parent needed for:', boardType, 'with config:', projectConfig.value)
        availableParents.value = []
      }
    }
  } catch (err) {
    console.error('Error in CreateDocumentDialog watcher:', err)
  }
}, { immediate: true })

const handleSubmit = async (e: Event) => {
  e.preventDefault()
  
  if (!title.value.trim()) {
    error.value = 'Title is required'
    return
  }

  const needsParent = props.boardType === 'strategy' || props.boardType === 'initiative' || props.boardType === 'task'
  if (needsParent && !parentId.value) {
    error.value = `Parent ${getParentTypeLabel().toLowerCase()} is required`
    return
  }

  loading.value = true
  error.value = null

  try {
    // Map backlog to task since backlog items are tasks without parents
    const documentType = props.boardType === 'backlog' ? 'task' : props.boardType
    
    const request: CreateDocumentRequest = {
      document_type: documentType,
      title: title.value.trim(),
      ...(needsParent && parentId.value && { parent_id: parentId.value }),
      ...(props.boardType === 'initiative' && { complexity: complexity.value }),
      ...(props.boardType === 'strategy' && { risk_level: riskLevel.value }),
    }

    console.log('Creating document with request:', request)
    const result = await createDocument(request)
    console.log('Create document result:', result)
    
    // Reset form and close dialog
    resetForm()
    emit('document-created')
    emit('close')
  } catch (err) {
    console.error('Create document error:', err)
    error.value = err instanceof Error ? err.message : 'Failed to create document'
  } finally {
    loading.value = false
  }
}

const resetForm = () => {
  title.value = ''
  description.value = ''
  complexity.value = 'm'
  riskLevel.value = 'medium'
  ticketType.value = 'feature'
  parentId.value = ''
  error.value = null
}

const handleClose = () => {
  resetForm()
  emit('close')
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

const handleInputFocus = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.borderColor = theme.value.colors.interactive.primary
  target.style.boxShadow = `0 0 0 3px ${theme.value.colors.interactive.primary}20`
}

const handleInputBlur = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.borderColor = theme.value.colors.border.primary
  target.style.boxShadow = 'none'
}

const handleSubmitButtonHover = (e: Event) => {
  const isFormValid = !loading.value && title.value.trim() && !(boardTypeRequiresParent(props.boardType) && !parentId.value)
  if (isFormValid) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.interactive.primaryHover || theme.value.colors.interactive.primary
    target.style.transform = 'translateY(-1px)'
    target.style.boxShadow = `0 8px 25px -8px ${theme.value.colors.interactive.primary}40`
  }
}

const handleSubmitButtonLeave = (e: Event) => {
  const isFormValid = !loading.value && title.value.trim() && !(boardTypeRequiresParent(props.boardType) && !parentId.value)
  if (isFormValid) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.interactive.primary
    target.style.transform = 'translateY(0)'
    target.style.boxShadow = 'none'
  }
}
</script>