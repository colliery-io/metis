<template>
  <div class="theme-toggle-container">
    <span class="theme-label">Theme:</span>
    <div class="theme-selector">
      <button
        @click="isOpen = !isOpen"
        class="theme-button"
      >
        {{ themeLabels[themeName] }}
      </button>
      
      <div
        v-if="isOpen"
        class="theme-dropdown"
      >
        <button
          v-for="theme in themes"
          :key="theme"
          @click="handleThemeSelect(theme)"
          :class="[
            'theme-option',
            { 'theme-option-active': theme === themeName }
          ]"
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

<style scoped>
.theme-toggle-container {
  display: flex;
  align-items: center;
  gap: 24px;
}

.theme-label {
  color: var(--color-text-secondary);
  font-size: 18px;
  font-weight: 500;
}

.theme-selector {
  position: relative;
}

.theme-button {
  padding: 12px 24px;
  background-color: transparent;
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  color: var(--color-text-secondary);
  font-size: 18px;
  font-weight: 500;
  min-width: 128px;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.theme-button:hover {
  color: var(--color-text-primary);
  background-color: var(--color-background-elevated);
  border-color: var(--color-interactive-primary);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.theme-dropdown {
  position: absolute;
  right: 0;
  top: 100%;
  margin-top: 8px;
  background-color: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 10;
  min-width: 160px;
  overflow: hidden;
}

.theme-option {
  width: 100%;
  padding: 12px 24px;
  background-color: transparent;
  border: none;
  color: var(--color-text-secondary);
  font-size: 16px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-option:hover {
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.theme-option-active {
  color: var(--color-text-primary);
  background-color: var(--color-interactive-secondary);
}

.theme-option-active:hover {
  background-color: var(--color-interactive-secondary);
}
</style>