import React from 'react';
import { ProjectInfo } from '../lib/tauri-api';
import { useTheme } from '../contexts/ThemeContext';

// Keep the original ProjectCard for the browser
interface ProjectBrowserCardProps {
  project: ProjectInfo;
  onSelect: (path: string) => void;
  isSelected?: boolean;
}

export function ProjectCard({ project, onSelect, isSelected = false }: ProjectBrowserCardProps) {
  const getStatusIcon = () => {
    if (!project.is_valid) return '❌';
    if (!project.vision_exists) return '⚠️';
    return '✅';
  };

  const getStatusText = () => {
    if (!project.is_valid) return 'Invalid Metis project';
    if (!project.vision_exists) return 'Missing vision document';
    return 'Valid Metis project';
  };

  const getStatusColor = () => {
    if (!project.is_valid) return 'text-red-600 bg-red-50 border-red-200';
    if (!project.vision_exists) return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    return 'text-green-600 bg-green-50 border-green-200';
  };

  const getProjectName = () => {
    const parts = project.path.split(/[/\\]/);
    return parts[parts.length - 1] || project.path;
  };

  return (
    <div
      className={`
        p-4 border rounded-lg cursor-pointer transition-all
        ${isSelected 
          ? 'border-blue-500 bg-blue-50' 
          : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
        }
        ${!project.is_valid ? 'opacity-60' : ''}
      `}
      onClick={() => onSelect(project.path)}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <h3 className="text-lg font-medium text-gray-900 truncate">
            {getProjectName()}
          </h3>
          <p className="text-sm text-gray-500 truncate mt-1" title={project.path}>
            {project.path}
          </p>
        </div>
        <div className="ml-4 flex-shrink-0">
          <span className="text-2xl" title={getStatusText()}>
            {getStatusIcon()}
          </span>
        </div>
      </div>
      
      <div className="mt-3">
        <span
          className={`
            inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border
            ${getStatusColor()}
          `}
        >
          {getStatusText()}
        </span>
      </div>
    </div>
  );
}

// New sidebar project card component
export interface SidebarProjectCardProps {
  project: ProjectInfo;
  isActive: boolean;
  onClick: () => void;
  onRemove: (e: React.MouseEvent) => void;
}

export const SidebarProjectCard: React.FC<SidebarProjectCardProps> = ({
  project,
  isActive,
  onClick,
  onRemove,
}) => {
  const { theme, themeName } = useTheme();
  
  // Debug: log the current theme colors
  console.log('Theme name:', themeName);
  console.log('Background elevated:', theme.colors.background.elevated);
  console.log('Interactive secondary:', theme.colors.interactive.secondary);
  
  const getProjectName = (project: ProjectInfo): string => {
    const parts = project.path.split('/').filter(Boolean);
    return parts[parts.length - 1] || 'Unknown Project';
  };

  const getProjectPath = (project: ProjectInfo): string => {
    const parts = project.path.split('/').filter(Boolean);
    const pathParts = parts.slice(0, -1);
    return pathParts.length > 2 ? `.../${pathParts.slice(-2).join('/')}` : pathParts.join('/') || '/';
  };

  return (
    <div
      className="group relative overflow-hidden rounded-lg transition-all duration-200 cursor-pointer p-3"
      style={{
        backgroundColor: isActive 
          ? theme.colors.interactive.secondary
          : theme.colors.background.elevated,
        border: `1px solid ${isActive 
          ? theme.colors.interactive.primary
          : theme.colors.border.primary}`
      }}
      onClick={onClick}
      onMouseEnter={(e) => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = theme.colors.background.secondary;
          e.currentTarget.style.borderColor = theme.colors.interactive.primary;
        }
      }}
      onMouseLeave={(e) => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = theme.colors.background.elevated;
          e.currentTarget.style.borderColor = theme.colors.border.primary;
        }
      }}
    >
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0 pr-3">
          <h3
            className={`font-medium truncate transition-colors ${
              isActive ? 'text-interactive-primary' : 'text-primary group-hover:text-interactive-primary'
            }`}
            style={{ fontSize: '14px' }}
          >
            {getProjectName(project)}
          </h3>
          <p
            className="text-tertiary truncate mt-1 transition-colors"
            style={{ fontSize: '11px' }}
            title={project.path}
          >
            {getProjectPath(project)}
          </p>
        </div>
        
        <div className="flex items-center gap-2">
          {/* Status indicator */}
          <div className="flex-shrink-0">
            <div
              className={`w-3 h-3 rounded-full transition-colors ${
                isActive ? 'bg-interactive-primary' : 'bg-text-tertiary group-hover:bg-interactive-primary'
              }`}
            />
          </div>
          
          {/* Remove button - always visible */}
          <button
            onClick={onRemove}
            className="w-5 h-5 flex items-center justify-center text-tertiary hover:text-interactive-danger transition-all duration-200"
            title="Remove project"
            style={{ fontSize: '14px', backgroundColor: 'transparent', border: 'none' }}
          >
            ×
          </button>
        </div>
      </div>
    </div>
  );
};