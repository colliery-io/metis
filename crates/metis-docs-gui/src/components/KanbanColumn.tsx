import React from 'react';
import { useDroppable } from '@dnd-kit/core';
import { DocumentInfo } from '../lib/tauri-api';
import { DraggableDocumentCard } from './DraggableDocumentCard';
import { useTheme } from '../contexts/ThemeContext';

export interface KanbanColumnProps {
  title: string;
  phase: string;
  documents: DocumentInfo[];
  onDocumentClick?: (document: DocumentInfo) => void;
  emptyMessage?: string;
  isDragging?: boolean;
}

export const KanbanColumn: React.FC<KanbanColumnProps> = ({
  title,
  phase,
  documents,
  onDocumentClick,
  emptyMessage,
  isDragging = false,
}) => {
  const { theme } = useTheme();
  const { isOver, setNodeRef } = useDroppable({
    id: phase,
  });

  const getPhaseStyle = () => {
    return {
      backgroundColor: isOver ? theme.colors.interactive.secondary : theme.colors.background.secondary,
      borderColor: isOver ? theme.colors.interactive.primary : theme.colors.border.primary,
    };
  };

  const phaseStyle = getPhaseStyle();

  return (
    <div 
      ref={setNodeRef}
      className={`flex flex-col h-full min-h-96 rounded-lg border-2 transition-all duration-200 ${
        isOver ? 'scale-105' : ''
      }`}
      style={{
        backgroundColor: phaseStyle.backgroundColor,
        borderColor: phaseStyle.borderColor,
        opacity: isDragging && !isOver ? 0.7 : 1,
      }}
    >
      {/* Column Header */}
      <div 
        className="p-4 border-b"
        style={{ borderColor: theme.colors.border.primary }}
      >
        <h3 
          className="font-semibold text-lg"
          style={{ color: theme.colors.text.primary }}
        >
          {title}
        </h3>
        <div className="flex items-center justify-between mt-1">
          <span 
            className="text-sm capitalize"
            style={{ color: theme.colors.text.secondary }}
          >
            {phase}
          </span>
          <span 
            className="text-sm font-medium px-2 py-1 rounded"
            style={{
              color: theme.colors.text.primary,
              backgroundColor: theme.colors.background.elevated,
            }}
          >
            {documents.length}
          </span>
        </div>
      </div>

      {/* Column Content */}
      <div className="flex-1 p-3 space-y-3 overflow-y-auto">
        {documents.length === 0 ? (
          <div 
            className="text-center py-8"
            style={{ color: theme.colors.text.tertiary }}
          >
            <div className="mb-2">
              <svg
                className="mx-auto h-8 w-8"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                style={{ color: theme.colors.text.tertiary }}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1}
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
            </div>
            <p className="text-sm">
              {emptyMessage || `No ${phase} documents`}
            </p>
          </div>
        ) : (
          documents.map((doc) => (
            <DraggableDocumentCard
              key={doc.short_code}
              document={doc}
              onClick={onDocumentClick}
            />
          ))
        )}
      </div>
    </div>
  );
};