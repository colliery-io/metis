import { reactive, onMounted, readonly, toRef } from 'vue'
import type { ProjectInfo } from '../lib/tauri-api'

interface ProjectState {
  currentProject: ProjectInfo | null
  recentProjects: ProjectInfo[]
  isLoading: boolean
  error: string | null
}

const state = reactive<ProjectState>({
  currentProject: null,
  recentProjects: [],
  isLoading: false,
  error: null,
})

export function useProject() {
  // Load recent projects from localStorage on first use
  onMounted(() => {
    const stored = localStorage.getItem('metis-recent-projects')
    if (stored) {
      try {
        const recentProjects = JSON.parse(stored)
        state.recentProjects = recentProjects
      } catch (error) {
        // Failed to load recent projects
      }
    }
  })

  const loadProject = async (path: string): Promise<void> => {
    state.isLoading = true
    state.error = null

    try {
      const { MetisAPI } = await import('../lib/tauri-api')
      const projectInfo = await MetisAPI.loadProject(path)
      
      if (projectInfo.is_valid) {
        state.currentProject = projectInfo
        state.isLoading = false
        state.error = null
        addRecentProject(projectInfo)
        saveRecentProject(projectInfo)
      } else {
        state.error = 'Invalid Metis project directory'
        state.isLoading = false
      }
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'Failed to load project'
      state.isLoading = false
    }
  }

  const clearProject = () => {
    state.currentProject = null
  }

  const setCurrentProject = (project: ProjectInfo | null) => {
    state.currentProject = project
  }

  const addRecentProject = (project: ProjectInfo) => {
    const filtered = state.recentProjects.filter(p => p.path !== project.path)
    state.recentProjects = [project, ...filtered].slice(0, 10) // Keep only 10 recent
  }

  const getRecentProjects = (): ProjectInfo[] => {
    return state.recentProjects
  }

  const saveRecentProject = (project: ProjectInfo) => {
    const filtered = state.recentProjects.filter(p => p.path !== project.path)
    const updated = [project, ...filtered].slice(0, 10)
    localStorage.setItem('metis-recent-projects', JSON.stringify(updated))
  }

  const removeProject = (path: string) => {
    state.recentProjects = state.recentProjects.filter(p => p.path !== path)
    const updated = state.recentProjects.filter(p => p.path !== path)
    localStorage.setItem('metis-recent-projects', JSON.stringify(updated))
    
    // If we're removing the current project, clear it
    if (state.currentProject?.path === path) {
      state.currentProject = null
    }
  }

  return {
    // State
    state: readonly(state),
    currentProject: readonly(toRef(state, 'currentProject')),
    recentProjects: readonly(toRef(state, 'recentProjects')),
    isLoading: readonly(toRef(state, 'isLoading')),
    error: readonly(toRef(state, 'error')),
    
    // Actions
    loadProject,
    clearProject,
    setCurrentProject,
    getRecentProjects,
    saveRecentProject,
    removeProject,
  }
}