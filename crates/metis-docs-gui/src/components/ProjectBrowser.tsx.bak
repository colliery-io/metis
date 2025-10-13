import { useState } from 'react';
import { useProject } from '../contexts/ProjectContext';
import { DirectoryPicker } from './DirectoryPicker';
import { ProjectCard } from './ProjectCard';
import { ProjectInfo } from '../lib/tauri-api';

export function ProjectBrowser() {
  const { state, loadProject } = useProject();
  const [validatingPath, setValidatingPath] = useState<string | null>(null);

  const handleDirectorySelected = async (path: string) => {
    setValidatingPath(path);
    try {
      await loadProject(path);
    } finally {
      setValidatingPath(null);
    }
  };

  const handleProjectSelect = async (path: string) => {
    if (state.isLoading || validatingPath) return;
    await loadProject(path);
  };

  const isProjectLoading = (project: ProjectInfo) => {
    return validatingPath === project.path || 
           (state.isLoading && state.currentProject?.path === project.path);
  };

  if (state.currentProject) {
    return (
      <div className="max-w-2xl mx-auto p-6">
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center">
            <span className="text-green-600 text-xl mr-3">✅</span>
            <div>
              <h2 className="text-lg font-medium text-green-800">
                Project Loaded Successfully
              </h2>
              <p className="text-green-700">
                {state.currentProject.path}
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">
          Select Metis Project
        </h1>
        <p className="text-gray-600">
          Choose a directory containing a Metis project, or browse for one
        </p>
      </div>

      {/* Directory Picker */}
      <div className="mb-8 text-center">
        <DirectoryPicker
          onDirectorySelected={handleDirectorySelected}
          disabled={state.isLoading || !!validatingPath}
          className="px-6 py-3 text-lg"
        >
          {validatingPath ? 'Validating...' : 'Browse for Project Directory'}
        </DirectoryPicker>
        
        {validatingPath && (
          <div className="mt-4 flex items-center justify-center text-blue-600">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600 mr-2"></div>
            Validating: {validatingPath}
          </div>
        )}
      </div>

      {/* Error Display */}
      {state.error && (
        <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center">
            <span className="text-red-600 text-xl mr-3">❌</span>
            <div>
              <h3 className="text-lg font-medium text-red-800">
                Error Loading Project
              </h3>
              <p className="text-red-700">{state.error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Recent Projects */}
      {state.recentProjects.length > 0 && (
        <div>
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Recent Projects
          </h2>
          <div className="grid gap-4 md:grid-cols-2">
            {state.recentProjects.map((project) => (
              <div key={project.path} className="relative">
                <ProjectCard
                  project={project}
                  onSelect={handleProjectSelect}
                  isSelected={false}
                />
                {isProjectLoading(project) && (
                  <div className="absolute inset-0 bg-white bg-opacity-75 rounded-lg flex items-center justify-center">
                    <div className="flex items-center text-blue-600">
                      <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600 mr-2"></div>
                      Loading...
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {state.recentProjects.length === 0 && !state.error && !validatingPath && (
        <div className="text-center py-12">
          <div className="text-gray-400 text-6xl mb-4">📁</div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            No Recent Projects
          </h3>
          <p className="text-gray-600">
            Use the browse button above to select your first Metis project
          </p>
        </div>
      )}
    </div>
  );
}