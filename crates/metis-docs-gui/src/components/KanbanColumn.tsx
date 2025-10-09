import React from 'react';
import { DocumentInfo } from '../lib/tauri-api';
import { DocumentCard } from './DocumentCard';

export interface KanbanColumnProps {
  title: string;
  phase: string;
  documents: DocumentInfo[];
  onDocumentClick?: (document: DocumentInfo) => void;
  emptyMessage?: string;
}

export const KanbanColumn: React.FC<KanbanColumnProps> = ({
  title,
  phase,
  documents,
  onDocumentClick,
  emptyMessage,
}) => {
  const getPhaseColor = (phaseKey: string) => {
    switch (phaseKey) {
      case 'draft':
      case 'todo':
      case 'discovery':
      case 'discussion':
        return 'border-orange-200 bg-orange-50';
      case 'review':
      case 'doing':
      case 'design':
      case 'decompose':
        return 'border-blue-200 bg-blue-50';
      case 'published':
      case 'completed':
      case 'decided':
      case 'ready':
      case 'active':
        return 'border-green-200 bg-green-50';
      case 'superseded':
        return 'border-red-200 bg-red-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  return (
    <div className={`
      flex flex-col h-full min-h-96 rounded-lg border-2 
      ${getPhaseColor(phase)}
    `}>
      {/* Column Header */}
      <div className="p-4 border-b border-gray-200">
        <h3 className="font-semibold text-gray-900 text-lg">
          {title}
        </h3>
        <div className="flex items-center justify-between mt-1">
          <span className="text-sm text-gray-600 capitalize">
            {phase}
          </span>
          <span className="text-sm font-medium text-gray-700 bg-white px-2 py-1 rounded">
            {documents.length}
          </span>
        </div>
      </div>

      {/* Column Content */}
      <div className="flex-1 p-3 space-y-3 overflow-y-auto">
        {documents.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <div className="text-gray-400 mb-2">
              <svg
                className="mx-auto h-8 w-8"
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
            <p className="text-sm">
              {emptyMessage || `No ${phase} documents`}
            </p>
          </div>
        ) : (
          documents.map((doc) => (
            <DocumentCard
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