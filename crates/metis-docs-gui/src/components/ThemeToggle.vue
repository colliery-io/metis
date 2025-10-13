<template>
  <div class="flex items-center gap-6">
    <span class="text-secondary" style="font-size: 18px">Theme:</span>
    <div class="relative">
      <button
        @click="isOpen = !isOpen"
        class="px-6 py-3 text-secondary hover:text-primary hover:bg-elevated rounded-lg transition-colors min-w-32 bg-transparent border-none"
        style="font-size: 18px"
      >
        {{ themeLabels[themeName] }}
      </button>
      
      <div
        v-if="isOpen"
        class="absolute right-0 top-full mt-2 bg-elevated rounded-lg shadow-lg z-10 min-w-40 overflow-hidden"
      >
        <button
          v-for="theme in themes"
          :key="theme"
          @click="handleThemeSelect(theme)"
          :class="[
            'w-full px-6 py-3 text-left transition-colors border-none',
            theme === themeName 
              ? 'text-primary bg-interactive-secondary' 
              : 'text-secondary hover:bg-secondary hover:text-primary'
          ]"
          :style="{
            fontSize: '16px',
            backgroundColor: theme === themeName ? 'var(--color-interactive-secondary)' : 'transparent'
          }"
        >
          {{ themeLabels[theme] }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTheme } from '../composables/useTheme'

const { themeName, setTheme } = useTheme()
const isOpen = ref(false)

const themeLabels = {
  light: 'Light',
  dark: 'Dark', 
  hyper: 'Hyper'
} as const

const themes = Object.keys(themeLabels) as Array<keyof typeof themeLabels>

const handleThemeSelect = (theme: keyof typeof themeLabels) => {
  setTheme(theme)
  isOpen.value = false
}
</script>