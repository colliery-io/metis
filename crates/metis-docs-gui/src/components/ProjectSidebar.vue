<template>
  <div 
    :class="[
      'bg-secondary flex flex-col transition-all duration-200',
      isExpanded ? 'w-1/5' : 'w-12'
    ]"
  >
    <!-- Collapse/Expand Button -->
    <div class="collapse-button-container">
      <button
        @click="isExpanded = !isExpanded"
        class="collapse-button"
        :title="isExpanded ? 'Collapse sidebar' : 'Expand sidebar'"
      >
        {{ isExpanded ? '<' : '>' }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <template v-if="isExpanded">
        <!-- Add Project Button -->
        <div class="add-project-container">
          <button
            @click="handleAddProject"
            class="add-project-button"
          >
            Add Project
          </button>
        </div>

        <!-- Project List -->
        <div v-if="recentProjects.length > 0" class="px-3">
          <div class="space-y-2">
            <ProjectCard
              v-for="(project, index) in recentProjects.slice(0, 8)"
              :key="`${project.path}-${index}`"
              :project="project"
              :isActive="currentProject?.path === project.path"
              :isSidebar="true"
              :onClick="() => handleProjectClick(project)"
              :onRemove="(e) => handleRemoveProject(e, project.path)"
            />
          </div>
        </div>

        <!-- Empty State -->
        <div v-if="recentProjects.length === 0 && !currentProject" class="px-3 py-6 text-center">
          <div class="text-tertiary text-xs">
            No projects yet
          </div>
          <div class="text-xs text-tertiary mt-1">
            Add a project to get started
          </div>
        </div>
      </template>
      
      <!-- Collapsed view -->
      <div v-else class="collapsed-content">
        <button
          @click="handleAddProject"
          class="add-project-button-collapsed"
          title="Add Project"
        >
          +
        </button>
        <div 
          v-if="currentProject"
          class="current-project-indicator" 
          :title="getProjectName(currentProject)"
        >
          <div class="project-dot"></div>
        </div>
      </div>
    </div>

    <!-- Custom initialization dialog -->
    <InitProjectDialog
      v-model:is-open="showInitDialog"
      :directory-name="directoryName"
      @confirm="handleInitConfirm"
      @cancel="handleInitCancel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProject } from '../composables/useProject'
import { ProjectInfo, MetisAPI } from '../lib/tauri-api'
import { open } from '@tauri-apps/plugin-dialog'
import ProjectCard from './ProjectCard.vue'
import InitProjectDialog from './InitProjectDialog.vue'

interface Props {
  onProjectSelect: (project: ProjectInfo) => void
  onShowProjectBrowser: () => void
}

const props = defineProps<Props>()

const { currentProject, getRecentProjects, loadProject, removeProject } = useProject()
const isExpanded = ref(true)
const showInitDialog = ref(false)
const selectedPath = ref('')
const directoryName = ref('')

const recentProjects = computed(() => {
  const projects = getRecentProjects()
  return projects
})

const handleAddProject = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Directory for Metis Project',
    })
    
    if (selected && typeof selected === 'string') {
      // First, try to load the project to see if it's already a valid Metis project
      try {
        const projectInfo = await MetisAPI.loadProject(selected)
        
        if (projectInfo.is_valid) {
          // Already a valid Metis project, load it normally
          await loadProject(selected)
        } else {
          // Not a valid Metis project, offer to initialize it
          const pathParts = selected.split('/').filter(Boolean)
          const projectName = pathParts[pathParts.length - 1] || 'unknown'
          selectedPath.value = selected
          directoryName.value = projectName
          showInitDialog.value = true
        }
      } catch (loadError) {
        // Failed to check/load project
        // If loading fails, still offer to initialize
        const pathParts = selected.split('/').filter(Boolean)
        const projectName = pathParts[pathParts.length - 1] || 'unknown'
        selectedPath.value = selected
        directoryName.value = projectName
        showInitDialog.value = true
      }
    }
  } catch (error) {
    // Failed to open directory
  }
}

const handleProjectClick = (project: ProjectInfo) => {
  props.onProjectSelect(project)
}

const handleRemoveProject = (e: Event, projectPath: string) => {
  e.stopPropagation()
  removeProject(projectPath)
}

const handleInitConfirm = async (prefix: string) => {
  showInitDialog.value = false
  
  try {
    // Initialize the project with the user-provided prefix
    const initResult = await MetisAPI.initializeProject(selectedPath.value, prefix)
    
    // Now load the newly initialized project
    await loadProject(selectedPath.value)
  } catch (initError) {
    // Failed to initialize project
    alert('Failed to initialize Metis project. Please check the directory permissions and try again.')
  }
}

const handleInitCancel = () => {
  showInitDialog.value = false
  selectedPath.value = ''
  directoryName.value = ''
}

const getProjectName = (project: ProjectInfo): string => {
  // Get the directory name (project name)
  const parts = project.path.split('/').filter(Boolean)
  return parts[parts.length - 1] || 'Unknown Project'
}
</script>

<style scoped>
/* Collapse/Expand Button */
.collapse-button-container {
  padding: 8px 12px;
  display: flex;
  justify-content: flex-end;
}

.collapse-button {
  padding: 6px 10px;
  background-color: transparent;
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.collapse-button:hover {
  background-color: var(--color-background-elevated);
  color: var(--color-text-primary);
  border-color: var(--color-interactive-primary);
}

/* Add Project Button - Expanded */
.add-project-container {
  padding: 12px 12px 8px 12px;
}

.add-project-button {
  width: 100%;
  padding: 10px 16px;
  background-color: var(--color-status-completed);
  border: 1px solid var(--color-status-completed);
  border-radius: 6px;
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.add-project-button:hover {
  background-color: var(--color-status-active);
  border-color: var(--color-status-active);
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

/* Add Project Button - Collapsed */
.collapsed-content {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.add-project-button-collapsed {
  width: 100%;
  padding: 8px;
  background-color: transparent;
  border: 1px solid var(--color-border-primary);
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-project-button-collapsed:hover {
  background-color: var(--color-background-elevated);
  color: var(--color-text-primary);
  border-color: var(--color-interactive-primary);
}

/* Current Project Indicator - Collapsed */
.current-project-indicator {
  width: 100%;
  padding: 8px;
  background-color: var(--color-interactive-secondary);
  border-radius: 8px;
  border: 1px solid var(--color-interactive-primary);
}

.project-dot {
  width: 8px;
  height: 8px;
  background-color: var(--color-interactive-primary);
  border-radius: 50%;
  margin: 0 auto;
}
</style>