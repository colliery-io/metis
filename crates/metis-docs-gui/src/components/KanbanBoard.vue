<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div 
      class="flex items-center justify-between p-6 border-b"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        borderColor: theme.colors.border.primary,
      }"
    >
      <div>
        <h1 
          class="text-2xl font-bold"
          :style="{ color: theme.colors.text.primary }"
        >
          {{ boardConfig?.title || 'Document Board' }}
        </h1>
        <p 
          class="text-sm mt-1"
          :style="{ color: theme.colors.text.secondary }"
        >
          {{ boardConfig?.description || 'Manage your documents' }}
        </p>
      </div>
      <div class="flex items-center gap-3">
        <button
          v-if="currentBoard !== 'vision'"
          @click="showCreateDialog = true"
          class="px-12 py-6 rounded-xl transition-colors font-bold"
          :style="{
            backgroundColor: theme.colors.interactive.secondary,
            color: theme.colors.interactive.primary,
            border: `2px solid ${theme.colors.interactive.primary}`,
            fontSize: '18px'
          }"
          @mouseenter="handleCreateButtonHover"
          @mouseleave="handleCreateButtonLeave"
        >
          + Create Ticket
        </button>
      </div>
    </div>

    <!-- Board Navigation -->
    <div class="flex gap-4 p-4 bg-secondary border-b border-primary">
      <button 
        v-for="board in availableBoards" 
        :key="board"
        @click="handleBoardChange(board)"
        :class="board === currentBoard ? 'bg-interactive-primary text-inverse' : 'bg-elevated text-secondary'"
        class="px-4 py-2 rounded"
      >
        {{ board }} ({{ documentCounts[board] || 0 }})
      </button>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="flex items-center justify-center min-h-64">
      <div :style="{ color: theme.colors.text.secondary }">Loading documents...</div>
    </div>

    <!-- Error State -->
    <div 
      v-else-if="error"
      class="rounded-lg p-4 m-6"
      :style="{
        backgroundColor: theme.colors.background.secondary,
        border: `1px solid ${theme.colors.border.error}`,
      }"
    >
      <div 
        class="font-medium"
        :style="{ color: theme.colors.border.error }"
      >
        Error loading documents
      </div>
      <div 
        class="text-sm mt-1"
        :style="{ color: theme.colors.border.error }"
      >
        {{ error }}
      </div>
    </div>

    <!-- Kanban Columns - hide when there are published visions on vision board -->
    <div v-else-if="!(hasPublishedVisions && currentBoard === 'vision')" class="flex-1 p-6 overflow-hidden">
      <div class="h-full flex gap-4 overflow-x-auto">
        <div 
          v-for="phase in boardConfig?.phases || []"
          :key="phase.key" 
          class="flex-shrink-0" 
          :style="{ width: columnWidth }"
          :data-phase="phase.key"
        >
          <div class="h-full bg-elevated rounded-lg border border-primary">
            <div class="p-4 border-b border-primary">
              <h3 class="font-semibold text-primary">{{ phase.title }}</h3>
            </div>
            <VueDraggable
              v-model="filteredDocumentsByPhase[phase.key]"
              :group="{ name: 'documents', pull: true, put: true }"
              :animation="200"
              ghost-class="opacity-50"
              chosen-class="scale-105"
              class="p-4 space-y-2 overflow-y-auto min-h-32"
              style="max-height: calc(100vh - 300px)"
              :onAdd="(evt) => handleDocumentAdd(evt, phase.key)"
              :onRemove="(evt) => handleDocumentRemove(evt, phase.key)"
              :onStart="(evt) => handleDragStart(evt)"
              :onEnd="(evt) => handleDragEnd(evt)"
              :onChange="(evt) => handleDocumentChange(evt, phase.key)"
            >
              <DocumentCard
                v-for="doc in filteredDocumentsByPhase[phase.key] || []" 
                :key="doc.short_code"
                :document="doc"
                :data-id="doc.short_code"
                @click="handleDocumentClick"
              />
            </VueDraggable>
          </div>
        </div>
      </div>
    </div>

    <!-- Published Vision Content -->
    <div v-if="hasPublishedVisions && currentBoard === 'vision'" class="flex-1 p-6 overflow-hidden">
      <div class="h-full overflow-y-auto">
        <VisionDisplay 
          v-for="vision in publishedVisions" 
          :key="vision.short_code"
          :vision="vision"
          @document-updated="handleDocumentUpdated"
        />
      </div>
    </div>

    <!-- Create Document Dialog -->
    <CreateDocumentDialog
      v-if="showCreateDialog"
      :isOpen="showCreateDialog"
      :boardType="currentBoard"
      @close="showCreateDialog = false"
      @document-created="handleDocumentCreated"
    />

    <!-- Document Viewer -->
    <DocumentViewer
      v-if="viewingDocument"
      :isOpen="!!viewingDocument"
      :document="viewingDocument"
      @close="viewingDocument = null"
      @document-updated="handleDocumentUpdated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { VueDraggable } from 'vue-draggable-plus'
import { useProject } from '../composables/useProject'
import { useTheme } from '../composables/useTheme'
import { listDocuments, MetisAPI, getProjectConfig, transitionPhase } from '../lib/tauri-api'
import { getBoardConfig, getDocumentsByPhase } from '../lib/board-config'
import type { DocumentInfo, ProjectConfig } from '../lib/tauri-api'
import type { BoardType } from '../types/board'
import CreateDocumentDialog from './CreateDocumentDialog.vue'
import DocumentViewer from './DocumentViewer.vue'
import DocumentCard from './DocumentCard.vue'
import VisionDisplay from './VisionDisplay.vue'

const availableBoards: BoardType[] = ['vision', 'initiative', 'task', 'adr', 'backlog']

interface Props {
  onBackToProjects?: () => void
}

defineProps<Props>()

const { currentProject } = useProject()
const { theme } = useTheme()

const documents = ref<DocumentInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const currentBoard = ref<BoardType>('vision')
const showCreateDialog = ref(false)
const viewingDocument = ref<DocumentInfo | null>(null)
const projectConfig = ref<ProjectConfig | null>(null)

// Computed properties
const boardConfig = computed(() => getBoardConfig(currentBoard.value))
const documentsByPhase = ref<Record<string, DocumentInfo[]>>({})

const documentCounts = computed(() => ({
  vision: documents.value.filter(d => d.document_type === 'vision').length,
  strategy: documents.value.filter(d => d.document_type === 'strategy').length,
  initiative: documents.value.filter(d => d.document_type === 'initiative').length,
  task: documents.value.filter(d => d.document_type === 'task').length,
  adr: documents.value.filter(d => d.document_type === 'adr').length,
  backlog: documents.value.filter(d => 
    d.document_type === 'task' && !d.filepath.includes('initiatives/')
  ).length,
}))

const columnWidth = computed(() => {
  const columnCount = boardConfig.value?.phases.length || 1
  const minWidth = 180
  const availableWidth = `calc((100vw - 320px - ${columnCount * 16}px) / ${columnCount})`
  return `max(${minWidth}px, ${availableWidth})`
})

// Special handling for published visions - show as full content instead of cards
const publishedVisions = computed(() => {
  if (currentBoard.value !== 'vision') return []
  return documentsByPhase.value.published || []
})

const hasPublishedVisions = computed(() => publishedVisions.value.length > 0)

// For vision board, filter out published documents from kanban display
const filteredDocumentsByPhase = computed(() => {
  if (currentBoard.value !== 'vision') return documentsByPhase.value
  
  const filtered = { ...documentsByPhase.value }
  // Show published visions as content, not in kanban
  filtered.published = []
  return filtered
})

// Helper functions
const formatDate = (timestamp: number) => {
  try {
    return new Date(timestamp * 1000).toLocaleDateString()
  } catch {
    return 'Invalid date'
  }
}

// Methods
const loadDocuments = async () => {
  if (!currentProject.value?.path) return

  try {
    loading.value = true
    error.value = null
    
    // First ensure the project is loaded in the backend
    await MetisAPI.loadProject(currentProject.value.path)
    
    // Get project configuration
    const config = await getProjectConfig()
    projectConfig.value = config
    
    // Then get the documents
    const docs = await listDocuments()
    documents.value = docs
    updateDocumentsByPhase()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load documents'
  } finally {
    loading.value = false
  }
}

const handleDocumentClick = (document: DocumentInfo) => {
  viewingDocument.value = document
}

const handleDocumentCreated = async () => {
  await loadDocuments()
}

const handleDocumentUpdated = async () => {
  await loadDocuments()
}

const handleBoardChange = (newBoard: BoardType) => {
  currentBoard.value = newBoard
  updateDocumentsByPhase()
}

const updateDocumentsByPhase = () => {
  const newDocsByPhase = getDocumentsByPhase(documents.value, currentBoard.value)
  console.log('Updated documents by phase:', newDocsByPhase)
  documentsByPhase.value = newDocsByPhase
}

let draggedDocument: DocumentInfo | null = null

const handleDragStart = (evt: any) => {
  console.log('Drag start event:', evt)
  const element = evt.item
  const shortCode = element.querySelector('.font-medium')?.textContent
  console.log('Found short code:', shortCode)
  if (shortCode) {
    draggedDocument = documents.value.find(doc => doc.short_code === shortCode) || null
    console.log('Set draggedDocument:', draggedDocument)
  }
}

const handleDragEnd = async (evt: any) => {
  console.log('Drag end event:', evt)
  console.log('Event from:', evt.from)
  console.log('Event to:', evt.to)
  console.log('Are from and to the same?', evt.from === evt.to)
  
  if (draggedDocument) {
    console.log('DraggedDocument exists, checking for phase change...')
    
    // Find the target phase by looking at the data-phase attribute
    const targetColumn = evt.to.closest('[data-phase]')
    let targetPhase = targetColumn?.getAttribute('data-phase')
    
    console.log('Target column element:', targetColumn)
    console.log('Target phase from DOM:', targetPhase)
    
    // If closest doesn't work, try looking at parent elements
    if (!targetPhase) {
      console.log('Trying parent element approach...')
      let current = evt.to.parentElement
      while (current && !current.hasAttribute('data-phase')) {
        current = current.parentElement
      }
      targetPhase = current?.getAttribute('data-phase')
      console.log('Fallback target phase:', targetPhase)
    }
    
    console.log('Dragged document current phase:', draggedDocument.phase)
    console.log('Target phase:', targetPhase)
    
    if (targetPhase && targetPhase !== draggedDocument.phase) {
      try {
        console.log(`Calling transitionPhase('${draggedDocument.short_code}', '${targetPhase}')`)
        await transitionPhase(draggedDocument.short_code, targetPhase)
        console.log(`Successfully transitioned ${draggedDocument.short_code} to ${targetPhase}`)
        
        // Reload documents to reflect the change
        await loadDocuments()
      } catch (error) {
        console.error('Failed to transition document phase:', error)
        console.error('Document short_code:', draggedDocument.short_code)
        console.error('Target phase:', targetPhase)
        console.error('Document info:', draggedDocument)
        // Reload documents to revert the UI change
        await loadDocuments()
      }
    }
  }
  
  draggedDocument = null
}

const handleDocumentAdd = async (evt: any, targetPhase: string) => {
  console.log('Document added to phase:', targetPhase, evt)
  
  if (draggedDocument) {
    const oldPhase = draggedDocument.phase
    console.log(`Moving document ${draggedDocument.short_code} from ${oldPhase} to ${targetPhase}`)
    
    if (oldPhase === targetPhase) {
      console.log('No phase change needed - same phase')
      return
    }

    try {
      console.log(`Calling transitionPhase('${draggedDocument.short_code}', '${targetPhase}')`)
      await transitionPhase(draggedDocument.short_code, targetPhase)
      console.log(`Successfully transitioned ${draggedDocument.short_code} to ${targetPhase}`)
      
      // Reload documents to reflect the change
      await loadDocuments()
    } catch (error) {
      console.error('Failed to transition document phase:', error)
      console.error('Document short_code:', draggedDocument.short_code)
      console.error('Target phase:', targetPhase)
      console.error('Document info:', draggedDocument)
      // Reload documents to revert the UI change
      await loadDocuments()
      // TODO: Show error notification to user
    }
  }
}

const handleDocumentRemove = (evt: any, sourcePhase: string) => {
  console.log('Document removed from phase:', sourcePhase, evt)
}

const handleDocumentChange = (evt: any, phase: string) => {
  console.log('Document list changed for phase:', phase, evt)
  // This event should fire when items are moved between lists
}

const handleCreateButtonHover = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.interactive.primary
  target.style.color = theme.value.colors.text.inverse
}

const handleCreateButtonLeave = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.interactive.secondary
  target.style.color = theme.value.colors.interactive.primary
}

// Watch for project changes
watch(() => currentProject.value?.path, loadDocuments, { immediate: true })

// Watch for documents or board changes to update phase grouping
watch([documents, currentBoard], updateDocumentsByPhase, { immediate: true })

// Watch for changes in documentsByPhase to detect moves
watch(documentsByPhase, async (newPhases) => {
  if (!draggedDocument) return
  
  console.log('Documents by phase changed:', newPhases)
  
  // Make a copy of draggedDocument to avoid null reference issues
  const docToTransition = { ...draggedDocument }
  
  // Find which phase the dragged document is now in
  for (const [phaseKey, docs] of Object.entries(newPhases)) {
    const foundDoc = docs.find(doc => doc.short_code === docToTransition.short_code)
    if (foundDoc && foundDoc.phase !== phaseKey) {
      console.log(`Document ${docToTransition.short_code} moved from ${foundDoc.phase} to ${phaseKey}`)
      
      try {
        console.log(`Calling transitionPhase('${docToTransition.short_code}', '${phaseKey}')`)
        
        await transitionPhase(docToTransition.short_code, phaseKey)
        console.log(`Successfully transitioned ${docToTransition.short_code} to ${phaseKey}`)
        
        // Reload documents to reflect the change
        await loadDocuments()
      } catch (error) {
        console.error('Failed to transition document phase:', error)
        console.error('Document short_code:', docToTransition.short_code)
        console.error('Target phase:', phaseKey)
        console.error('Document info:', docToTransition)
        // Reload documents to revert the UI change
        await loadDocuments()
      }
      break
    }
  }
}, { deep: true })
</script>

<style scoped>
.scale-105 {
  transform: scale(1.05);
}
</style>