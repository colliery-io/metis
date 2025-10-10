import React from 'react';
import type { Document } from './DocumentBoard';
import { useTheme } from '../contexts/ThemeContext';

export interface DocumentCardProps {
  document: Document;
  onClick?: (document: Document) => void;
}

export const DocumentCard: React.FC<DocumentCardProps> = ({ document, onClick }) => {
  const { theme } = useTheme();

  const getTypeColor = (type: string) => {
    const color = theme.colors.documentType[type as keyof typeof theme.colors.documentType] || theme.colors.documentType.backlog;
    return {
      backgroundColor: color + '20', // 20% opacity
      color: color,
      borderColor: color + '40', // 40% opacity
    };
  };

  const getPhaseColor = (phase?: string) => {
    switch (phase) {
      case 'draft':
      case 'todo':
        return {
          backgroundColor: theme.colors.status.draft + '20',
          color: theme.colors.status.draft,
        };
      case 'review':
      case 'doing':
      case 'active':
        return {
          backgroundColor: theme.colors.status.active + '20',
          color: theme.colors.status.active,
        };
      case 'published':
      case 'completed':
        return {
          backgroundColor: theme.colors.status.completed + '20',
          color: theme.colors.status.completed,
        };
      case 'decided':
        return {
          backgroundColor: theme.colors.interactive.primary + '20',
          color: theme.colors.interactive.primary,
        };
      case 'superseded':
        return {
          backgroundColor: theme.colors.interactive.danger + '20',
          color: theme.colors.interactive.danger,
        };
      default:
        return {
          backgroundColor: theme.colors.status.draft + '20',
          color: theme.colors.status.draft,
        };
    }
  };

  const formatDate = (timestamp: number) => {
    try {
      return new Date(timestamp * 1000).toLocaleDateString();
    } catch {
      return 'Invalid date';
    }
  };

  const typeStyle = getTypeColor(document.document_type);
  const phaseStyle = getPhaseColor(document.phase);

  return (
    <div
      className={`
        border border-primary rounded-lg p-4 bg-elevated hover:shadow-md transition-shadow
        ${onClick ? 'cursor-pointer hover:border-focus' : ''}
      `}
      onClick={() => onClick?.(document)}
    >
      {/* Header with type and short code */}
      <div className="flex items-center justify-between mb-2">
        <span
          className="px-2 py-1 text-xs font-medium rounded border"
          style={typeStyle}
        >
          {document.document_type.toUpperCase()}
        </span>
        <span className="text-xs font-mono text-tertiary">
          {document.short_code}
        </span>
      </div>

      {/* Title */}
      <h4 className="font-medium text-primary mb-2 line-clamp-2">
        {document.title}
      </h4>

      {/* Phase and date */}
      <div className="flex items-center justify-between text-xs">
        {document.phase && (
          <span
            className="px-2 py-1 rounded font-medium"
            style={phaseStyle}
          >
            {document.phase}
          </span>
        )}
        <span className="text-tertiary">
          {formatDate(document.updated_at)}
        </span>
      </div>
    </div>
  );
};