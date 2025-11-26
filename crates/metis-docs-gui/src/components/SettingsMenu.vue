<template>
  <div class="relative">
    <button
      @click="toggleMenu"
      class="transition-all"
      style="
        padding: 0.5rem 1rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 1rem;
        background-color: var(--color-interactive-primary);
        color: var(--color-text-inverse);
        border: 2px solid var(--color-interactive-primary);
      "
      title="Settings Menu"
    >
      ☰ Menu
    </button>

    <Transition name="dropdown">
      <div
        v-if="isOpen"
        class="absolute right-0 mt-2 w-80 rounded-lg shadow-lg z-50"
        style="background-color: var(--color-background-elevated); border: 2px solid var(--color-border-primary);"
      >
        <div class="py-2">
          <!-- Theme Selection -->
          <div class="px-6 py-2 text-xs font-semibold" style="color: var(--color-text-secondary);">Theme</div>
          <button
            v-for="theme in themes"
            :key="theme"
            @click="handleThemeChange(theme)"
            class="w-full text-left px-6 py-4 text-lg font-medium transition-colors flex items-center justify-between"
            style="background-color: transparent; color: var(--color-text-primary);"
            @mouseenter="$event.currentTarget.style.backgroundColor = 'var(--color-interactive-secondary)'"
            @mouseleave="$event.currentTarget.style.backgroundColor = 'transparent'"
          >
            <span>{{ themeLabels[theme] }}</span>
            <span v-if="themeName === theme" class="text-xl">✓</span>
          </button>

          <!-- Divider -->
          <div class="my-2" style="border-top: 1px solid var(--color-border-primary);"></div>

          <!-- CLI Section -->
          <div class="px-6 py-2 text-xs font-semibold" style="color: var(--color-text-secondary);">CLI</div>
          <button
            @click="handleReinstallCli"
            :disabled="isInstalling"
            class="w-full text-left px-6 py-4 text-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-3"
            style="background-color: transparent; color: var(--color-text-primary);"
            @mouseenter="!isInstalling && ($event.currentTarget.style.backgroundColor = 'var(--color-interactive-secondary)')"
            @mouseleave="$event.currentTarget.style.backgroundColor = 'transparent'"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" style="color: var(--color-text-primary);">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            <span>{{ isInstalling ? 'Installing...' : 'Re-install' }}</span>
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { installCliElevated } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'

const { themeName, setTheme } = useTheme()

const isOpen = ref(false)
const isInstalling = ref(false)

const themeLabels = {
  light: 'Light',
  dark: 'Dark',
  hyper: 'Hyper'
} as const

const themes: Array<keyof typeof themeLabels> = ['light', 'dark', 'hyper']

const toggleMenu = () => {
  isOpen.value = !isOpen.value
}

const handleThemeChange = (theme: keyof typeof themeLabels) => {
  setTheme(theme)
  isOpen.value = false
}

const handleReinstallCli = async () => {
  isInstalling.value = true
  isOpen.value = false

  try {
    await installCliElevated()
    // Success toast will be shown by App.vue listening to 'cli-installed' event
  } catch (error) {
    console.error('Failed to install CLI:', error)
    // Error will be handled by the Tauri command
  } finally {
    isInstalling.value = false
  }
}

// Close menu when clicking outside
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.relative')) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
