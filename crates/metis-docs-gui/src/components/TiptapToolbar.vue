<template>
  <div 
    class="editor-toolbar" 
    :style="{ 
      display: 'flex', 
      alignItems: 'center', 
      gap: 'var(--spacing-xs)', 
      padding: 'var(--spacing-sm) 1rem', 
      backgroundColor: '#ffffff', 
      borderBottom: '1px solid var(--color-border)', 
      flexWrap: 'wrap',
      position: 'sticky',
      top: 0,
      zIndex: 10,
      flexShrink: 0
    }"
  >
    <!-- Heading buttons -->
    <button
      @click="toggleHeading(1)"
      :class="{ 'btn-toolbar--active': isActive('heading', { level: 1 }).value }"
      class="btn-toolbar"
      title="Heading 1"
    >
      H1
    </button>
    <button
      @click="toggleHeading(2)"
      :class="{ 'btn-toolbar--active': isActive('heading', { level: 2 }).value }"
      class="btn-toolbar"
      title="Heading 2"
    >
      H2
    </button>
    <button
      @click="toggleHeading(3)"
      :class="{ 'btn-toolbar--active': isActive('heading', { level: 3 }).value }"
      class="btn-toolbar"
      title="Heading 3"
    >
      H3
    </button>
    
    <div class="btn-toolbar-divider"></div>
    
    <!-- Text formatting -->
    <button
      @click="toggleBold"
      :class="{ 'btn-toolbar--active': isActive('bold').value }"
      class="btn-toolbar"
      title="Bold (Ctrl+B)"
    >
      <strong>B</strong>
    </button>
    <button
      @click="toggleItalic"
      :class="{ 'btn-toolbar--active': isActive('italic').value }"
      class="btn-toolbar"
      title="Italic (Ctrl+I)"
    >
      <em>I</em>
    </button>
    <button
      @click="toggleStrike"
      :class="{ 'btn-toolbar--active': isActive('strike').value }"
      class="btn-toolbar"
      title="Strikethrough"
    >
      <span style="text-decoration: line-through;">S</span>
    </button>
    
    <div class="btn-toolbar-divider"></div>
    
    <!-- Lists and blocks -->
    <button
      @click="toggleBulletList"
      :class="{ 'btn-toolbar--active': isActive('bulletList').value }"
      class="btn-toolbar"
      title="Bullet List"
    >
      • List
    </button>
    <button
      @click="toggleOrderedList"
      :class="{ 'btn-toolbar--active': isActive('orderedList').value }"
      class="btn-toolbar"
      title="Numbered List"
    >
      1. List
    </button>
    <button
      @click="toggleBlockquote"
      :class="{ 'btn-toolbar--active': isActive('blockquote').value }"
      class="btn-toolbar"
      title="Quote"
    >
      " Quote
    </button>
    
    <div class="btn-toolbar-divider"></div>
    
    <!-- Table commands -->
    <button
      @click="insertTable"
      class="btn-toolbar"
      title="Insert Table"
    >
      + Table
    </button>
    <button
      @click="deleteTable"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Delete Table"
    >
      - Table
    </button>
    <button
      @click="addColumnAfter"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Add Column"
    >
      + Col
    </button>
    <button
      @click="deleteColumn"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Delete Column"
    >
      - Col
    </button>
    <button
      @click="addRowAfter"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Add Row"
    >
      + Row
    </button>
    <button
      @click="deleteRow"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Delete Row"
    >
      - Row
    </button>
    <button
      @click="toggleHeaderRow"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Toggle Header Row"
    >
      H Row
    </button>
    <button
      @click="toggleHeaderColumn"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Toggle Header Column"
    >
      H Col
    </button>
    <button
      @click="toggleHeaderCell"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Toggle Header Cell"
    >
      H Cell
    </button>
    <button
      @click="mergeOrSplit"
      :disabled="!isActive('table').value"
      class="btn-toolbar"
      title="Merge or Split Cells"
    >
      Merge
    </button>
    
    <div class="btn-toolbar-divider"></div>
    
    <!-- Other commands -->
    <button
      @click="setHorizontalRule"
      class="btn-toolbar"
      title="Horizontal Rule"
    >
      — Rule
    </button>
    <button
      @click="undo"
      :disabled="!canUndo"
      class="btn-toolbar"
      title="Undo (Ctrl+Z)"
    >
      ↶ Undo
    </button>
    <button
      @click="redo"
      :disabled="!canRedo"
      class="btn-toolbar"
      title="Redo (Ctrl+Y)"
    >
      ↷ Redo
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useToolbar } from '../composables/useToolbar.ts'

interface Props {
  editor: any
}

const props = defineProps<Props>()

const {
  toggleBold,
  toggleItalic,
  toggleStrike,
  toggleHeading,
  toggleBulletList,
  toggleOrderedList,
  toggleBlockquote,
  setHorizontalRule,
  undo,
  redo,
  insertTable,
  deleteTable,
  addColumnAfter,
  deleteColumn,
  addRowAfter,
  deleteRow,
  toggleHeaderRow,
  toggleHeaderColumn,
  toggleHeaderCell,
  mergeOrSplit,
  isActive,
  canUndo,
  canRedo,
} = useToolbar(ref(props.editor))
</script>

<style scoped>
/* Toolbar button styles */
.btn-toolbar {
  padding: 8px 12px;
  border: 1px solid var(--color-border-primary);
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 40px;
  height: 36px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.btn-toolbar:hover {
  background-color: var(--color-background-elevated);
  border-color: var(--color-interactive-primary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

.btn-toolbar:active {
  transform: translateY(0);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.btn-toolbar:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background-color: var(--color-background-tertiary);
  transform: none;
  box-shadow: none;
}

.btn-toolbar:disabled:hover {
  transform: none;
  box-shadow: none;
  border-color: var(--color-border-primary);
}

.btn-toolbar--active {
  background-color: var(--color-interactive-primary);
  color: var(--color-text-inverse);
  border-color: var(--color-interactive-primary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
}

.btn-toolbar--active:hover {
  background-color: var(--color-interactive-primary);
  transform: none;
}

.btn-toolbar-divider {
  width: 1px;
  height: 28px;
  background-color: var(--color-border-primary);
  margin: 0 8px;
  opacity: 0.6;
}
</style>