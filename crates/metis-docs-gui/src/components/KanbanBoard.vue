<template>
  <div class="kanban-board">
    <!-- Board Selection Header -->
    <div class="board-header">
      <div class="flex items-center justify-between">
        <h2>{{ currentBoardConfig?.title || 'Kanban Board' }}</h2>
        <button
          v-if="currentBoard !== 'vision'"
          @click="showCreateDialog = true"
          class="board-tab create-button"
          style="background-color: #10b981; color: white; border: 1px solid #10b981;"
        >
          + Create {{ getDocumentTypeLabel(currentBoard) }}
        </button>
      </div>
      <div class="board-tabs">
        <button
          v-for="board in availableBoards"
          :key="board"
          :class="['board-tab', { active: currentBoard === board }]"
          @click="switchBoard(board)"
        >
          {{ getBoardTitle(board) }} ({{ getDocumentCount(board) }})
        </button>
      </div>
    </div>

    <!-- Vision Board - Single Document Editor -->
    <div v-if="currentBoard === 'vision'" class="vision-container flex flex-col">
      <VisionDisplay
        v-if="visionDocument"
        :vision="visionDocument"
        @document-updated="handleDocumentUpdated"
      />
      <div v-else class="text-center py-8" style="color: var(--color-text-secondary)">
        No vision document found. Create one first.
      </div>
    </div>

    <!-- Other Boards - Kanban Columns -->
    <div v-else class="columns-container">
      <KanbanColumn
        v-for="phase in currentBoardConfig?.phases || []"
        :key="phase.key"
        :title="phase.title"
        :phase-key="phase.key"
        :documents="documentsByPhase[phase.key] || []"
        :board-type="currentBoard"
        @documents-changed="handleDocumentsChanged"
        @promote="handlePromoteToTaskBoard"
        @view="handleViewDocument"
        @archive="handleArchiveDocument"
      />
    </div>

    <!-- Create Document Dialog -->
    <CreateDocumentDialog
      :isOpen="showCreateDialog"
      :boardType="currentBoard"
      @close="showCreateDialog = false"
      @document-created="handleDocumentCreated"
    />

    <!-- Document Viewer Modal -->
    <DocumentViewer
      :isOpen="showDocumentViewer"
      :document="selectedDocument"
      @close="handleCloseDocumentViewer"
      @document-updated="handleDocumentViewerUpdated"
    />

    <!-- Archive Confirmation Modal -->
    <div
      v-if="showArchiveConfirmation"
      class="modal-overlay"
      @click="cancelArchive"
    >
      <div
        class="archive-modal"
        @click.stop
      >
        <div class="modal-header">
          <h3>Archive Document</h3>
        </div>
        
        <div class="modal-content">
          <p class="archive-warning">
            Are you sure you want to archive "<strong>{{ documentToArchive?.title }}</strong>"?
          </p>
          <p class="archive-details">
            This will move the document and all its children to the archived folder. 
            This action cannot be undone.
          </p>
        </div>
        
        <div class="modal-actions">
          <button
            @click="cancelArchive"
            class="cancel-button"
          >
            Cancel
          </button>
          <button
            @click="confirmArchive"
            class="archive-button"
          >
            Archive Document
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import type { DocumentInfo } from '../lib/tauri-api'
import { listDocuments, transitionPhase, archiveDocument } from '../lib/tauri-api'
import { useProject } from '../composables/useProject'
import { getBoardConfig, getDocumentsByPhase } from '../lib/board-config'
import type { BoardType } from '../types/board'
import KanbanColumn from './KanbanColumn.vue'
import VisionDisplay from './VisionDisplay.vue'
import CreateDocumentDialog from './CreateDocumentDialog.vue'
import DocumentViewer from './DocumentViewer.vue'

interface Props {
  onBackToProjects: () => void
}

defineProps<Props>()

const { currentProject } = useProject()

// Multi-board support for flight levels
const availableBoards: BoardType[] = ['vision', 'initiative', 'task', 'adr', 'backlog']
const currentBoard = ref<BoardType>('vision')
const allDocuments = ref<DocumentInfo[]>([])
const showCreateDialog = ref(false)

// Document viewer modal state
const showDocumentViewer = ref(false)
const selectedDocument = ref<DocumentInfo | null>(null)

// Archive confirmation modal state
const showArchiveConfirmation = ref(false)
const documentToArchive = ref<DocumentInfo | null>(null)

// Board configuration
const currentBoardConfig = computed(() => getBoardConfig(currentBoard.value))

// Documents organized by phase for current board
const documentsByPhase = ref<Record<string, DocumentInfo[]>>({})

// Get the vision document (should be only one)
const visionDocument = computed(() => {
  return allDocuments.value.find(doc => doc.document_type === 'vision')
})

// Update documents by phase when board changes or documents load
const updateDocumentsByPhase = () => {
  documentsByPhase.value = getDocumentsByPhase(allDocuments.value, currentBoard.value)
}

// Load documents from backend
const loadDocuments = async () => {
  if (!currentProject.value) return
  
  try {
    allDocuments.value = await listDocuments()
    updateDocumentsByPhase()
  } catch (error) {
    // Failed to load documents
  }
}

// Board switching and utilities
const switchBoard = (board: BoardType) => {
  currentBoard.value = board
  updateDocumentsByPhase()
}

const getBoardTitle = (board: BoardType) => {
  const config = getBoardConfig(board)
  return config?.title || board
}

const getDocumentCount = (board: BoardType) => {
  const config = getBoardConfig(board)
  if (!config) return 0
  
  return allDocuments.value.filter(config.documentFilter).length
}

// Handle when documents change in columns
const handleDocumentsChanged = async (phaseKey: string, newDocs: DocumentInfo[]) => {
  
  // Update the documents for this phase immediately for responsiveness
  documentsByPhase.value = {
    ...documentsByPhase.value,
    [phaseKey]: newDocs
  }
  
  // Reload all documents from backend to ensure consistency after phase transitions
  setTimeout(async () => {
    await loadDocuments()
  }, 100) // Small delay to let backend transition complete
}

// Handle when vision document is updated
const handleDocumentUpdated = async () => {
  await loadDocuments()
}

// Handle when a new document is created
const handleDocumentCreated = async () => {
  await loadDocuments()
}

// Handle promoting a backlog item to the task board
const handlePromoteToTaskBoard = async (document: DocumentInfo) => {
  
  try {
    // Transition the phase from 'backlog' to 'todo'
    // This will move it from backlog board to task board
    await transitionPhase(document.short_code, 'todo')
    
    // Reload documents to reflect the change
    await loadDocuments()
    
    // Switch to task board to show the promoted item
    switchBoard('task')
    
  } catch (error) {
    // Failed to promote document
    // TODO: Show user-friendly error message
  }
}

// Handle viewing a document
const handleViewDocument = (document: DocumentInfo) => {
  selectedDocument.value = document
  showDocumentViewer.value = true
}

// Handle closing document viewer
const handleCloseDocumentViewer = () => {
  showDocumentViewer.value = false
  selectedDocument.value = null
}

// Handle when document is updated in viewer
const handleDocumentViewerUpdated = async () => {
  await loadDocuments()
}

// Handle archiving a document
const handleArchiveDocument = (document: DocumentInfo) => {
  documentToArchive.value = document
  showArchiveConfirmation.value = true
}

// Confirm and execute archive
const confirmArchive = async () => {
  if (!documentToArchive.value) return
  
  const document = documentToArchive.value
  
  // Close modal first
  showArchiveConfirmation.value = false
  documentToArchive.value = null
  
  try {
    await archiveDocument(document.short_code)
    
    
    // Reload documents to reflect the change
    await loadDocuments()
    
  } catch (error) {
    // Failed to archive document
    alert(`Failed to archive document: ${error}`)
  }
}

// Cancel archive
const cancelArchive = () => {
  showArchiveConfirmation.value = false
  documentToArchive.value = null
}

// Get document type label for create button
const getDocumentTypeLabel = (boardType: BoardType) => {
  switch (boardType) {
    case 'initiative':
      return 'Initiative'
    case 'task':
      return 'Task'
    case 'adr':
      return 'ADR'
    case 'backlog':
      return 'Backlog Item'
    case 'strategy':
      return 'Strategy'
    default:
      return 'Document'
  }
}

onMounted(() => {
  loadDocuments()
})
</script>

<style scoped>
.kanban-board {
  padding: 24px;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-primary);
}

.board-header {
  margin-bottom: 24px;
}

.board-header h2 {
  color: var(--color-text-primary);
  font-size: 24px;
  font-weight: 600;
  margin: 0 0 16px 0;
}

.board-tabs {
  display: flex;
  gap: 8px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.board-tab {
  padding: 10px 16px;
  border: 1px solid var(--color-border-primary);
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.board-tab:hover {
  background-color: var(--color-background-elevated);
  border-color: var(--color-interactive-primary);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.board-tab.active {
  background-color: var(--color-interactive-primary);
  color: var(--color-text-inverse);
  border-color: var(--color-interactive-primary);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.board-tab.active:hover {
  transform: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.columns-container {
  display: flex;
  gap: 20px;
  height: calc(100vh - 180px);
  overflow-x: auto;
  padding-bottom: 8px;
}

.vision-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.create-button {
  background-color: var(--color-status-completed) !important;
  color: var(--color-text-inverse) !important;
  border: 1px solid var(--color-status-completed) !important;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.create-button:hover {
  background-color: var(--color-status-active) !important;
  border-color: var(--color-status-active) !important;
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

/* Archive Confirmation Modal */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.archive-modal {
  background-color: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 12px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
  width: max-content;
  min-width: 420px;
  max-width: min(500px, 90vw);
}

.modal-header {
  padding: 20px 24px 16px 24px;
  border-bottom: 1px solid var(--color-border-primary);
}

.modal-header h3 {
  color: var(--color-text-primary);
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.modal-content {
  padding: 20px 24px;
}

.archive-warning {
  color: var(--color-text-primary);
  font-size: 16px;
  font-weight: 500;
  margin: 0 0 12px 0;
}

.archive-details {
  color: var(--color-text-secondary);
  font-size: 14px;
  line-height: 1.5;
  margin: 0;
}

.modal-actions {
  padding: 16px 24px 20px 24px;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.cancel-button {
  padding: 10px 20px;
  background-color: var(--color-background-secondary);
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.cancel-button:hover {
  background-color: var(--color-background-elevated);
  border-color: var(--color-interactive-primary);
}

.archive-button {
  padding: 10px 20px;
  background-color: var(--color-interactive-danger);
  border: 1px solid var(--color-interactive-danger);
  border-radius: 6px;
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.archive-button:hover {
  background-color: var(--color-status-active);
  border-color: var(--color-status-active);
}
</style>