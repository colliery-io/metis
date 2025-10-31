<template>
  <div class="h-screen bg-secondary overflow-hidden">
    <div class="h-screen flex flex-col">
      <!-- Show full-screen project browser when explicitly requested -->
      <template v-if="showProjectBrowser">
        <div class="p-4 bg-elevated border-b border-primary flex items-center justify-between">
          <h1 class="text-xl font-semibold text-primary">Select Project</h1>
          <button
            @click="handleBackFromBrowser"
            class="px-4 py-2 text-secondary hover:text-primary hover:bg-secondary rounded-lg transition-colors"
          >
            ← Back
          </button>
        </div>
        <div class="flex-1">
          <ProjectBrowser />
        </div>
      </template>

      <!-- Main app layout with top bar and sidebar -->
      <template v-else>
        <!-- Top Bar -->
        <div class="bg-secondary flex items-center">
          <!-- Left section - matches sidebar width -->
          <div class="w-1/5 flex items-center justify-between px-6 py-4">
            <img
              :src="getMascotImage()"
              alt="Home"
              @click="setCurrentProject(null)"
              class="home-icon-topbar"
              title="Home"
            />
            <ThemeToggle />
          </div>
          
          <!-- Main content area - matches main content width -->
          <div class="flex-1 flex items-center justify-center py-4">
            <h1 class="text-xl font-semibold text-primary">{{ getProjectDisplayName() }}</h1>
          </div>
        </div>

        <!-- Main content area with sidebar -->
        <div class="flex-1 flex overflow-hidden">
          <ProjectSidebar
            :onProjectSelect="handleProjectSelect"
            :onShowProjectBrowser="handleShowProjectBrowser"
          />
          <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Normal Kanban Board -->
            <KanbanBoard 
              v-if="currentProject"
              :onBackToProjects="() => setCurrentProject(null)"
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
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import './App.css'
import './styles/theme.css'
import { useProject } from './composables/useProject'
import { useTheme } from './composables/useTheme'
import ThemeToggle from './components/ThemeToggle.vue'
import ProjectSidebar from './components/ProjectSidebar.vue'
import KanbanBoard from './components/KanbanBoard.vue'
import { ProjectInfo } from './lib/tauri-api'

// Temporary placeholder components until we convert them
const ProjectBrowser = {
  template: '<div class="p-8 text-center text-secondary">ProjectBrowser component - coming soon</div>'
}

const { currentProject, setCurrentProject, loadProject } = useProject()
const { themeName } = useTheme()
const showProjectBrowser = ref(false)

watch(currentProject, () => {
  // Project changed
}, { immediate: true })

const handleProjectSelect = async (project: ProjectInfo) => {
  try {
    await loadProject(project.path)
    showProjectBrowser.value = false
  } catch (error) {
    console.error('Failed to load project:', error)
  }
}

const handleShowProjectBrowser = () => {
  showProjectBrowser.value = true
}

const handleBackFromBrowser = () => {
  showProjectBrowser.value = false
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
</style>