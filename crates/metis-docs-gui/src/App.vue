<template>
  <div class="h-screen bg-secondary overflow-hidden">
    <!-- Toast notification -->
    <Transition name="toast">
      <div
        v-if="toastVisible && toastMessage"
        class="fixed bottom-4 right-4 z-50 bg-elevated border border-primary rounded-lg shadow-lg p-4 max-w-sm"
      >
        <div class="flex items-start gap-3">
          <div :class="['flex-shrink-0', toastType === 'error' ? 'text-red-500' : 'text-green-500']">
            <svg v-if="toastType === 'success'" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
            </svg>
            <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
            </svg>
          </div>
          <div class="flex-1">
            <p class="text-sm text-primary">{{ toastMessage }}</p>
          </div>
          <button @click="hideToast" class="text-secondary hover:text-primary">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
            </svg>
          </button>
        </div>
      </div>
    </Transition>

    <div class="h-screen flex flex-col">
        <!-- Top Bar -->
        <div class="bg-secondary flex items-center">
          <!-- Left section - matches sidebar width -->
          <div class="w-1/5 flex items-center justify-between px-6 py-4">
            <div class="flex flex-col items-center">
              <img
                :src="getMascotImage()"
                alt="Home"
                @click="setCurrentProject(null)"
                class="home-icon-topbar cursor-pointer"
                title="Home"
              />
              <span class="text-xs text-secondary mt-1">v{{ appVersion }}</span>
            </div>
            <ThemeToggle />
          </div>
          
          <!-- Main content area - matches main content width -->
          <div class="flex-1 flex items-center px-6 py-4">
            <!-- Search Bar (left side, only when project is loaded) -->
            <div class="w-80 ml-4">
              <SearchBar v-if="currentProject" />
            </div>
            <!-- Centered project title -->
            <h1 class="flex-1 text-xl font-semibold text-primary text-center">{{ getProjectDisplayName() }}</h1>
            <!-- Spacer to balance the search bar -->
            <div class="w-80"></div>
          </div>
        </div>

        <!-- Main content area with sidebar -->
        <div class="flex-1 flex overflow-hidden">
          <ProjectSidebar
            :onProjectSelect="handleProjectSelect"
          />
          <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Normal Kanban Board -->
            <KanbanBoard
              v-if="currentProject"
              :onBackToProjects="() => setCurrentProject(null)"
              :highlightedDocument="selectedDocument"
            />
            
            <!-- Home Screen -->
            <div v-else class="flex-1 flex items-center justify-center">
              <div class="text-center">
                <!-- Mascot -->
                <div>
                  <img
                    :src="getMascotImage()"
                    alt="Metis mascot"
                    class="home-icon-main"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import './App.css'
import './styles/theme.css'
import { useProject } from './composables/useProject'
import { useTheme } from './composables/useTheme'
import ThemeToggle from './components/ThemeToggle.vue'
import ProjectSidebar from './components/ProjectSidebar.vue'
import KanbanBoard from './components/KanbanBoard.vue'
import SearchBar from './components/SearchBar.vue'
import { ProjectInfo, DocumentInfo, getAppVersion } from './lib/tauri-api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'


const { currentProject, setCurrentProject, loadProject } = useProject()
const { themeName } = useTheme()
const appVersion = ref('...')

// Toast notification state
const toastMessage = ref<string | null>(null)
const toastVisible = ref(false)
const toastType = ref<'success' | 'error'>('success')
let toastTimeout: ReturnType<typeof setTimeout> | null = null
let unlistenCliInstalled: UnlistenFn | null = null
let unlistenShowToast: UnlistenFn | null = null
let unlistenSearchSelect: UnlistenFn | null = null

// Selected document from search (for highlighting/navigation)
const selectedDocument = ref<DocumentInfo | null>(null)

const showToast = (message: string, type: 'success' | 'error' = 'success', duration = 5000) => {
  toastMessage.value = message
  toastType.value = type
  toastVisible.value = true
  if (toastTimeout) clearTimeout(toastTimeout)
  toastTimeout = setTimeout(() => {
    toastVisible.value = false
  }, duration)
}

const hideToast = () => {
  toastVisible.value = false
  if (toastTimeout) clearTimeout(toastTimeout)
}

onMounted(async () => {
  try {
    appVersion.value = await getAppVersion()
  } catch (error) {
    console.error('Failed to get app version:', error)
    appVersion.value = '0.0.0'
  }

  // Listen for CLI installation events
  unlistenCliInstalled = await listen<string>('cli-installed', (event) => {
    showToast(event.payload)
  })

  // Listen for generic toast events from components
  unlistenShowToast = await listen<{ message: string, type?: 'success' | 'error' }>('show-toast', (event) => {
    showToast(event.payload.message, event.payload.type || 'success')
  })

  // Listen for search document selection
  unlistenSearchSelect = await listen<DocumentInfo>('search-select-document', (event) => {
    handleSearchSelect(event.payload)
  })
})

onUnmounted(() => {
  if (unlistenCliInstalled) {
    unlistenCliInstalled()
  }
  if (unlistenShowToast) {
    unlistenShowToast()
  }
  if (unlistenSearchSelect) {
    unlistenSearchSelect()
  }
  if (toastTimeout) {
    clearTimeout(toastTimeout)
  }
})

watch(currentProject, () => {
  // Project changed
}, { immediate: true })

const handleProjectSelect = async (project: ProjectInfo) => {
  try {
    await loadProject(project.path)
  } catch (error) {
    console.error('Failed to load project:', error)
  }
}

const handleSearchSelect = (doc: DocumentInfo) => {
  selectedDocument.value = doc
  // Clear selection after a delay (used for highlighting animation)
  setTimeout(() => {
    selectedDocument.value = null
  }, 2000)
}

const getMascotImage = () => {
  switch (themeName.value) {
    case 'dark':
      return '/assets/metis-dark.png'
    case 'hyper':
      return '/assets/metis-hyper.png'
    case 'light':
    default:
      return '/assets/metis-light.png'
  }
}

const getProjectDisplayName = () => {
  if (!currentProject.value) {
    return 'Metis'
  }
  const parts = currentProject.value.path.split('/').filter(Boolean)
  return parts[parts.length - 1] || 'Metis'
}
</script>

<style scoped>
/* Theme-specific glow colors for main owl only */
:global([data-theme="light"]) .home-icon-main {
  --glow-color: rgba(0, 0, 0, 0.5);
  --glow-size-base-1: 32px;
  --glow-size-base-2: 64px;
  --glow-size-hover-1: 40px;
  --glow-size-hover-2: 80px;
}

:global([data-theme="dark"]) .home-icon-main {
  --glow-color: rgba(255, 255, 255, 0.4);
  --glow-size-base-1: 24px;
  --glow-size-base-2: 48px;
  --glow-size-hover-1: 32px;
  --glow-size-hover-2: 64px;
}

:global([data-theme="hyper"]) .home-icon-main {
  --glow-color: rgba(255, 20, 147, 0.6);
  --glow-size-base-1: 24px;
  --glow-size-base-2: 48px;
  --glow-size-hover-1: 32px;
  --glow-size-hover-2: 64px;
}

/* Home Icon - Top Bar (no glow) */
.home-icon-topbar {
  width: clamp(56px, 10vw, 80px);
  height: clamp(56px, 10vw, 80px);
  cursor: pointer;
  transition: all 0.3s ease;
}

.home-icon-topbar:hover {
  transform: scale(1.05);
}

/* Home Icon - Main Screen */
.home-icon-main {
  width: clamp(320px, 45vw, 640px);
  height: clamp(320px, 45vw, 640px);
  max-width: 90vw;
  max-height: 70vh;
  margin: 0 auto;
  display: block;
  animation: bounce-gentle 3s ease-in-out infinite;
  filter: drop-shadow(0 0 var(--glow-size-base-1, 24px) var(--glow-color, rgba(59, 130, 246, 0.5))) 
          drop-shadow(0 0 var(--glow-size-base-2, 48px) var(--glow-color, rgba(59, 130, 246, 0.2)));
  transition: all 0.3s ease;
}

.home-icon-main:hover {
  transform: scale(1.02);
  filter: drop-shadow(0 0 var(--glow-size-hover-1, 32px) var(--glow-color, rgba(59, 130, 246, 0.7))) 
          drop-shadow(0 0 var(--glow-size-hover-2, 64px) var(--glow-color, rgba(59, 130, 246, 0.3)));
}

/* Gentle bounce animation */
@keyframes bounce-gentle {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

/* Toast transitions */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(20px);
}
</style>