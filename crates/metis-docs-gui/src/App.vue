<template>
  <div class="h-screen bg-secondary overflow-hidden">
    <!-- Onboarding Wizard -->
    <OnboardingWizard
      :isOpen="showOnboarding"
      @skip="handleOnboardingSkip"
      @complete="handleOnboardingComplete"
      @close="showOnboarding = false"
    />

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
            <img
              :src="getMascotImage()"
              alt="Home"
              @click="setCurrentProject(null)"
              class="home-icon-topbar cursor-pointer"
              title="Home"
            />
            <SettingsMenu />
          </div>
          
          <!-- Main content area - matches main content width -->
          <div class="flex-1 flex items-center px-6 py-4">
            <!-- Search Bar (left side, only when project is loaded) -->
            <div class="w-80 ml-4">
              <SearchBar v-if="currentProject" />
            </div>
            <!-- Centered project title -->
            <h1 class="flex-1 text-xl font-semibold text-primary text-center">{{ getProjectDisplayName() }}</h1>
            <!-- Sync status + spacer to balance the search bar -->
            <div class="w-80 flex items-center justify-end">
              <SyncStatusIndicator v-if="currentProject" ref="syncStatusRef" />
            </div>
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
            <div v-else class="home-screen">
              <!-- Atmospheric Background -->
              <div class="home-atmosphere"></div>

              <div class="home-content">
                <!-- Mascot -->
                <div class="mascot-container">
                  <img
                    :src="getMascotImage()"
                    alt="Metis mascot"
                    class="home-icon-main"
                  />
                </div>

                <!-- Tagline -->
                <div class="home-tagline">
                  <p class="tagline-text">Flight Levels Project Management</p>
                  <p class="tagline-sub">Vision → Initiative → Task</p>
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
import SettingsMenu from './components/SettingsMenu.vue'
import ProjectSidebar from './components/ProjectSidebar.vue'
import KanbanBoard from './components/KanbanBoard.vue'
import SearchBar from './components/SearchBar.vue'
import SyncStatusIndicator from './components/SyncStatusIndicator.vue'
import OnboardingWizard from './components/OnboardingWizard.vue'
import { ProjectInfo, DocumentInfo, MetisAPI } from './lib/tauri-api'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'


const { currentProject, setCurrentProject, loadProject } = useProject()
const { themeName } = useTheme()

// Toast notification state
const toastMessage = ref<string | null>(null)
const toastVisible = ref(false)
const toastType = ref<'success' | 'error'>('success')
let toastTimeout: ReturnType<typeof setTimeout> | null = null
let unlistenCliInstalled: UnlistenFn | null = null
let unlistenShowToast: UnlistenFn | null = null
let unlistenSearchSelect: UnlistenFn | null = null

// Onboarding wizard state
const showOnboarding = ref(false)
const syncStatusRef = ref<InstanceType<typeof SyncStatusIndicator> | null>(null)

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

watch(currentProject, async (project) => {
  if (project) {
    // Check if upstream is configured — if not, offer onboarding
    try {
      const configured = await MetisAPI.isUpstreamConfigured()
      if (!configured) {
        // Check if onboarding was previously dismissed for this project
        const dismissed = localStorage.getItem(`metis-onboarding-dismissed-${project.path}`)
        if (!dismissed) {
          showOnboarding.value = true
        }
      }
    } catch {
      // Silently fail — don't block app usage
    }
    // Refresh sync status indicator
    syncStatusRef.value?.checkUpstream()
  }
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

const handleOnboardingSkip = () => {
  showOnboarding.value = false
  if (currentProject.value) {
    localStorage.setItem(`metis-onboarding-dismissed-${currentProject.value.path}`, 'true')
  }
}

const handleOnboardingComplete = () => {
  showOnboarding.value = false
  // Refresh sync status indicator to show the new upstream
  syncStatusRef.value?.checkUpstream()
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
  height: auto;
  aspect-ratio: 1;
  object-fit: contain;
  cursor: pointer;
  transition: all 0.3s ease;
}

.home-icon-topbar:hover {
  transform: scale(1.05);
}

/* Home Icon - Main Screen */
.home-icon-main {
  width: min(45vw, 70vh, 500px);
  height: auto;
  aspect-ratio: 1;
  object-fit: contain;
  min-width: 200px;
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

/* Home Screen */
.home-screen {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

/* Atmospheric background with gradient blobs */
.home-atmosphere {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

:global([data-theme="light"]) .home-atmosphere {
  background:
    radial-gradient(ellipse 60% 50% at 25% 30%, rgba(157, 107, 83, 0.08) 0%, transparent 60%),
    radial-gradient(ellipse 50% 60% at 75% 70%, rgba(91, 124, 101, 0.06) 0%, transparent 60%),
    radial-gradient(ellipse 40% 40% at 50% 50%, rgba(201, 162, 39, 0.04) 0%, transparent 50%);
}

:global([data-theme="dark"]) .home-atmosphere {
  background:
    radial-gradient(ellipse 60% 50% at 20% 25%, rgba(96, 165, 250, 0.06) 0%, transparent 60%),
    radial-gradient(ellipse 50% 60% at 80% 75%, rgba(56, 189, 248, 0.05) 0%, transparent 60%),
    radial-gradient(ellipse 40% 40% at 50% 50%, rgba(167, 139, 250, 0.04) 0%, transparent 50%);
}

:global([data-theme="hyper"]) .home-atmosphere {
  background:
    radial-gradient(ellipse 60% 50% at 20% 25%, rgba(224, 64, 251, 0.1) 0%, transparent 60%),
    radial-gradient(ellipse 50% 60% at 80% 75%, rgba(0, 229, 255, 0.08) 0%, transparent 60%),
    radial-gradient(ellipse 40% 40% at 50% 50%, rgba(0, 230, 118, 0.05) 0%, transparent 50%);
}

.home-content {
  position: relative;
  z-index: 1;
  text-align: center;
}

.mascot-container {
  margin-bottom: 24px;
}

/* Tagline */
.home-tagline {
  animation: tagline-enter 0.8s ease-out 0.3s backwards;
}

.tagline-text {
  font-family: var(--font-display);
  font-size: clamp(1rem, 2vw, 1.25rem);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 8px 0;
  letter-spacing: -0.01em;
}

.tagline-sub {
  font-family: var(--font-mono);
  font-size: clamp(0.75rem, 1.2vw, 0.875rem);
  color: var(--color-text-tertiary);
  margin: 0;
  letter-spacing: 0.05em;
}

@keyframes tagline-enter {
  from {
    opacity: 0;
    transform: translateY(16px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Respect reduced motion */
@media (prefers-reduced-motion: reduce) {
  .home-tagline {
    animation: none;
  }
}
</style>