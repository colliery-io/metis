<template>
  <div class="tiptap-editor h-full flex flex-col">
    <!-- Toolbar - only show when editable -->
    <div v-if="editable" class="editor-toolbar" style="display: flex; align-items: center; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-lg); background-color: var(--color-surface); border-bottom: 1px solid var(--color-border); flex-wrap: wrap;">
      <button
        @click="editor?.chain().focus().toggleHeading({ level: 1 }).run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 1 }) }"
        class="btn-toolbar"
      >
        H1
      </button>
      <button
        @click="editor?.chain().focus().toggleHeading({ level: 2 }).run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 2 }) }"
        class="btn-toolbar"
      >
        H2
      </button>
      <button
        @click="editor?.chain().focus().toggleHeading({ level: 3 }).run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('heading', { level: 3 }) }"
        class="btn-toolbar"
      >
        H3
      </button>
      <div class="btn-toolbar-divider"></div>
      <button
        @click="editor?.chain().focus().toggleBold().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('bold') }"
        class="btn-toolbar"
      >
        <strong>B</strong>
      </button>
      <button
        @click="editor?.chain().focus().toggleItalic().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('italic') }"
        class="btn-toolbar"
      >
        <em>I</em>
      </button>
      <button
        @click="editor?.chain().focus().toggleStrike().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('strike') }"
        class="btn-toolbar"
      >
        <strike>S</strike>
      </button>
      <div class="btn-toolbar-divider"></div>
      <button
        @click="editor?.chain().focus().toggleBulletList().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('bulletList') }"
        class="btn-toolbar"
      >
        • List
      </button>
      <button
        @click="editor?.chain().focus().toggleOrderedList().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('orderedList') }"
        class="btn-toolbar"
      >
        1. List
      </button>
      <button
        @click="editor?.chain().focus().toggleBlockquote().run()"
        :class="{ 'btn-toolbar--active': editor?.isActive('blockquote') }"
        class="btn-toolbar"
      >
        " Quote
      </button>
      <div class="btn-toolbar-divider"></div>
      <button
        @click="editor?.chain().focus().setHorizontalRule().run()"
        class="btn-toolbar"
      >
        — Rule
      </button>
      <button
        @click="editor?.chain().focus().undo().run()"
        :disabled="!editor?.can().undo()"
        class="btn-toolbar"
      >
        ↶ Undo
      </button>
      <button
        @click="editor?.chain().focus().redo().run()"
        :disabled="!editor?.can().redo()"
        class="btn-toolbar"
      >
        ↷ Redo
      </button>
    </div>

    <!-- Editor Content -->
    <EditorContent 
      :editor="editor" 
      class="tiptap-content p-4 prose prose-sm max-w-none focus:outline-none flex-1"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import { Markdown } from 'tiptap-markdown-3'
// Removed gray-matter to avoid Node.js Buffer dependency in browser

interface Props {
  content: string
  placeholder?: string
  editable?: boolean
}

interface Emits {
  (e: 'update', content: string): void
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Start writing...',
  editable: true,
})

const emit = defineEmits<Emits>()

const pendingContent = ref<string | null>(null)
const isUpdatingContent = ref(false)
const savedCursorPosition = ref(0)
const lastEmittedContent = ref('')

// Process content to separate frontmatter from markdown
const processContent = (content: string): string => {
  try {
    // Manual frontmatter removal (browser-safe, no Node.js dependencies)
    const lines = content.split('\n')
    if (lines[0] === '---') {
      const endIndex = lines.findIndex((line, index) => index > 0 && line === '---')
      if (endIndex > 0) {
        return lines.slice(endIndex + 1).join('\n').trim()
      }
    }
    return content.trim()
  } catch (error) {
    // Error parsing frontmatter
    return content.trim()
  }
}

const getMarkdown = (): string => {
  if (!editor.value) return ''
  const storage = editor.value.storage as any
  return storage.markdown?.getMarkdown() || ''
}

const editor = useEditor({
  content: '',
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3, 4, 5, 6] }
    }),
    Placeholder.configure({
      placeholder: props.placeholder,
    }),
    Markdown.configure({
      html: true,
      tightLists: true,
      bulletListMarker: '-'
    })
  ],
  onCreate: ({ editor: e }) => {
    // Load content when editor is ready
    if (props.content) {
      setTimeout(() => {
        const processedContent = processContent(props.content)
        if (e && processedContent) {
          isUpdatingContent.value = true
          e.commands.setContent(processedContent)
          setTimeout(() => {
            isUpdatingContent.value = false
          }, 100)
        }
      }, 50)
    }
  },
  onUpdate: () => {
    // Don't emit updates when we're programmatically setting content
    if (isUpdatingContent.value) {
      return
    }
    
    
    // Get markdown content from the editor
    const markdownContent = getMarkdown()
    lastEmittedContent.value = markdownContent
    emit('update', markdownContent)
  },
})

// Watch for content changes (new document loaded)
watch(() => props.content, (newContent, oldContent) => {
  
  // Don't update if this content change is from our own editor emission
  if (newContent === lastEmittedContent.value) {
    return
  }
  
  if (newContent && editor.value && newContent !== oldContent) {
    const processedContent = processContent(newContent)
    
    // Save current cursor position before updating content
    savedCursorPosition.value = editor.value.state.selection.from
    
    isUpdatingContent.value = true
    editor.value.commands.setContent(processedContent)
    
    // Restore cursor position after content is set
    setTimeout(() => {
      if (editor.value) {
        const maxPos = editor.value.state.doc.content.size
        const safePosition = Math.min(savedCursorPosition.value, maxPos)
        editor.value.commands.setTextSelection(safePosition)
      }
      isUpdatingContent.value = false
    }, 100)
  } else if (newContent && !editor.value) {
    pendingContent.value = newContent
  }
}, { immediate: false })

// Watch for editable changes
watch(() => props.editable, (newEditable) => {
  if (editor.value) {
    editor.value.setEditable(newEditable)
  }
}, { immediate: true })

onBeforeUnmount(() => {
  if (editor.value) {
    editor.value.destroy()
  }
})
</script>

<style scoped>
.tiptap-content {
  display: flex;
  flex-direction: column;
  overflow: visible !important;
  max-height: none !important;
  height: auto !important;
}

.tiptap-content :deep(.ProseMirror) {
  outline: none;
  color: var(--color-text-primary);
  overflow: visible !important;
  max-height: none !important;
  height: auto !important;
}

.tiptap-content :deep(.ProseMirror p.is-editor-empty:first-child::before) {
  color: var(--color-text-tertiary);
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}

.tiptap-content :deep(.ProseMirror h1),
.tiptap-content :deep(.ProseMirror h2),
.tiptap-content :deep(.ProseMirror h3),
.tiptap-content :deep(.ProseMirror h4),
.tiptap-content :deep(.ProseMirror h5),
.tiptap-content :deep(.ProseMirror h6) {
  color: var(--color-text-primary);
}

.tiptap-content :deep(.ProseMirror p),
.tiptap-content :deep(.ProseMirror li),
.tiptap-content :deep(.ProseMirror td),
.tiptap-content :deep(.ProseMirror th) {
  color: var(--color-text-primary);
}

.tiptap-content :deep(.ProseMirror blockquote) {
  color: var(--color-text-secondary);
  border-left-color: var(--color-border-primary);
}

.tiptap-content :deep(.ProseMirror strong) {
  color: var(--color-text-primary);
}

.tiptap-content :deep(.ProseMirror em) {
  color: var(--color-text-primary);
}

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