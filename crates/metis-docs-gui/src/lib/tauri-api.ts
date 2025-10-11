import { invoke } from '@tauri-apps/api/core';

// Types matching the Rust structs
export interface ProjectInfo {
  path: string;
  is_valid: boolean;
  vision_exists: boolean;
}

export interface DocumentInfo {
  id: string;
  title: string;
  document_type: string;
  short_code: string;
  filepath: string;
  phase: string;
  archived: boolean;
  created_at: number;
  updated_at: number;
}

export interface DocumentContent {
  id: string;
  title: string;
  content: string;
  frontmatter_json: string;
}

export interface InitializationResult {
  metis_dir: string;
  database_path: string;
  vision_path: string;
}

// API functions
export class MetisAPI {
  /**
   * Initialize a new Metis project at the given path
   */
  static async initializeProject(
    path: string,
    prefix?: string
  ): Promise<InitializationResult> {
    return invoke('initialize_project', { path, prefix });
  }

  /**
   * Load an existing Metis project
   */
  static async loadProject(path: string): Promise<ProjectInfo> {
    return invoke('load_project', { path });
  }

  /**
   * Get all documents in the current project
   */
  static async listDocuments(): Promise<DocumentInfo[]> {
    return invoke('list_documents');
  }

  /**
   * Read a specific document by its short code
   */
  static async readDocument(shortCode: string): Promise<DocumentContent> {
    return invoke('read_document', { short_code: shortCode });
  }

  /**
   * Search documents by content
   */
  static async searchDocuments(query: string): Promise<DocumentInfo[]> {
    return invoke('search_documents', { query });
  }
}

// Document type helpers
export enum DocumentType {
  Vision = 'vision',
  Strategy = 'strategy', 
  Initiative = 'initiative',
  Task = 'task',
  ADR = 'adr',
}

export enum Phase {
  Draft = 'draft',
  Review = 'review',
  Published = 'published',
  Shaping = 'shaping',
  Design = 'design',
  Ready = 'ready',
  Active = 'active',
  Completed = 'completed',
  Discovery = 'discovery',
  Decompose = 'decompose',
  Todo = 'todo',
  Doing = 'doing',
  Discussion = 'discussion',
  Decided = 'decided',
  Superseded = 'superseded',
}

// Utility functions
export function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString();
}

export function getDocumentTypeIcon(type: string): string {
  switch (type) {
    case 'vision':
      return '🎯';
    case 'strategy':
      return '🎨';
    case 'initiative':
      return '🚀';
    case 'task':
      return '✅';
    case 'adr':
      return '📋';
    default:
      return '📄';
  }
}

export function getPhaseColor(phase: string): string {
  switch (phase) {
    case 'draft':
    case 'shaping':
    case 'discovery':
    case 'todo':
    case 'discussion':
      return 'orange';
    case 'review':
    case 'design':
    case 'decompose':
    case 'doing':
      return 'blue';
    case 'published':
    case 'ready':
    case 'active':
    case 'decided':
      return 'green';
    case 'completed':
    case 'superseded':
      return 'gray';
    default:
      return 'gray';
  }
}

export interface CreateDocumentRequest {
  document_type: string;
  title: string;
  parent_id?: string;
  complexity?: string;
  risk_level?: string;
}

export interface CreateDocumentResult {
  id: string;
  short_code: string;
  filepath: string;
}

// API functions for document CRUD operations
export class DocumentAPI {
  /**
   * Create a new document
   */
  static async createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResult> {
    return invoke('create_document', { request });
  }

  /**
   * Update document content
   */
  static async updateDocument(shortCode: string, content: string): Promise<void> {
    return invoke('update_document', { short_code: shortCode, content });
  }

  /**
   * Delete a document
   */
  static async deleteDocument(shortCode: string): Promise<void> {
    return invoke('delete_document', { short_code: shortCode });
  }

  /**
   * Transition a document to a new phase
   */
  static async transitionPhase(shortCode: string, newPhase?: string): Promise<string> {
    return invoke('transition_phase', { shortCode: shortCode, newPhase: newPhase });
  }
}

// Standalone functions for direct import
export const listDocuments = MetisAPI.listDocuments;
export const readDocument = MetisAPI.readDocument;
export const searchDocuments = MetisAPI.searchDocuments;
export const createDocument = DocumentAPI.createDocument;
export const updateDocument = DocumentAPI.updateDocument;
export const deleteDocument = DocumentAPI.deleteDocument;
export const transitionPhase = DocumentAPI.transitionPhase;