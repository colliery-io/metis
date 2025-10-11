import React, { useState } from 'react';
import { useProject } from '../contexts/ProjectContext';
import { ProjectInfo, MetisAPI } from '../lib/tauri-api';
import { open } from '@tauri-apps/plugin-dialog';
import { SidebarProjectCard } from './ProjectCard';
import { InitProjectDialog } from './InitProjectDialog';

export interface ProjectSidebarProps {
  onProjectSelect: (project: ProjectInfo) => void;
  onShowProjectBrowser: () => void;
}

export const ProjectSidebar: React.FC<ProjectSidebarProps> = ({
  onProjectSelect,
}) => {
  const { currentProject, getRecentProjects, loadProject, removeProject } = useProject();
  const [isExpanded, setIsExpanded] = useState(true);
  const [showInitDialog, setShowInitDialog] = useState(false);
  const [selectedPath, setSelectedPath] = useState<string>('');
  const [directoryName, setDirectoryName] = useState('');
  const recentProjects = getRecentProjects();

  const handleAddProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Directory for Metis Project',
      });
      
      if (selected && typeof selected === 'string') {
        // First, try to load the project to see if it's already a valid Metis project
        try {
          const projectInfo = await MetisAPI.loadProject(selected);
          
          if (projectInfo.is_valid) {
            // Already a valid Metis project, load it normally
            await loadProject(selected);
          } else {
            // Not a valid Metis project, offer to initialize it
            const pathParts = selected.split('/').filter(Boolean);
            const projectName = pathParts[pathParts.length - 1] || 'unknown';
            setSelectedPath(selected);
            setDirectoryName(projectName);
            setShowInitDialog(true);
          }
        } catch (loadError) {
          console.error('Failed to check/load project:', loadError);
          // If loading fails, still offer to initialize
          const pathParts = selected.split('/').filter(Boolean);
          const projectName = pathParts[pathParts.length - 1] || 'unknown';
          setSelectedPath(selected);
          setDirectoryName(projectName);
          setShowInitDialog(true);
        }
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

  const handleInitConfirm = async (prefix: string) => {
    setShowInitDialog(false);
    
    try {
      // Initialize the project with the user-provided prefix
      const initResult = await MetisAPI.initializeProject(selectedPath, prefix);
      console.log('Project initialized:', initResult);
      
      // Now load the newly initialized project
      await loadProject(selectedPath);
    } catch (initError) {
      console.error('Failed to initialize project:', initError);
      alert('Failed to initialize Metis project. Please check the directory permissions and try again.');
    }
  };

  const handleInitCancel = () => {
    setShowInitDialog(false);
    setSelectedPath('');
    setDirectoryName('');
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
                <div className="space-y-2">
                  {recentProjects
                    .slice(0, 8)
                    .map((project, index) => (
                    <SidebarProjectCard
                      key={`${project.path}-${index}`}
                      project={project}
                      isActive={currentProject?.path === project.path}
                      onClick={() => handleProjectClick(project)}
                      onRemove={(e) => handleRemoveProject(e, project.path)}
                    />
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

      {/* Custom initialization dialog */}
      <InitProjectDialog
        isOpen={showInitDialog}
        directoryName={directoryName}
        onConfirm={handleInitConfirm}
        onCancel={handleInitCancel}
      />
    </div>
  );
};