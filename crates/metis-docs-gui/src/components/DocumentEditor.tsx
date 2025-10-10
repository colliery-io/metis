import React, { useState, useEffect } from 'react';
import { DocumentInfo, DocumentContent, readDocument, updateDocument } from '../lib/tauri-api';
import { TiptapEditor } from './TiptapEditor';

export interface DocumentEditorProps {
  isOpen: boolean;
  onClose: () => void;
  document: DocumentInfo | null;
  onDocumentUpdated?: () => void;
}

export const DocumentEditor: React.FC<DocumentEditorProps> = ({
  isOpen,
  onClose,
  document,
  onDocumentUpdated,
}) => {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [documentContent, setDocumentContent] = useState<DocumentContent | null>(null);

  useEffect(() => {
    if (!document || !isOpen) return;

    const loadDocument = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const docContent = await readDocument(document.short_code);
        setDocumentContent(docContent);
        setContent(docContent.content || '');
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load document');
      } finally {
        setLoading(false);
      }
    };

    loadDocument();
  }, [document, isOpen]);

  const handleSave = async () => {
    if (!document) return;

    try {
      setSaving(true);
      setError(null);
      
      await updateDocument(document.short_code, content);
      
      if (onDocumentUpdated) {
        onDocumentUpdated();
      }
      
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save document');
    } finally {
      setSaving(false);
    }
  };

  const handleClose = () => {
    setContent('');
    setDocumentContent(null);
    setError(null);
    onClose();
  };

  if (!isOpen || !document) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full h-full mx-4 my-4 flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div className="flex-1">
            <h2 className="text-xl font-semibold text-gray-900">
              Edit Document: {document.title}
            </h2>
            <p className="text-sm text-gray-600 mt-1">
              {document.short_code} • {document.document_type} • {document.phase}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={handleSave}
              disabled={saving || loading}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {saving ? 'Saving...' : 'Save'}
            </button>
            <button
              onClick={handleClose}
              className="px-4 py-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {loading ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-gray-600">Loading document...</div>
            </div>
          ) : error ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="bg-red-50 border border-red-200 rounded-lg p-4 max-w-md">
                <div className="text-red-800 font-medium">Error loading document</div>
                <div className="text-red-600 text-sm mt-1">{error}</div>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex flex-col overflow-hidden">
              {/* Document metadata */}
              {documentContent && (
                <div className="p-4 bg-gray-50 border-b border-gray-200">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="font-medium text-gray-700">Title:</span>
                      <span className="ml-2 text-gray-900">{documentContent.title}</span>
                    </div>
                    <div>
                      <span className="font-medium text-gray-700">Type:</span>
                      <span className="ml-2 text-gray-900">{document.document_type}</span>
                    </div>
                    <div>
                      <span className="font-medium text-gray-700">Phase:</span>
                      <span className="ml-2 text-gray-900">{document.phase}</span>
                    </div>
                    <div>
                      <span className="font-medium text-gray-700">Short Code:</span>
                      <span className="ml-2 text-gray-900">{document.short_code}</span>
                    </div>
                  </div>
                </div>
              )}

              {/* Editor */}
              <div className="flex-1 border border-gray-200 m-4 rounded-lg overflow-hidden">
                <TiptapEditor
                  content={content}
                  onChange={setContent}
                  placeholder="Start writing your document content..."
                  className="h-full"
                />
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};