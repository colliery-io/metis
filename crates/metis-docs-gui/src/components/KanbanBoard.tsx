import React, { useState, useEffect } from 'react';
import { useProject } from '../contexts/ProjectContext';
import { listDocuments, MetisAPI, DocumentInfo } from '../lib/tauri-api';
import { BoardNavigation, BoardType } from './BoardNavigation';
import { KanbanColumn } from './KanbanColumn';
import { CreateDocumentDialog } from './CreateDocumentDialog';
import { DocumentEditor } from './DocumentEditor';
import { getBoardConfig, getDocumentsByPhase } from '../lib/board-config';

export interface KanbanBoardProps {
  onBackToProjects: () => void;
}

export const KanbanBoard: React.FC<KanbanBoardProps> = ({ onBackToProjects }) => {
  const { currentProject } = useProject();
  const [documents, setDocuments] = useState<DocumentInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentBoard, setCurrentBoard] = useState<BoardType>('vision');
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingDocument, setEditingDocument] = useState<DocumentInfo | null>(null);

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

  // Calculate document counts for navigation
  const documentCounts = {
    vision: documents.filter(d => d.document_type === 'vision').length,
    initiative: documents.filter(d => d.document_type === 'initiative').length,
    task: documents.filter(d => d.document_type === 'task').length,
    adr: documents.filter(d => d.document_type === 'adr').length,
    backlog: documents.filter(d => 
      d.document_type === 'task' && !d.filepath.includes('initiatives/')
    ).length,
  };

  const handleDocumentClick = (document: DocumentInfo) => {
    setEditingDocument(document);
  };

  const handleDocumentCreated = () => {
    // Reload documents after creation
    if (!currentProject?.path) return;
    
    const loadDocuments = async () => {
      try {
        setLoading(true);
        setError(null);
        
        await MetisAPI.loadProject(currentProject.path);
        const docs = await listDocuments();
        setDocuments(docs);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to reload documents');
      } finally {
        setLoading(false);
      }
    };

    loadDocuments();
  };

  const handleDocumentUpdated = () => {
    // Reload documents after update
    handleDocumentCreated();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="text-secondary">Loading documents...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-secondary border border-error rounded-lg p-4 m-6">
        <div className="text-interactive-danger font-medium">Error loading documents</div>
        <div className="text-interactive-danger text-sm mt-1">{error}</div>
      </div>
    );
  }

  const boardConfig = getBoardConfig(currentBoard);
  const documentsByPhase = getDocumentsByPhase(documents, currentBoard);

  if (!boardConfig) {
    return (
      <div className="p-6">
        <div className="text-interactive-danger">Invalid board configuration</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-6 bg-elevated border-b border-primary">
        <div>
          <h1 className="text-2xl font-bold text-primary">
            {boardConfig.title}
          </h1>
          <p className="text-secondary text-sm mt-1">
            {currentProject?.path} • {boardConfig.description}
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => setShowCreateDialog(true)}
            className="btn-primary px-4 py-2 rounded-lg transition-colors"
          >
            + Create {boardConfig.title.slice(0, -1)}
          </button>
          <button
            onClick={onBackToProjects}
            className="btn-secondary px-4 py-2 rounded-lg transition-colors"
          >
            ← Back to Projects
          </button>
        </div>
      </div>

      {/* Board Navigation */}
      <BoardNavigation
        currentBoard={currentBoard}
        onBoardChange={setCurrentBoard}
        documentCounts={documentCounts}
      />

      {/* Kanban Columns */}
      <div className="flex-1 p-6 overflow-hidden">
        <div className="h-full flex gap-4 overflow-x-auto">
          {boardConfig.phases.map((phase) => (
            <div key={phase.key} className="flex-shrink-0 w-80">
              <KanbanColumn
                title={phase.title}
                phase={phase.key}
                documents={documentsByPhase[phase.key] || []}
                onDocumentClick={handleDocumentClick}
                emptyMessage={phase.emptyMessage}
              />
            </div>
          ))}
        </div>
      </div>

      {/* Create Document Dialog */}
      <CreateDocumentDialog
        isOpen={showCreateDialog}
        onClose={() => setShowCreateDialog(false)}
        boardType={currentBoard}
        onDocumentCreated={handleDocumentCreated}
      />

      {/* Document Editor */}
      <DocumentEditor
        isOpen={editingDocument !== null}
        onClose={() => setEditingDocument(null)}
        document={editingDocument}
        onDocumentUpdated={handleDocumentUpdated}
      />
    </div>
  );
};