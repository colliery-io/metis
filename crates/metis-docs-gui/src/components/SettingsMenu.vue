<template>
  <div class="relative">
    <button
      @click="toggleMenu"
      class="menu-trigger"
      :class="{ 'menu-open': isOpen }"
      title="Settings"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83"></path>
      </svg>
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen" class="menu-dropdown">
        <!-- Theme Selection -->
        <div class="menu-section-label">Theme</div>
        <button
          v-for="theme in themes"
          :key="theme"
          @click="handleThemeChange(theme)"
          class="menu-item"
          :class="{ 'menu-item-active': themeName === theme }"
        >
          <span>{{ themeLabels[theme] }}</span>
          <svg v-if="themeName === theme" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </button>

        <!-- Divider -->
        <div class="menu-divider"></div>

        <!-- CLI Section -->
        <div class="menu-section-label">CLI</div>
        <button
          @click="handleReinstallCli"
          :disabled="isInstalling"
          class="menu-item"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          <span>{{ isInstalling ? 'Installing...' : 'Re-install CLI' }}</span>
        </button>

        <!-- Version -->
        <div class="menu-divider"></div>
        <div class="menu-version">v{{ appVersion }}</div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { installCliElevated, getAppVersion } from '../lib/tauri-api'
import { useTheme } from '../composables/useTheme'

const { themeName, setTheme } = useTheme()

const isOpen = ref(false)
const isInstalling = ref(false)
const appVersion = ref('...')

onMounted(async () => {
  try {
    appVersion.value = await getAppVersion()
  } catch {
    appVersion.value = '0.0.0'
  }
  document.addEventListener('click', handleClickOutside)
})

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

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* Menu trigger button */
.menu-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  background: transparent;
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.menu-trigger:hover {
  background: var(--color-background-tertiary);
  color: var(--color-text-primary);
  border-color: var(--color-border-secondary);
}

.menu-trigger.menu-open {
  background: var(--color-background-tertiary);
  color: var(--color-interactive-primary);
  border-color: var(--color-interactive-primary);
}

.menu-trigger svg {
  transition: transform 0.3s ease;
}

.menu-trigger:hover svg,
.menu-trigger.menu-open svg {
  transform: rotate(45deg);
}

/* Dropdown menu */
.menu-dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  width: 200px;
  background: var(--color-background-elevated);
  border: 1px solid var(--color-border-primary);
  border-radius: 10px;
  box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.15);
  padding: 6px;
  z-index: 50;
}

.menu-section-label {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--color-text-tertiary);
  padding: 8px 12px 4px;
}

.menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.menu-item:hover {
  background: var(--color-background-tertiary);
}

.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-item-active {
  color: var(--color-interactive-primary);
}

.menu-item svg {
  flex-shrink: 0;
}

.menu-divider {
  height: 1px;
  background: var(--color-border-primary);
  margin: 6px 0;
}

.menu-version {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  text-align: center;
  padding: 8px 12px;
  letter-spacing: 0.03em;
}

/* Dropdown transitions */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}
</style>
