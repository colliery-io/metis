<template>
  <div class="embedded-document">
    <div v-if="loading" class="text-center py-8" :style="{ color: theme.colors.text.secondary }">
      Loading document...
    </div>
    <div v-else-if="error" class="text-center py-8" :style="{ color: theme.colors.border.error }">
      Error loading document: {{ error }}
    </div>
    <div v-else class="h-full">
      <TiptapEditor 
        :content="content"
        :editable="false"
        class="h-full"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { readDocument } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'
import TiptapEditor from './TiptapEditor.vue'
import type { DocumentInfo } from '../lib/tauri-api'

interface Props {
  document: DocumentInfo
}

const props = defineProps<Props>()
const { theme } = useTheme()

const content = ref('')
const loading = ref(true)
const error = ref<string | null>(null)

const loadContent = async () => {
  try {
    loading.value = true
    error.value = null
    
    const docContent = await readDocument(props.document.short_code)
    // Pass raw markdown content to TiptapEditor for proper rendering
    content.value = docContent.content
    
  } catch (err) {
    // Failed to load document content
    error.value = err instanceof Error ? err.message : 'Failed to load document'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadContent()
})
</script>

<style scoped>
.prose {
  line-height: 1.75;
}
</style>