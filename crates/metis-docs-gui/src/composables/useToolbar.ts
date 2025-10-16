import { computed, type Ref, type ComputedRef } from 'vue'
import type { Editor } from '@tiptap/vue-3'
import type { Level } from '@tiptap/extension-heading'

export function useToolbar(editor: Ref<Editor | null>) {
  // Basic formatting commands
  const toggleBold = () => editor.value?.chain().focus().toggleBold().run()
  const toggleItalic = () => editor.value?.chain().focus().toggleItalic().run()
  const toggleStrike = () => editor.value?.chain().focus().toggleStrike().run()
  
  // Heading commands
  const toggleHeading = (level: Level) => editor.value?.chain().focus().toggleHeading({ level }).run()
  
  // List commands
  const toggleBulletList = () => editor.value?.chain().focus().toggleBulletList().run()
  const toggleOrderedList = () => editor.value?.chain().focus().toggleOrderedList().run()
  const toggleBlockquote = () => editor.value?.chain().focus().toggleBlockquote().run()
  
  // Other commands
  const setHorizontalRule = () => editor.value?.chain().focus().setHorizontalRule().run()
  const undo = () => editor.value?.chain().focus().undo().run()
  const redo = () => editor.value?.chain().focus().redo().run()
  
  // Table commands
  const insertTable = () => editor.value?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
  const deleteTable = () => editor.value?.chain().focus().deleteTable().run()
  const addColumnBefore = () => editor.value?.chain().focus().addColumnBefore().run()
  const addColumnAfter = () => editor.value?.chain().focus().addColumnAfter().run()
  const deleteColumn = () => editor.value?.chain().focus().deleteColumn().run()
  const addRowBefore = () => editor.value?.chain().focus().addRowBefore().run()
  const addRowAfter = () => editor.value?.chain().focus().addRowAfter().run()
  const deleteRow = () => editor.value?.chain().focus().deleteRow().run()
  const toggleHeaderColumn = () => editor.value?.chain().focus().toggleHeaderColumn().run()
  const toggleHeaderRow = () => editor.value?.chain().focus().toggleHeaderRow().run()
  const toggleHeaderCell = () => editor.value?.chain().focus().toggleHeaderCell().run()
  const mergeCells = () => editor.value?.chain().focus().mergeCells().run()
  const splitCell = () => editor.value?.chain().focus().splitCell().run()
  const mergeOrSplit = () => editor.value?.chain().focus().mergeOrSplit().run()
  const setCellAttribute = (name: string, value: any) => editor.value?.chain().focus().setCellAttribute(name, value).run()
  
  // Active state helpers
  const isActive = (name: string, attrs: Record<string, any> = {}): ComputedRef<boolean> => 
    computed(() => editor.value?.isActive(name, attrs) ?? false)
  
  // Can execute helpers
  const canUndo = computed(() => editor.value?.can().undo() ?? false)
  const canRedo = computed(() => editor.value?.can().redo() ?? false)
  
  return {
    // Commands
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
    
    // Table commands
    insertTable,
    deleteTable,
    addColumnBefore,
    addColumnAfter,
    deleteColumn,
    addRowBefore,
    addRowAfter,
    deleteRow,
    toggleHeaderColumn,
    toggleHeaderRow,
    toggleHeaderCell,
    mergeCells,
    splitCell,
    mergeOrSplit,
    setCellAttribute,
    
    // State
    isActive,
    canUndo,
    canRedo,
  }
}