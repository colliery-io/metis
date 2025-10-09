import React, { useState, useEffect } from 'react';
import { useProject } from '../contexts/ProjectContext';
import { listDocuments, MetisAPI, DocumentInfo } from '../lib/tauri-api';
import { DocumentCard } from './DocumentCard';
import { DocumentTypeFilter } from './DocumentTypeFilter';
import { EmptyState } from './EmptyState';

// Use DocumentInfo from tauri-api instead of defining our own
export type Document = DocumentInfo;

export interface DocumentBoardProps {
  onBackToProjects: () => void;
}

export const DocumentBoard: React.FC<DocumentBoardProps> = ({ onBackToProjects }) => {
  const { currentProject } = useProject();
  const [documents, setDocuments] = useState<Document[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<string>('all');

  useEffect(() => {
    if (!currentProject?.path) return;

    const loadDocuments = async () => {
      try {
        setLoading(true);
        setError(null);
        
        // First ensure the project is loaded in the backend
        await MetisAPI.loadProject(currentProject.path);
        
        // Then get the documents
        const docs = await listDocuments();
        setDocuments(docs);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load documents');
      } finally {
        setLoading(false);
      }
    };

    loadDocuments();
  }, [currentProject?.path]);

  const filteredDocuments = documents.filter(doc => {
    if (selectedType === 'all') return true;
    return doc.document_type === selectedType;
  });

  const groupDocumentsByPhase = (docs: Document[]) => {
    const groups: Record<string, Document[]> = {};
    docs.forEach(doc => {
      const phase = doc.phase || 'unknown';
      if (!groups[phase]) groups[phase] = [];
      groups[phase].push(doc);
    });
    return groups;
  };

  const documentsByPhase = groupDocumentsByPhase(filteredDocuments);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="text-gray-600">Loading documents...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="text-red-800 font-medium">Error loading documents</div>
        <div className="text-red-600 text-sm mt-1">{error}</div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Documents</h1>
          <p className="text-gray-600 text-sm mt-1">
            {currentProject?.path}
          </p>
        </div>
        <button
          onClick={onBackToProjects}
          className="px-4 py-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors"
        >
          ← Back to Projects
        </button>
      </div>

      {/* Document Type Filter */}
      <DocumentTypeFilter
        selectedType={selectedType}
        onTypeChange={setSelectedType}
        documentCounts={{
          all: documents.length,
          vision: documents.filter(d => d.document_type === 'vision').length,
          initiative: documents.filter(d => d.document_type === 'initiative').length,
          task: documents.filter(d => d.document_type === 'task').length,
          adr: documents.filter(d => d.document_type === 'adr').length,
        }}
      />

      {/* Document Board */}
      {filteredDocuments.length === 0 ? (
        <EmptyState selectedType={selectedType} />
      ) : (
        <div className="grid gap-6">
          {Object.entries(documentsByPhase).map(([phase, phaseDocuments]) => (
            <div key={phase} className="bg-white rounded-lg border border-gray-200 p-4">
              <h3 className="font-medium text-gray-900 mb-3 capitalize">
                {phase} ({phaseDocuments.length})
              </h3>
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {phaseDocuments.map(doc => (
                  <DocumentCard key={doc.short_code} document={doc} />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};