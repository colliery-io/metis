<template>
  <div class="kanban-column">
    <div class="column-header">
      <h3 class="column-title">{{ title }}</h3>
      <span class="document-count">{{ documents.length }}</span>
    </div>
    
    <Container
      group-name="documents"
      orientation="vertical"
      :get-child-payload="getChildPayload"
      @drop="handleDrop"
      drag-class="cursor-grabbing"
      drag-handle-selector=".kanbancard-drag"
      class="column-container"
    >
      <Draggable
        v-for="doc in documents"
        :key="doc.short_code"
      >
        <KanbanCard
          :document="doc"
          :dragging-enabled="true"
          :board-type="boardType"
          @promote="handlePromote"
          @view="handleView"
          @archive="handleArchive"
        />
      </Draggable>
    </Container>
  </div>
</template>

<script setup lang="ts">
import type { DocumentInfo } from '../lib/tauri-api'
import { transitionPhase } from '../lib/tauri-api'
import KanbanCard from './KanbanCard.vue'
// @ts-ignore
import { Container, Draggable } from 'vue3-smooth-dnd'
import { applyDrag } from '../utils/drag-n-drop'

interface Props {
  title: string
  documents: DocumentInfo[]
  phaseKey: string
  boardType?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'documents-changed': [phaseKey: string, newDocs: DocumentInfo[]]
  'promote': [document: DocumentInfo]
  'view': [document: DocumentInfo]
  'archive': [document: DocumentInfo]
}>()

const handlePromote = (document: DocumentInfo) => {
  emit('promote', document)
}

const handleView = (document: DocumentInfo) => {
  emit('view', document)
}

const handleArchive = (document: DocumentInfo) => {
  emit('archive', document)
}

// Debug: Log documents received

// Individual payload function for this column
const getChildPayload = (index: number) => {
  const doc = props.documents[index]
  return {
    ...doc,
    phase: props.phaseKey
  }
}

// Individual drop handler for this column
const handleDrop = async (dropResult: any) => {
  
  if (!dropResult || (dropResult.removedIndex === null && dropResult.addedIndex === null)) {
    return
  }
  
  // Apply the drag operation
  const newDocs = applyDrag(props.documents, dropResult)
  
  // Update documents with new phase
  const updatedDocs = newDocs.map(doc => ({
    ...doc,
    phase: props.phaseKey
  }))
  
  
  // Check if this was a cross-phase move (document added from another column)
  const { payload, removedIndex, addedIndex } = dropResult
  if (payload && removedIndex === null && addedIndex !== null) {
    // This is a cross-phase move - call backend to make it persistent
    try {
      
      const result = await transitionPhase(payload.short_code, props.phaseKey)
      
      // Emit the change to parent to reload from backend
      emit('documents-changed', props.phaseKey, updatedDocs)
    } catch (error) {
      // Backend transition failed
      
      // Still emit the change to parent for UI update, but backend may be inconsistent
      emit('documents-changed', props.phaseKey, updatedDocs)
    }
  } else {
    // Just a reorder within the same column - no backend call needed
    emit('documents-changed', props.phaseKey, updatedDocs)
  }
}
</script>

<style scoped>
.kanban-column {
  width: 320px;
  border: 1px solid var(--color-border-primary);
  padding: 20px;
  background-color: var(--color-background-secondary);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: box-shadow 0.2s ease;
  display: flex;
  flex-direction: column;
  height: 100%;
  max-height: calc(100vh - 200px);
}

.kanban-column:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 12px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--color-border-primary);
  flex-shrink: 0;
}

.column-title {
  color: var(--color-text-primary);
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.document-count {
  background-color: var(--color-background-elevated);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  padding: 4px 8px;
  border-radius: 12px;
  border: 1px solid var(--color-border-primary);
  min-width: 24px;
  text-align: center;
}

.column-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 4px 8px 0;
  margin-right: -4px;
  min-height: 200px;
}

/* Custom scrollbar styling */
.column-container::-webkit-scrollbar {
  width: 6px;
}

.column-container::-webkit-scrollbar-track {
  background: var(--color-background-primary);
  border-radius: 3px;
}

.column-container::-webkit-scrollbar-thumb {
  background: var(--color-border-primary);
  border-radius: 3px;
}

.column-container::-webkit-scrollbar-thumb:hover {
  background: var(--color-interactive-primary);
}

/* Smooth DnD styling */
.column-container :deep(.smooth-dnd-container) {
  min-height: 100%;
}

.column-container :deep(.smooth-dnd-draggable-wrapper) {
  margin-bottom: 12px;
}

.column-container :deep(.smooth-dnd-ghost) {
  opacity: 0.5;
  transform: rotate(2deg);
}

.column-container :deep(.smooth-dnd-drop-preview) {
  background-color: var(--color-interactive-primary);
  opacity: 0.1;
  border: 2px dashed var(--color-interactive-primary);
  border-radius: 8px;
  margin: 8px 0;
  height: 80px;
}
</style>