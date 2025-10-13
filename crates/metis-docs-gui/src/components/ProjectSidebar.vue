<template>
  <div 
    :class="[
      'bg-secondary flex flex-col transition-all duration-200',
      isExpanded ? 'w-1/5' : 'w-12'
    ]"
  >
    <!-- Collapse/Expand Button -->
    <div class="px-3 py-2 flex justify-end">
      <button
        @click="isExpanded = !isExpanded"
        class="btn btn-ghost btn-sm"
        :title="isExpanded ? 'Collapse sidebar' : 'Expand sidebar'"
      >
        {{ isExpanded ? '<' : '>' }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <template v-if="isExpanded">
        <!-- Add Project Button -->
        <div class="px-3 pt-3 pb-2">
          <button
            @click="handleAddProject"
            class="btn btn-secondary btn-sm w-full"
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
      <div v-else class="p-2 space-y-2">
        <button
          @click="handleAddProject"
          class="btn btn-ghost w-full"
          title="Add Project"
        >
          +
        </button>
        <div 
          v-if="currentProject"
          class="w-full p-2 bg-interactive-secondary rounded-lg" 
          :title="getProjectName(currentProject)"
        >
          <div class="w-2 h-2 bg-interactive-primary rounded-full mx-auto"></div>
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
  console.log('Recent projects:', projects)
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
        console.error('Failed to check/load project:', loadError)
        // If loading fails, still offer to initialize
        const pathParts = selected.split('/').filter(Boolean)
        const projectName = pathParts[pathParts.length - 1] || 'unknown'
        selectedPath.value = selected
        directoryName.value = projectName
        showInitDialog.value = true
      }
    }
  } catch (error) {
    console.error('Failed to open directory:', error)
  }
}

const handleProjectClick = (project: ProjectInfo) => {
  console.log('ProjectSidebar handleProjectClick called', project.path)
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
    console.log('Project initialized:', initResult)
    
    // Now load the newly initialized project
    await loadProject(selectedPath.value)
  } catch (initError) {
    console.error('Failed to initialize project:', initError)
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