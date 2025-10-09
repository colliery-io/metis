import React from 'react';

export interface DocumentCounts {
  all: number;
  vision: number;
  initiative: number;
  task: number;
  adr: number;
}

export interface DocumentTypeFilterProps {
  selectedType: string;
  onTypeChange: (type: string) => void;
  documentCounts: DocumentCounts;
}

export const DocumentTypeFilter: React.FC<DocumentTypeFilterProps> = ({
  selectedType,
  onTypeChange,
  documentCounts,
}) => {
  const filterOptions = [
    { id: 'all', label: 'All Documents', count: documentCounts.all },
    { id: 'vision', label: 'Visions', count: documentCounts.vision },
    { id: 'initiative', label: 'Initiatives', count: documentCounts.initiative },
    { id: 'task', label: 'Tasks', count: documentCounts.task },
    { id: 'adr', label: 'ADRs', count: documentCounts.adr },
  ];

  return (
    <div className="mb-6">
      <div className="flex flex-wrap gap-2">
        {filterOptions.map(option => (
          <button
            key={option.id}
            onClick={() => onTypeChange(option.id)}
            className={`
              px-4 py-2 rounded-lg text-sm font-medium transition-colors
              ${selectedType === option.id
                ? 'bg-blue-100 text-blue-800 border border-blue-200'
                : 'bg-white text-gray-700 border border-gray-200 hover:bg-gray-50'
              }
            `}
          >
            {option.label}
            {option.count > 0 && (
              <span className="ml-2 text-xs opacity-75">
                ({option.count})
              </span>
            )}
          </button>
        ))}
      </div>
    </div>
  );
};