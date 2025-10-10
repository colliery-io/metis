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
  const { currentProject, getRecentProjects, loadProject, removeProject } = useProject();
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

  const handleRemoveProject = (e: React.MouseEvent, projectPath: string) => {
    e.stopPropagation();
    removeProject(projectPath);
  };

  const getProjectName = (project: ProjectInfo): string => {
    // Get the directory name (project name)
    const parts = project.path.split('/').filter(Boolean);
    return parts[parts.length - 1] || 'Unknown Project';
  };


  return (
    <div className={`bg-secondary flex flex-col transition-all duration-200 ${
      isExpanded ? 'w-1/5' : 'w-12'
    }`}>
      {/* Collapse/Expand Button */}
      <div className="px-3 py-2 flex justify-end">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="btn btn-ghost btn-sm"
          title={isExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {isExpanded ? '<' : '>'}
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {isExpanded ? (
          <>
            {/* Add Project Button */}
            <div className="px-3 pt-3 pb-2">
              <button
                onClick={handleAddProject}
                className="btn btn-secondary btn-sm w-full"
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
                    <div
                      key={`${project.path}-${index}`}
                      className={`group w-full text-left px-3 py-1.5 text-xs rounded hover:bg-elevated transition-colors flex items-center justify-between ${
                        currentProject?.path === project.path 
                          ? 'bg-interactive-secondary text-interactive-primary font-medium' 
                          : 'text-secondary'
                      }`}
                    >
                      <button
                        onClick={() => handleProjectClick(project)}
                        className="flex-1 text-left"
                      >
                        {getProjectName(project)}
                      </button>
                      <button
                        onClick={(e) => handleRemoveProject(e, project.path)}
                        className="opacity-0 group-hover:opacity-100 ml-2 p-1 text-tertiary hover:text-interactive-danger transition-all"
                        title="Remove project"
                      >
                        ×
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Empty State */}
            {recentProjects.length === 0 && !currentProject && (
              <div className="px-3 py-6 text-center">
                <div className="text-tertiary text-xs">
                  No projects yet
                </div>
                <div className="text-xs text-tertiary mt-1">
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
              className="btn btn-ghost w-full"
              title="Add Project"
            >
              +
            </button>
            {currentProject && (
              <div className="w-full p-2 bg-interactive-secondary rounded-lg" title={getProjectName(currentProject)}>
                <div className="w-2 h-2 bg-interactive-primary rounded-full mx-auto"></div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};