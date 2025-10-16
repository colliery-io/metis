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
    
    // State
    isActive,
    canUndo,
    canRedo,
  }
}