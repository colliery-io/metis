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
              style="width: 64px; height: 64px; cursor: pointer"
              class="home-icon-glow"
              title="Home"
            />
            <ThemeToggle />
          </div>
          
          <!-- Main content area - matches main content width -->
          <div class="flex-1 flex items-center justify-center py-4">
            <h1 class="text-xl font-semibold text-primary">Metis</h1>
          </div>
        </div>

        <!-- Main content area with sidebar -->
        <div class="flex-1 flex overflow-hidden">
          <ProjectSidebar
            :onProjectSelect="handleProjectSelect"
            :onShowProjectBrowser="handleShowProjectBrowser"
          />
          <div class="flex-1 flex flex-col overflow-hidden">
            <KanbanBoard 
              v-if="currentProject"
              :onBackToProjects="() => setCurrentProject(null)"
            />
            <div v-else class="flex-1 flex items-center justify-center">
              <div class="text-center">
                <!-- Mascot -->
                <div>
                  <img
                    :src="getMascotImage()"
                    alt="Metis mascot"
                    class="mx-auto animate-bounce-gentle filter drop-shadow-glow"
                    style="width: 512px; height: 512px"
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

// Debug watcher
watch(currentProject, (newProject, oldProject) => {
  console.log('currentProject changed:', { 
    old: oldProject?.path, 
    new: newProject?.path,
    hasNew: !!newProject 
  })
}, { immediate: true })

const handleProjectSelect = async (project: ProjectInfo) => {
  console.log('App handleProjectSelect called', project.path)
  try {
    await loadProject(project.path)
    showProjectBrowser.value = false
    console.log('Project loaded successfully')
    console.log('Current project after load:', currentProject.value)
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
</script>