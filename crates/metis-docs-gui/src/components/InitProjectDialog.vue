<template>
  <div 
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="position: fixed; top: 0; left: 0; right: 0; bottom: 0"
  >
    <!-- Backdrop -->
    <div 
      class="absolute inset-0 transition-opacity"
      :style="{ backgroundColor: theme.colors.background.overlay }"
      @click="emit('cancel')"
    />
    
    <!-- Dialog -->
    <div 
      class="relative rounded-lg shadow-lg p-6 z-10"
      :style="{
        backgroundColor: theme.colors.background.elevated,
        border: `1px solid ${theme.colors.border.primary}`,
        width: '400px',
        maxWidth: '90vw'
      }"
    >
      <form @submit="handleSubmit">
        <!-- Title -->
        <h3 
          class="font-semibold mb-4"
          :style="{ 
            fontSize: '18px', 
            color: theme.colors.text.primary 
          }"
        >
          Initialize Metis Project
        </h3>

        <!-- Message -->
        <p 
          class="mb-4 leading-relaxed"
          :style="{ 
            fontSize: '14px', 
            color: theme.colors.text.secondary 
          }"
        >
          The directory "{{ directoryName }}" will be initialized as a new Metis project.
        </p>

        <!-- Configuration Preset -->
        <div class="mb-4">
          <label 
            class="block mb-2 font-medium"
            :style="{ 
              fontSize: '14px', 
              color: theme.colors.text.primary 
            }"
          >
            Configuration Preset
          </label>
          <select
            v-model="preset"
            class="w-full px-3 py-2 rounded-lg transition-colors"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `1px solid ${theme.colors.border.primary}`,
              color: theme.colors.text.primary,
              fontSize: '14px'
            }"
          >
            <option value="streamlined">Streamlined (Vision → Initiative → Task)</option>
            <option value="full">Full (Vision → Strategy → Initiative → Task)</option>
            <option value="direct">Direct (Vision → Task)</option>
          </select>
          <p 
            class="mt-1 text-xs"
            :style="{ 
              color: theme.colors.text.tertiary 
            }"
          >
            Determines which document types are available in your project
          </p>
        </div>

        <!-- Project Prefix Input -->
        <div class="mb-4">
          <label 
            class="block mb-2 font-medium"
            :style="{ 
              fontSize: '14px', 
              color: theme.colors.text.primary 
            }"
          >
            Project Prefix
          </label>
          <input
            v-model="prefix"
            type="text"
            class="w-full px-3 py-2 rounded-lg transition-colors"
            :style="{
              backgroundColor: theme.colors.background.primary,
              border: `1px solid ${error ? theme.colors.border.error : theme.colors.border.primary}`,
              color: theme.colors.text.primary,
              fontSize: '14px'
            }"
            placeholder="PROJ"
            maxlength="8"
            autofocus
            @input="handlePrefixChange"
          />
          <p 
            class="mt-1 text-xs"
            :style="{ 
              color: theme.colors.text.tertiary 
            }"
          >
            2-8 uppercase letters (used for document IDs like {{ prefix || 'PROJ' }}-V-0001)
          </p>
          <p 
            v-if="error"
            class="mt-1 text-xs"
            :style="{ 
              color: theme.colors.border.error 
            }"
          >
            {{ error }}
          </p>
        </div>

        <!-- Buttons -->
        <div class="flex gap-3 justify-end">
          <button
            type="button"
            class="px-4 py-2 rounded-lg font-medium transition-colors"
            :style="{
              backgroundColor: theme.colors.background.secondary,
              color: theme.colors.text.primary,
              border: `1px solid ${theme.colors.border.primary}`,
              fontSize: '14px'
            }"
            @click="emit('cancel')"
            @mouseenter="handleCancelHover"
            @mouseleave="handleCancelLeave"
          >
            Cancel
          </button>
          
          <button
            type="submit"
            :disabled="!!error || !prefix"
            class="px-4 py-2 rounded-lg font-medium transition-colors"
            :style="{
              backgroundColor: error || !prefix ? theme.colors.background.tertiary : theme.colors.interactive.primary,
              color: error || !prefix ? theme.colors.text.tertiary : theme.colors.text.inverse,
              border: `1px solid ${error || !prefix ? theme.colors.border.secondary : theme.colors.interactive.primary}`,
              fontSize: '14px',
              cursor: error || !prefix ? 'not-allowed' : 'pointer'
            }"
            @mouseenter="handleSubmitHover"
            @mouseleave="handleSubmitLeave"
          >
            Initialize Project
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useTheme } from '../composables/useTheme'

interface Props {
  isOpen: boolean
  directoryName: string
}

interface Emits {
  (e: 'confirm', prefix: string, preset: string): void
  (e: 'cancel'): void
  (e: 'update:isOpen', value: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { theme } = useTheme()
const prefix = ref('')
const preset = ref('streamlined') // Default to streamlined
const error = ref('')

// Generate default prefix when dialog opens
watch(() => props.isOpen, (isOpen) => {
  if (isOpen && props.directoryName) {
    const defaultPrefix = props.directoryName.toUpperCase().replace(/[^A-Z]/g, '').slice(0, 8)
    prefix.value = defaultPrefix.length >= 2 ? defaultPrefix : 'PROJ'
    error.value = ''
  }
})

const validatePrefix = (value: string): string => {
  if (value.length < 2) return 'Prefix must be at least 2 characters'
  if (value.length > 8) return 'Prefix must be no more than 8 characters'
  if (!/^[A-Z]+$/.test(value)) return 'Prefix must contain only uppercase letters'
  return ''
}

const handlePrefixChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const value = target.value.toUpperCase().replace(/[^A-Z]/g, '')
  prefix.value = value
  target.value = value
  error.value = validatePrefix(value)
}

const handleSubmit = (e: Event) => {
  e.preventDefault()
  const validationError = validatePrefix(prefix.value)
  if (validationError) {
    error.value = validationError
    return
  }
  emit('confirm', prefix.value, preset.value)
}

const handleCancelHover = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.interactive.secondary
}

const handleCancelLeave = (e: Event) => {
  const target = e.currentTarget as HTMLElement
  target.style.backgroundColor = theme.value.colors.background.secondary
}

const handleSubmitHover = (e: Event) => {
  if (!error.value && prefix.value) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.interactive.primaryHover
    target.style.borderColor = theme.value.colors.interactive.primaryHover
  }
}

const handleSubmitLeave = (e: Event) => {
  if (!error.value && prefix.value) {
    const target = e.currentTarget as HTMLElement
    target.style.backgroundColor = theme.value.colors.interactive.primary
    target.style.borderColor = theme.value.colors.interactive.primary
  }
}
</script>