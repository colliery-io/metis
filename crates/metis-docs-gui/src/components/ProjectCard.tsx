import { ProjectInfo } from '../lib/tauri-api';

interface ProjectCardProps {
  project: ProjectInfo;
  onSelect: (path: string) => void;
  isSelected?: boolean;
}

export function ProjectCard({ project, onSelect, isSelected = false }: ProjectCardProps) {
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