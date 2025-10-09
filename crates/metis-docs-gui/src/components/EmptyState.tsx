import React from 'react';

export interface EmptyStateProps {
  selectedType: string;
}

export const EmptyState: React.FC<EmptyStateProps> = ({ selectedType }) => {
  const getEmptyMessage = (type: string) => {
    switch (type) {
      case 'vision':
        return {
          title: 'No visions found',
          description: 'Create a vision to define the strategic direction for your project.',
          suggestion: 'Visions capture the WHY and WHERE of your work.',
        };
      case 'initiative':
        return {
          title: 'No initiatives found',
          description: 'Create initiatives to break down your vision into concrete projects.',
          suggestion: 'Initiatives deliver specific capabilities that advance your strategy.',
        };
      case 'task':
        return {
          title: 'No tasks found',
          description: 'Create tasks to organize the specific work that needs to be done.',
          suggestion: 'Tasks are individual work items that contribute to initiative delivery.',
        };
      case 'adr':
        return {
          title: 'No ADRs found',
          description: 'Create Architectural Decision Records to document important decisions.',
          suggestion: 'ADRs capture significant technical and architectural choices.',
        };
      default:
        return {
          title: 'No documents found',
          description: 'This project doesn\'t have any documents yet.',
          suggestion: 'Start by creating a vision to define your project\'s direction.',
        };
    }
  };

  const message = getEmptyMessage(selectedType);

  return (
    <div className="text-center py-12">
      <div className="max-w-md mx-auto">
        <div className="text-gray-400 mb-4">
          <svg
            className="mx-auto h-12 w-12"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
        </div>
        <h3 className="text-lg font-medium text-gray-900 mb-2">
          {message.title}
        </h3>
        <p className="text-gray-600 mb-1">
          {message.description}
        </p>
        <p className="text-sm text-gray-500">
          {message.suggestion}
        </p>
      </div>
    </div>
  );
};