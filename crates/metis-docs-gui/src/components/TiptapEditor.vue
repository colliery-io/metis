<template>
  <div class="tiptap-editor-container">
    <!-- Fixed Toolbar -->
    <TiptapToolbar 
      v-if="editable && editor" 
      :editor="editor" 
    />

    <!-- Scrollable Editor Content -->
    <div class="tiptap-content-wrapper">
      <EditorContent 
        :editor="editor" 
        class="tiptap-content prose prose-sm max-w-none focus:outline-none"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import { Markdown } from 'tiptap-markdown-3'
import TiptapToolbar from './TiptapToolbar.vue'
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
  editorProps: {
    scrollThreshold: 80,
    scrollMargin: 80,
    attributes: {
      class: 'prose prose-sm max-w-none focus:outline-none',
    },
  },
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
.tiptap-editor-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.tiptap-content-wrapper {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: 1rem;
}

.tiptap-content {
  width: 100%;
  height: auto;
}

.tiptap-content :deep(.ProseMirror) {
  outline: none;
  color: var(--color-text-primary);
  min-height: 300px;
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

/* Toolbar styles moved to TiptapToolbar.vue */
</style>