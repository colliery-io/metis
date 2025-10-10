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
}) => {
  const { currentProject, getRecentProjects, loadProject } = useProject();
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
    // Get the directory name (project name)
    const parts = project.path.split('/').filter(Boolean);
    return parts[parts.length - 1] || 'Unknown Project';
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
                className="w-full px-3 py-2 text-sm text-gray-700 border border-gray-300 rounded hover:bg-gray-50 transition-colors"
              >
                Add Project
              </button>
            </div>


            {/* Project List */}
            {recentProjects.length > 0 && (
              <div className="px-3">
                <div className="space-y-1">
                  {recentProjects
                    .slice(0, 8)
                    .map((project, index) => (
                    <button
                      key={`${project.path}-${index}`}
                      onClick={() => handleProjectClick(project)}
                      className={`w-full text-left px-3 py-2 text-sm rounded hover:bg-gray-100 transition-colors ${
                        currentProject?.path === project.path 
                          ? 'bg-blue-50 text-blue-700 border-l-2 border-blue-500' 
                          : 'text-gray-700'
                      }`}
                    >
                      {getProjectName(project)}
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