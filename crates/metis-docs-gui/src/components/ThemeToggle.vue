<template>
  <button
    @click="cycleTheme"
    class="px-4 py-2 rounded-lg transition-all font-medium"
    style="background-color: #8b5cf6; color: white; border: 1px solid #8b5cf6;"
    :title="`Switch theme (currently ${themeLabels[themeName]})`"
  >
    {{ themeLabels[themeName] }} Theme
  </button>
</template>

<script setup lang="ts">
import { useTheme } from '../composables/useTheme'

const { themeName, setTheme } = useTheme()

const themeLabels = {
  light: 'Light',
  dark: 'Dark',
  hyper: 'Hyper'
} as const

const themes: Array<keyof typeof themeLabels> = ['light', 'dark', 'hyper']

const cycleTheme = () => {
  const currentIndex = themes.indexOf(themeName.value)
  const nextIndex = (currentIndex + 1) % themes.length
  setTheme(themes[nextIndex])
}
</script>

