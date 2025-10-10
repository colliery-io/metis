import React, { useState } from 'react';
import { useProject } from '../contexts/ProjectContext';
import { ProjectInfo } from '../lib/tauri-api';
import { open } from '@tauri-apps/plugin-dialog';

export interface ProjectSidebarProps {
  onProjectSelect: (project: ProjectInfo) => void;
  onShowProjectBrowser: () => void;
}

export const ProjectSidebar: React.FC<ProjectSidebarProps> = ({
  onProjectSelect,
  onShowProjectBrowser,
}) => {
  const { state, currentProject, getRecentProjects, loadProject } = useProject();
  const [isExpanded, setIsExpanded] = useState(true);
  const recentProjects = getRecentProjects();

  const handleAddProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Metis Project Directory',
      });
      
      if (selected && typeof selected === 'string') {
        await loadProject(selected);
      }
    } catch (error) {
      console.error('Failed to open directory:', error);
    }
  };

  const handleProjectClick = (project: ProjectInfo) => {
    onProjectSelect(project);
  };

  const getProjectName = (project: ProjectInfo): string => {
    const parts = project.path.split('/');
    return parts[parts.length - 1] || 'Unknown Project';
  };

  const truncatePath = (path: string, maxLength: number = 30): string => {
    if (path.length <= maxLength) return path;
    return '...' + path.slice(-(maxLength - 3));
  };

  return (
    <div className={`bg-white border-r border-gray-200 flex flex-col transition-all duration-200 ${
      isExpanded ? 'w-64' : 'w-12'
    }`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200 flex items-center justify-between">
        {isExpanded && (
          <h2 className="text-lg font-semibold text-gray-800">Projects</h2>
        )}
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="p-1 text-gray-500 hover:text-gray-700 rounded"
          title={isExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {isExpanded ? '◀' : '▶'}
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {isExpanded ? (
          <>
            {/* Add Project Button */}
            <div className="p-3">
              <button
                onClick={handleAddProject}
                className="w-full px-3 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center justify-center gap-2"
              >
                <span>+</span>
                Add Project
              </button>
            </div>

            {/* Browse Projects Button */}
            <div className="px-3 pb-3">
              <button
                onClick={onShowProjectBrowser}
                className="w-full px-3 py-2 text-sm border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Browse Projects
              </button>
            </div>

            {/* Current Project */}
            {currentProject && (
              <div className="px-3 pb-3">
                <div className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
                  Current Project
                </div>
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                  <div className="font-medium text-blue-900 text-sm">
                    {getProjectName(currentProject)}
                  </div>
                  <div className="text-xs text-blue-600 mt-1" title={currentProject.path}>
                    {truncatePath(currentProject.path)}
                  </div>
                  <div className="mt-2 flex items-center gap-2">
                    <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                      currentProject.is_valid 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {currentProject.is_valid ? 'Valid' : 'Invalid'}
                    </span>
                    {currentProject.vision_exists && (
                      <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                        Vision
                      </span>
                    )}
                  </div>
                </div>
              </div>
            )}

            {/* Recent Projects */}
            {recentProjects.length > 0 && (
              <div className="px-3">
                <div className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
                  Recent Projects
                </div>
                <div className="space-y-1">
                  {recentProjects
                    .filter(p => p.path !== currentProject?.path)
                    .slice(0, 8)
                    .map((project, index) => (
                    <button
                      key={`${project.path}-${index}`}
                      onClick={() => handleProjectClick(project)}
                      className="w-full text-left p-2 rounded-lg hover:bg-gray-50 transition-colors border border-transparent hover:border-gray-200"
                    >
                      <div className="font-medium text-sm text-gray-900">
                        {getProjectName(project)}
                      </div>
                      <div className="text-xs text-gray-500 mt-1" title={project.path}>
                        {truncatePath(project.path)}
                      </div>
                      <div className="mt-1">
                        <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium ${
                          project.is_valid 
                            ? 'bg-green-100 text-green-700' 
                            : 'bg-red-100 text-red-700'
                        }`}>
                          {project.is_valid ? '✓' : '✗'}
                        </span>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Empty State */}
            {recentProjects.length === 0 && !currentProject && (
              <div className="px-3 py-8 text-center">
                <div className="text-gray-400 text-sm">
                  No projects yet
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  Add a project to get started
                </div>
              </div>
            )}
          </>
        ) : (
          /* Collapsed view */
          <div className="p-2 space-y-2">
            <button
              onClick={handleAddProject}
              className="w-full p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
              title="Add Project"
            >
              +
            </button>
            {currentProject && (
              <div className="w-full p-2 bg-blue-50 rounded-lg" title={getProjectName(currentProject)}>
                <div className="w-2 h-2 bg-blue-600 rounded-full mx-auto"></div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};