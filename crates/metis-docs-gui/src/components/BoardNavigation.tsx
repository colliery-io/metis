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
      count: documentCounts?.vision || 0,
    },
    {
      id: 'initiative' as BoardType,
      label: 'Initiative',
      description: 'Concrete projects and capabilities',
      count: documentCounts?.initiative || 0,
    },
    {
      id: 'task' as BoardType,
      label: 'Task',
      description: 'Individual work items',
      count: documentCounts?.task || 0,
    },
    {
      id: 'adr' as BoardType,
      label: 'ADR',
      description: 'Architectural decisions',
      count: documentCounts?.adr || 0,
    },
    {
      id: 'backlog' as BoardType,
      label: 'Backlog',
      description: 'Unassigned work items',
      count: documentCounts?.backlog || 0,
    },
  ];

  return (
    <div className="border-b border-primary bg-elevated">
      <div className="px-6 py-4">
        <nav className="flex space-x-1">
          {boardConfigs.map((board) => (
            <button
              key={board.id}
              onClick={() => onBoardChange(board.id)}
              className={`
                group flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-colors
                ${currentBoard === board.id
                  ? 'bg-interactive-secondary text-interactive-primary border border-interactive-primary'
                  : 'text-secondary hover:text-primary hover:bg-secondary'
                }
              `}
              title={board.description}
            >
              <span>{board.label}</span>
              {board.count > 0 && (
                <span
                  className={`
                    ml-2 px-2 py-0.5 text-xs rounded-full
                    ${currentBoard === board.id
                      ? 'bg-interactive-primary text-inverse'
                      : 'bg-tertiary text-secondary group-hover:bg-secondary'
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