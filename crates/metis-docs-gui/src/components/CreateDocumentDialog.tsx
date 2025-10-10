import React, { useState } from 'react';
import { createDocument, CreateDocumentRequest } from '../lib/tauri-api';
import { BoardType } from './BoardNavigation';

export interface CreateDocumentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  boardType: BoardType;
  onDocumentCreated: () => void;
}

export const CreateDocumentDialog: React.FC<CreateDocumentDialogProps> = ({
  isOpen,
  onClose,
  boardType,
  onDocumentCreated,
}) => {
  const [title, setTitle] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!title.trim()) {
      setError('Title is required');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const request: CreateDocumentRequest = {
        document_type: boardType,
        title: title.trim(),
      };

      await createDocument(request);
      
      // Reset form and close dialog
      setTitle('');
      onDocumentCreated();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create document');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setTitle('');
    setError(null);
    onClose();
  };

  if (!isOpen) return null;

  const getDocumentTypeLabel = (type: BoardType) => {
    switch (type) {
      case 'vision':
        return 'Vision';
      case 'initiative':
        return 'Initiative';
      case 'task':
        return 'Task';
      case 'adr':
        return 'ADR (Architectural Decision Record)';
      case 'backlog':
        return 'Backlog Item';
      default:
        return 'Document';
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">
            Create New {getDocumentTypeLabel(boardType)}
          </h2>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-gray-600 text-xl font-bold"
          >
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label htmlFor="title" className="block text-sm font-medium text-gray-700 mb-2">
              Title
            </label>
            <input
              type="text"
              id="title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder={`Enter ${getDocumentTypeLabel(boardType).toLowerCase()} title...`}
              disabled={loading}
            />
          </div>

          {error && (
            <div className="mb-4 text-red-600 text-sm">
              {error}
            </div>
          )}

          <div className="flex justify-end space-x-3">
            <button
              type="button"
              onClick={handleClose}
              className="px-4 py-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-md transition-colors"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              disabled={loading || !title.trim()}
            >
              {loading ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};