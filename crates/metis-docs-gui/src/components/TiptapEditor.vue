<template>
  <div class="tiptap-editor h-full flex flex-col">
    <!-- Toolbar - only show when editable -->
    <div v-if="editable" class="tiptap-toolbar border-b border-gray-200 p-2 flex gap-2 bg-gray-50">
      <button
        @click="editor?.chain().focus().toggleBold().run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('bold') 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        Bold
      </button>
      <button
        @click="editor?.chain().focus().toggleItalic().run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('italic') 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        Italic
      </button>
      <button
        @click="editor?.chain().focus().toggleHeading({ level: 1 }).run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('heading', { level: 1 }) 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        H1
      </button>
      <button
        @click="editor?.chain().focus().toggleHeading({ level: 2 }).run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('heading', { level: 2 }) 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        H2
      </button>
      <button
        @click="editor?.chain().focus().toggleBulletList().run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('bulletList') 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        Bullet List
      </button>
      <button
        @click="editor?.chain().focus().toggleOrderedList().run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('orderedList') 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        Numbered List
      </button>
      <button
        @click="editor?.chain().focus().toggleBlockquote().run()"
        :class="[
          'px-2 py-1 rounded text-sm transition-colors',
          editor?.isActive('blockquote') 
            ? 'bg-blue-600 text-white' 
            : 'bg-white text-gray-700 hover:bg-gray-100'
        ]"
      >
        Quote
      </button>
    </div>

    <!-- Editor Content -->
    <EditorContent 
      :editor="editor" 
      class="tiptap-content p-4 prose prose-sm max-w-none focus:outline-none flex-1 overflow-y-auto h-full"
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
    console.error('Error parsing frontmatter:', error)
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
          e.commands.setContent(processedContent)
        }
      }, 50)
    }
  },
  onUpdate: () => {
    // Get markdown content from the editor
    const markdownContent = getMarkdown()
    emit('update', markdownContent)
  },
})

// Watch for content changes (new document loaded)
watch(() => props.content, (newContent) => {
  if (newContent && editor.value) {
    const processedContent = processContent(newContent)
    if (processedContent !== editor.value.getHTML()) {
      editor.value.commands.setContent(processedContent)
    }
  } else if (newContent) {
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
  height: 100% !important;
  max-height: none !important;
  display: flex;
  flex-direction: column;
}

.tiptap-content :deep(.ProseMirror) {
  outline: none;
  height: 100% !important;
  flex: 1;
  min-height: 0;
  color: var(--color-text-primary);
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
</style>