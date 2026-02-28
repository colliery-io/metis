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
    <div class="dialog-container">
      <div class="dialog-header">
        <h2 class="dialog-title">
          Create New {{ getDocumentTypeLabel(boardType) }}
        </h2>
        <button
          @click="handleClose"
          class="close-button"
          @mouseenter="handleCloseButtonHover"
          @mouseleave="handleCloseButtonLeave"
        >
          ×
        </button>
      </div>

      <form @submit="handleSubmit" class="dialog-form">
        <div class="form-group">
          <label for="title" class="form-label">
            Title *
          </label>
          <input
            type="text"
            id="title"
            v-model="title"
            class="form-input"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :placeholder="`Enter ${getDocumentTypeLabel(boardType).toLowerCase()} title...`"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="description" class="form-label">
            Description
          </label>
          <textarea
            id="description"
            v-model="description"
            rows="3"
            class="form-input form-textarea"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            placeholder="Brief description of what this accomplishes..."
            :disabled="loading"
          />
        </div>

        <!-- Parent Selection -->
        <div v-if="boardTypeRequiresParent(boardType)" class="form-group">
          <label for="parent" class="form-label">
            Parent {{ getParentTypeLabel() }} *
          </label>
          <select
            id="parent"
            v-model="parentId"
            class="form-input"
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
            class="form-help-text"
          >
            No available {{ getParentTypeLabel().toLowerCase() }}s found. Create one first.
          </div>
        </div>

        <div v-if="boardType === 'initiative'" class="form-group">
          <label for="complexity" class="form-label">
            Complexity
          </label>
          <select
            id="complexity"
            v-model="complexity"
            class="form-input"
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

        <div v-if="boardType === 'strategy'" class="form-group">
          <label for="riskLevel" class="form-label">
            Risk Level
          </label>
          <select
            id="riskLevel"
            v-model="riskLevel"
            class="form-input"
            @focus="handleInputFocus"
            @blur="handleInputBlur"
            :disabled="loading"
          >
            <option value="low">Low - Well understood, minimal uncertainty</option>
            <option value="medium">Medium - Some unknowns, manageable risk</option>
            <option value="high">High - Significant unknowns, requires exploration</option>
          </select>
        </div>

        <div v-if="boardType === 'backlog'" class="form-group">
          <label for="ticketType" class="form-label">
            Ticket Type
          </label>
          <select
            id="ticketType"
            v-model="ticketType"
            class="form-input"
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

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="form-submit">
          <button
            type="submit"
            class="submit-button"
            :class="{ 'submit-button--disabled': loading || !title.trim() || (boardTypeRequiresParent(boardType) && !parentId) }"
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
  if (boardType === 'backlog') return false  // Backlog items never have parents by definition
  if (boardType === 'adr') return false      // ADRs don't require parents
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
  } catch (err) {
    // Failed to load project config
    // Default to full configuration if we can't load it
    projectConfig.value = { strategies_enabled: false, initiatives_enabled: true }
  }
}

const loadAvailableParents = async (boardType: BoardType) => {
  // Check if this board type needs parents based on configuration
  if (!boardTypeRequiresParent(boardType)) return
  
  loadingParents.value = true
  try {
    // boardType is guaranteed to need parents here
    const childType = boardType
    const parents = await getAvailableParents(childType)
    availableParents.value = parents
  } catch (err) {
    // Failed to load available parents
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
  try {
    if (isOpen) {
      // First load the project configuration
      if (!projectConfig.value) {
        await loadProjectConfig()
      }
      
      // Then check if this board type needs parents based on config
      if (boardTypeRequiresParent(boardType)) {
        await loadAvailableParents(boardType)
      } else {
        availableParents.value = []
      }
    }
  } catch (err) {
    // Error in CreateDocumentDialog watcher
  }
}, { immediate: true })

const handleSubmit = async (e: Event) => {
  e.preventDefault()
  
  if (!title.value.trim()) {
    error.value = 'Title is required'
    return
  }

  const needsParent = boardTypeRequiresParent(props.boardType)
  if (needsParent && !parentId.value) {
    error.value = `Parent ${getParentTypeLabel().toLowerCase()} is required`
    return
  }

  loading.value = true
  error.value = null

  try {
    // Map backlog to task since backlog items are tasks without parents
    const documentType = props.boardType === 'backlog' ? 'task' : props.boardType
    
    // Prepare tags for backlog items based on ticket type
    const tags: string[] = []
    if (props.boardType === 'backlog') {
      const typeTag = ticketType.value === 'bug' ? '#bug' 
                    : ticketType.value === 'feature' ? '#feature'
                    : ticketType.value === 'tech-debt' ? '#tech-debt'
                    : null // general has no tag
      if (typeTag) {
        tags.push(typeTag)
      }
    }

    const request: CreateDocumentRequest = {
      document_type: documentType,
      title: title.value.trim(),
      // Only include parent_id if this is NOT a backlog item and needsParent is true
      ...(props.boardType !== 'backlog' && needsParent && parentId.value && { parent_id: parentId.value }),
      ...(props.boardType === 'initiative' && { complexity: complexity.value }),
      ...(props.boardType === 'strategy' && { risk_level: riskLevel.value }),
      ...(tags.length > 0 && { tags }),
    }

    await createDocument(request)
    
    // Reset form and close dialog
    resetForm()
    emit('document-created')
    emit('close')
  } catch (err) {
    // Create document error
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

<style scoped>
.dialog-container {
  background-color: var(--color-background-elevated);
  border: 2px solid var(--color-interactive-primary);
  border-radius: 16px;
  width: max-content;
  min-width: 480px;
  max-width: min(600px, 90vw);
  max-height: 90vh;
  padding: 24px;
  position: relative;
  z-index: 10;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 10px 10px -5px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border-primary);
  flex-shrink: 0;
  width: 100%;
}

.dialog-title {
  color: var(--color-text-primary);
  font-size: 20px;
  font-weight: 700;
  margin: 0;
}

.close-button {
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  font-size: 24px;
  font-weight: bold;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.close-button:hover {
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.dialog-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex: 1;
  min-height: 0;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

.form-input {
  width: 100%;
  min-width: 0;
  padding: 12px 16px;
  background-color: var(--color-background-secondary);
  border: 2px solid var(--color-border-primary);
  border-radius: 8px;
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  outline: none;
  box-sizing: border-box;
}

.form-input:focus {
  border-color: var(--color-interactive-primary);
  box-shadow: 0 0 0 3px rgba(var(--color-interactive-primary-rgb), 0.1);
}

.form-input:disabled {
  background-color: var(--color-background-tertiary);
  color: var(--color-text-tertiary);
  cursor: not-allowed;
  opacity: 0.7;
}

.form-textarea {
  resize: vertical;
  min-height: 72px;
  font-family: inherit;
}

.form-help-text {
  color: var(--color-text-secondary);
  font-size: 12px;
  margin-top: 4px;
}

.error-message {
  color: var(--color-border-error, #ef4444);
  font-size: 14px;
  padding: 12px 16px;
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 8px;
  word-break: break-words;
}

.form-submit {
  display: flex;
  justify-content: center;
  padding-top: 4px;
}

.submit-button {
  background-color: var(--color-interactive-primary);
  color: var(--color-text-inverse);
  border: 2px solid var(--color-interactive-primary);
  border-radius: 12px;
  padding: 16px 32px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 200px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

.submit-button:hover:not(:disabled) {
  background-color: var(--color-interactive-primaryHover, var(--color-interactive-primary));
  transform: translateY(-1px);
  box-shadow: 0 6px 12px -1px rgba(0, 0, 0, 0.15);
}

.submit-button:active:not(:disabled) {
  transform: translateY(0);
}

.submit-button--disabled {
  background-color: var(--color-background-tertiary);
  color: var(--color-text-tertiary);
  border-color: var(--color-border-secondary);
  cursor: not-allowed;
  opacity: 0.6;
}

.submit-button--disabled:hover {
  transform: none;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

/* Custom scrollbar for form */
.dialog-form::-webkit-scrollbar {
  width: 6px;
}

.dialog-form::-webkit-scrollbar-track {
  background: var(--color-background-primary);
  border-radius: 3px;
}

.dialog-form::-webkit-scrollbar-thumb {
  background: var(--color-border-primary);
  border-radius: 3px;
}

.dialog-form::-webkit-scrollbar-thumb:hover {
  background: var(--color-interactive-primary);
}
</style>