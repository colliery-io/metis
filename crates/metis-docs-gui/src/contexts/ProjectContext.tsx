import React, { createContext, useContext, useReducer, ReactNode } from 'react';
import { ProjectInfo } from '../lib/tauri-api';

interface ProjectState {
  currentProject: ProjectInfo | null;
  recentProjects: ProjectInfo[];
  isLoading: boolean;
  error: string | null;
}

type ProjectAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'LOAD_PROJECT_SUCCESS'; payload: ProjectInfo }
  | { type: 'CLEAR_PROJECT' }
  | { type: 'ADD_RECENT_PROJECT'; payload: ProjectInfo }
  | { type: 'LOAD_RECENT_PROJECTS'; payload: ProjectInfo[] };

const initialState: ProjectState = {
  currentProject: null,
  recentProjects: [],
  isLoading: false,
  error: null,
};

function projectReducer(state: ProjectState, action: ProjectAction): ProjectState {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, isLoading: false };
    case 'LOAD_PROJECT_SUCCESS':
      return {
        ...state,
        currentProject: action.payload,
        isLoading: false,
        error: null,
      };
    case 'CLEAR_PROJECT':
      return { ...state, currentProject: null };
    case 'ADD_RECENT_PROJECT':
      const filtered = state.recentProjects.filter(p => p.path !== action.payload.path);
      return {
        ...state,
        recentProjects: [action.payload, ...filtered].slice(0, 10), // Keep only 10 recent
      };
    case 'LOAD_RECENT_PROJECTS':
      return { ...state, recentProjects: action.payload };
    default:
      return state;
  }
}

interface ProjectContextType {
  state: ProjectState;
  dispatch: React.Dispatch<ProjectAction>;
  loadProject: (path: string) => Promise<void>;
  clearProject: () => void;
  getRecentProjects: () => ProjectInfo[];
  saveRecentProject: (project: ProjectInfo) => void;
  // Convenience properties
  currentProject: ProjectInfo | null;
  setCurrentProject: (project: ProjectInfo | null) => void;
}

const ProjectContext = createContext<ProjectContextType | undefined>(undefined);

interface ProjectProviderProps {
  children: ReactNode;
}

export function ProjectProvider({ children }: ProjectProviderProps) {
  const [state, dispatch] = useReducer(projectReducer, initialState);

  // Load recent projects from localStorage on mount
  React.useEffect(() => {
    const stored = localStorage.getItem('metis-recent-projects');
    if (stored) {
      try {
        const recentProjects = JSON.parse(stored);
        dispatch({ type: 'LOAD_RECENT_PROJECTS', payload: recentProjects });
      } catch (error) {
        console.error('Failed to load recent projects:', error);
      }
    }
  }, []);

  const loadProject = async (path: string): Promise<void> => {
    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });

    try {
      const { MetisAPI } = await import('../lib/tauri-api');
      const projectInfo = await MetisAPI.loadProject(path);
      
      if (projectInfo.is_valid) {
        dispatch({ type: 'LOAD_PROJECT_SUCCESS', payload: projectInfo });
        dispatch({ type: 'ADD_RECENT_PROJECT', payload: projectInfo });
        saveRecentProject(projectInfo);
      } else {
        dispatch({ type: 'SET_ERROR', payload: 'Invalid Metis project directory' });
      }
    } catch (error) {
      dispatch({ 
        type: 'SET_ERROR', 
        payload: error instanceof Error ? error.message : 'Failed to load project' 
      });
    }
  };

  const clearProject = () => {
    dispatch({ type: 'CLEAR_PROJECT' });
  };

  const getRecentProjects = (): ProjectInfo[] => {
    return state.recentProjects;
  };

  const saveRecentProject = (project: ProjectInfo) => {
    const filtered = state.recentProjects.filter(p => p.path !== project.path);
    const updated = [project, ...filtered].slice(0, 10);
    localStorage.setItem('metis-recent-projects', JSON.stringify(updated));
  };

  const setCurrentProject = (project: ProjectInfo | null) => {
    if (project) {
      dispatch({ type: 'LOAD_PROJECT_SUCCESS', payload: project });
    } else {
      dispatch({ type: 'CLEAR_PROJECT' });
    }
  };

  const value: ProjectContextType = {
    state,
    dispatch,
    loadProject,
    clearProject,
    getRecentProjects,
    saveRecentProject,
    currentProject: state.currentProject,
    setCurrentProject,
  };

  return (
    <ProjectContext.Provider value={value}>
      {children}
    </ProjectContext.Provider>
  );
}

export function useProject(): ProjectContextType {
  const context = useContext(ProjectContext);
  if (context === undefined) {
    throw new Error('useProject must be used within a ProjectProvider');
  }
  return context;
}