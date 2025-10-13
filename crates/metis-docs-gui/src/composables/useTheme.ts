import { ref, watchEffect, onMounted, readonly, computed } from 'vue'
import type { ThemeName } from '../types/theme'
import { themes } from '../themes/definitions'

const themeName = ref<ThemeName>('light')

export function useTheme() {
  // Load theme preference from localStorage on mount
  onMounted(() => {
    const stored = localStorage.getItem('metis-theme') as ThemeName
    if (stored && themes[stored]) {
      themeName.value = stored
    }
  })

  // Save theme preference to localStorage and apply CSS when changed
  watchEffect(() => {
    localStorage.setItem('metis-theme', themeName.value)
    
    // Apply theme to document root for CSS custom properties
    const root = document.documentElement
    const theme = themes[themeName.value]
    
    // Set CSS custom properties for the current theme
    Object.entries(theme.colors).forEach(([category, colors]) => {
      if (typeof colors === 'object' && colors !== null) {
        Object.entries(colors).forEach(([key, value]) => {
          if (typeof value === 'string') {
            root.style.setProperty(`--color-${category}-${key}`, value)
          }
        })
      }
    })
    
    // Set theme name as data attribute for potential CSS selectors
    root.setAttribute('data-theme', themeName.value)
  })

  const setTheme = (newThemeName: ThemeName) => {
    themeName.value = newThemeName
  }

  const toggleTheme = () => {
    const themeOrder: ThemeName[] = ['light', 'dark', 'hyper']
    const currentIndex = themeOrder.indexOf(themeName.value)
    const nextIndex = (currentIndex + 1) % themeOrder.length
    themeName.value = themeOrder[nextIndex]
  }

  return {
    theme: computed(() => themes[themeName.value]),
    themeName: readonly(themeName),
    setTheme,
    toggleTheme,
  }
}

// Utility composable to get theme-aware styles
export function useThemedStyles() {
  const { theme } = useTheme()
  
  return {
    // Helper functions for common style patterns
    bg: (level: keyof typeof theme.value.colors.background) => ({
      backgroundColor: theme.value.colors.background[level],
    }),
    text: (level: keyof typeof theme.value.colors.text) => ({
      color: theme.value.colors.text[level],
    }),
    border: (level: keyof typeof theme.value.colors.border) => ({
      borderColor: theme.value.colors.border[level],
    }),
    // Direct access to theme colors
    colors: theme.value.colors,
  }
}