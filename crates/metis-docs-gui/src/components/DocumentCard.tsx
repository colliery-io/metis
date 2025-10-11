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
        rounded-lg p-3 transition-all duration-200
        ${onClick ? 'cursor-pointer hover:scale-105 hover:shadow-md' : ''}
      `}
      style={{
        backgroundColor: theme.colors.background.elevated,
        border: `1px solid ${theme.colors.border.primary}`,
      }}
      onClick={() => onClick?.(document)}
      onMouseEnter={(e) => {
        if (onClick) {
          e.currentTarget.style.borderColor = theme.colors.interactive.primary;
        }
      }}
      onMouseLeave={(e) => {
        if (onClick) {
          e.currentTarget.style.borderColor = theme.colors.border.primary;
        }
      }}
    >
      {/* Header with short code */}
      <div className="flex items-center justify-between mb-2">
        <span
          className="text-xs font-mono font-medium"
          style={{ color: theme.colors.text.tertiary }}
        >
          {document.short_code}
        </span>
        <span
          className="px-1.5 py-0.5 text-xs font-medium rounded"
          style={typeStyle}
        >
          {document.document_type.charAt(0).toUpperCase()}
        </span>
      </div>

      {/* Title */}
      <h4 
        className="font-medium mb-2 text-sm leading-tight"
        style={{ 
          color: theme.colors.text.primary,
          display: '-webkit-box',
          WebkitLineClamp: 2,
          WebkitBoxOrient: 'vertical',
          overflow: 'hidden',
        }}
      >
        {document.title}
      </h4>

      {/* Phase */}
      {document.phase && (
        <div className="mb-2">
          <span
            className="px-2 py-1 rounded text-xs font-medium"
            style={phaseStyle}
          >
            {document.phase}
          </span>
        </div>
      )}

      {/* Date */}
      <div className="text-xs" style={{ color: theme.colors.text.tertiary }}>
        {formatDate(document.updated_at)}
      </div>
    </div>
  );
};