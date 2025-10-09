import React from 'react';
import type { Document } from './DocumentBoard';

export interface DocumentCardProps {
  document: Document;
  onClick?: (document: Document) => void;
}

export const DocumentCard: React.FC<DocumentCardProps> = ({ document, onClick }) => {
  const getTypeColor = (type: string) => {
    switch (type) {
      case 'vision':
        return 'bg-purple-100 text-purple-800 border-purple-200';
      case 'initiative':
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'task':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'adr':
        return 'bg-orange-100 text-orange-800 border-orange-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const getPhaseColor = (phase?: string) => {
    switch (phase) {
      case 'draft':
      case 'todo':
        return 'bg-gray-100 text-gray-700';
      case 'review':
      case 'doing':
      case 'active':
        return 'bg-yellow-100 text-yellow-800';
      case 'published':
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'decided':
        return 'bg-blue-100 text-blue-800';
      case 'superseded':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const formatDate = (timestamp: number) => {
    try {
      return new Date(timestamp * 1000).toLocaleDateString();
    } catch {
      return 'Invalid date';
    }
  };

  return (
    <div
      className={`
        border rounded-lg p-4 bg-white hover:shadow-md transition-shadow
        ${onClick ? 'cursor-pointer hover:border-blue-300' : ''}
      `}
      onClick={() => onClick?.(document)}
    >
      {/* Header with type and short code */}
      <div className="flex items-center justify-between mb-2">
        <span
          className={`
            px-2 py-1 text-xs font-medium rounded border
            ${getTypeColor(document.document_type)}
          `}
        >
          {document.document_type.toUpperCase()}
        </span>
        <span className="text-xs font-mono text-gray-500">
          {document.short_code}
        </span>
      </div>

      {/* Title */}
      <h4 className="font-medium text-gray-900 mb-2 line-clamp-2">
        {document.title}
      </h4>

      {/* Phase and date */}
      <div className="flex items-center justify-between text-xs">
        {document.phase && (
          <span
            className={`
              px-2 py-1 rounded font-medium
              ${getPhaseColor(document.phase)}
            `}
          >
            {document.phase}
          </span>
        )}
        <span className="text-gray-500">
          {formatDate(document.updated_at)}
        </span>
      </div>
    </div>
  );
};