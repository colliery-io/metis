<template>
  <div class="vision-display mb-8 flex flex-col h-full">
    <!-- Vision Header -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <h1 
          class="text-3xl font-bold"
          :style="{ color: theme.colors.text.primary }"
        >
          {{ vision.title }}
        </h1>
        <div class="flex items-center gap-3">
          <button
            @click="toggleEditMode"
            class="px-4 py-2 rounded-lg font-medium transition-all hover:shadow-md"
            :style="{
              backgroundColor: isEditing ? theme.colors.interactive.danger : theme.colors.interactive.primary,
              color: theme.colors.text.inverse,
            }"
          >
            {{ isEditing ? 'Cancel' : 'Edit' }}
          </button>
          <button
            v-if="isEditing"
            @click="handleSave"
            class="px-4 py-2 rounded-lg font-medium transition-all hover:shadow-md"
            :style="{
              backgroundColor: theme.colors.status.completed,
              color: theme.colors.text.inverse,
              opacity: saving ? 0.6 : 1
            }"
            :disabled="saving"
          >
            {{ saving ? 'Saving...' : 'Save' }}
          </button>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium"
            :style="{
              backgroundColor: theme.colors.status.completed + '20',
              color: theme.colors.status.completed,
            }"
          >
            {{ vision.short_code }}
          </span>
        </div>
      </div>
      <div 
        class="text-sm"
        :style="{ color: theme.colors.text.secondary }"
      >
        Published {{ formatDate(vision.updated_at) }}
      </div>
    </div>

    <!-- Vision Content -->
    <div 
      class="rounded-lg border flex-1 flex flex-col"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        borderColor: theme.colors.border.primary,
        height: 'calc(100vh - 300px)',
      }"
    >
      <div v-if="loading" class="text-center py-8" :style="{ color: theme.colors.text.secondary }">
        Loading document...
      </div>
      <div v-else-if="error" class="text-center py-8" :style="{ color: theme.colors.border.error }">
        Error: {{ error }}
      </div>
      <div v-else class="flex-1 flex flex-col">
        <TiptapEditor 
          :content="content"
          :editable="isEditing"
          @update:content="content = $event"
          class="flex-1"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { readDocument, updateDocument } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'
import TiptapEditor from './TiptapEditor.vue'
import type { DocumentInfo } from '../lib/tauri-api'

interface Props {
  vision: DocumentInfo
}

interface Emits {
  (e: 'document-updated'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { theme } = useTheme()

const content = ref('')
const loading = ref(true)
const error = ref<string | null>(null)
const isEditing = ref(false)
const saving = ref(false)
const originalContent = ref('')

const formatDate = (timestamp: number) => {
  try {
    return new Date(timestamp * 1000).toLocaleDateString()
  } catch {
    return 'Invalid date'
  }
}

const loadContent = async () => {
  try {
    loading.value = true
    error.value = null
    
    const docContent = await readDocument(props.vision.short_code)
    content.value = docContent.content
    originalContent.value = docContent.content
    
  } catch (err) {
    console.error('Failed to load document content:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load document'
  } finally {
    loading.value = false
  }
}

const toggleEditMode = () => {
  if (isEditing.value) {
    // Cancel editing - restore original content
    content.value = originalContent.value
    isEditing.value = false
  } else {
    // Start editing
    isEditing.value = true
  }
}

const handleSave = async () => {
  try {
    saving.value = true
    console.log('Saving document:', props.vision.short_code, 'with content length:', content.value.length)
    await updateDocument(props.vision.short_code, content.value)
    console.log('Document saved successfully')
    originalContent.value = content.value
    isEditing.value = false
    emit('document-updated')
  } catch (err) {
    console.error('Failed to save document:', err)
    console.error('Error details:', {
      shortCode: props.vision.short_code,
      contentLength: content.value.length,
      error: err
    })
    error.value = err instanceof Error ? err.message : 'Failed to save document'
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadContent()
})
</script>