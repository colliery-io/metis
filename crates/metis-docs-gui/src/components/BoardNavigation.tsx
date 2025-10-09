import React from 'react';

export type BoardType = 'vision' | 'initiative' | 'task' | 'adr' | 'backlog';

export interface BoardNavigationProps {
  currentBoard: BoardType;
  onBoardChange: (board: BoardType) => void;
  documentCounts?: {
    vision: number;
    initiative: number;
    task: number;
    adr: number;
    backlog: number;
  };
}

export const BoardNavigation: React.FC<BoardNavigationProps> = ({
  currentBoard,
  onBoardChange,
  documentCounts,
}) => {
  const boardConfigs = [
    {
      id: 'vision' as BoardType,
      label: 'Vision',
      description: 'Strategic direction and outcomes',
      icon: '🎯',
      count: documentCounts?.vision || 0,
    },
    {
      id: 'initiative' as BoardType,
      label: 'Initiative',
      description: 'Concrete projects and capabilities',
      icon: '🚀',
      count: documentCounts?.initiative || 0,
    },
    {
      id: 'task' as BoardType,
      label: 'Task',
      description: 'Individual work items',
      icon: '✅',
      count: documentCounts?.task || 0,
    },
    {
      id: 'adr' as BoardType,
      label: 'ADR',
      description: 'Architectural decisions',
      icon: '📋',
      count: documentCounts?.adr || 0,
    },
    {
      id: 'backlog' as BoardType,
      label: 'Backlog',
      description: 'Unassigned work items',
      icon: '📝',
      count: documentCounts?.backlog || 0,
    },
  ];

  return (
    <div className="border-b border-gray-200 bg-white">
      <div className="px-6 py-4">
        <nav className="flex space-x-1">
          {boardConfigs.map((board) => (
            <button
              key={board.id}
              onClick={() => onBoardChange(board.id)}
              className={`
                group flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-colors
                ${currentBoard === board.id
                  ? 'bg-blue-100 text-blue-800 border border-blue-200'
                  : 'text-gray-600 hover:text-gray-800 hover:bg-gray-50'
                }
              `}
              title={board.description}
            >
              <span className="mr-2 text-base">{board.icon}</span>
              <span>{board.label}</span>
              {board.count > 0 && (
                <span
                  className={`
                    ml-2 px-2 py-0.5 text-xs rounded-full
                    ${currentBoard === board.id
                      ? 'bg-blue-200 text-blue-800'
                      : 'bg-gray-200 text-gray-600 group-hover:bg-gray-300'
                    }
                  `}
                >
                  {board.count}
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>
    </div>
  );
};