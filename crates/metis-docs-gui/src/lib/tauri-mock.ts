/**
 * Mock Tauri API for development and testing without the Tauri runtime.
 * This allows the app to run in a browser for E2E testing.
 */

import type { ProjectInfo, DocumentInfo, DocumentContent, InitializationResult } from './tauri-api';

// Test data
const mockProjects: Record<string, ProjectInfo> = {
  '/test/project': {
    path: '/test/project',
    is_valid: true,
    vision_exists: true,
  },
};

const mockDocuments: DocumentInfo[] = [
  {
    id: '1',
    title: 'Test Vision Document',
    document_type: 'vision',
    short_code: 'TEST-V-0001',
    filepath: 'visions/TEST-V-0001.md',
    phase: 'published',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
  },
  {
    id: '2',
    title: 'Implement Feature X',
    document_type: 'task',
    short_code: 'TEST-T-0001',
    filepath: 'tasks/TEST-T-0001.md',
    phase: 'todo',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
  },
  {
    id: '3',
    title: 'Fix Bug in Login',
    document_type: 'task',
    short_code: 'TEST-T-0002',
    filepath: 'tasks/TEST-T-0002.md',
    phase: 'doing',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
  },
  {
    id: '4',
    title: 'Add Search Feature',
    document_type: 'task',
    short_code: 'TEST-T-0003',
    filepath: 'tasks/TEST-T-0003.md',
    phase: 'completed',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
  },
  {
    id: '5',
    title: 'Use PostgreSQL for Database',
    document_type: 'adr',
    short_code: 'TEST-A-0001',
    filepath: 'adrs/TEST-A-0001.md',
    phase: 'decided',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
  },
];

const mockDocumentContent: Record<string, DocumentContent> = {
  'TEST-V-0001': {
    id: '1',
    title: 'Test Vision Document',
    content: '# Test Vision\n\nThis is a test vision document.',
    frontmatter_json: JSON.stringify({ phase: 'published' }),
  },
  'TEST-T-0001': {
    id: '2',
    title: 'Implement Feature X',
    content: '# Implement Feature X\n\nDescription of the task.',
    frontmatter_json: JSON.stringify({ phase: 'todo' }),
  },
};

// Mock command handlers
type CommandHandler = (args: Record<string, unknown>) => Promise<unknown>;

const commandHandlers: Record<string, CommandHandler> = {
  load_project: async (args) => {
    const path = args.path as string;
    return mockProjects[path] || { path, is_valid: false, vision_exists: false };
  },

  list_documents: async () => {
    return mockDocuments;
  },

  read_document: async (args) => {
    const shortCode = args.shortCode as string;
    return mockDocumentContent[shortCode] || null;
  },

  search_documents: async (args) => {
    const query = (args.query as string).toLowerCase();
    return mockDocuments.filter(
      (doc) =>
        doc.title.toLowerCase().includes(query) ||
        doc.short_code.toLowerCase().includes(query)
    );
  },

  initialize_project: async (args): Promise<InitializationResult> => {
    const path = args.path as string;
    mockProjects[path] = {
      path,
      is_valid: true,
      vision_exists: true,
    };
    return {
      metis_dir: `${path}/.metis`,
      database_path: `${path}/.metis/metis.db`,
      vision_path: `${path}/.metis/visions/PROJ-V-0001.md`,
    };
  },

  transition_phase: async (args) => {
    const shortCode = args.shortCode as string;
    const phase = args.phase as string;
    const doc = mockDocuments.find((d) => d.short_code === shortCode);
    if (doc) {
      doc.phase = phase;
    }
    return { success: true };
  },

  get_project_config: async () => {
    return {
      prefix: 'TEST',
      preset: 'direct',
      strategies_enabled: false,
      initiatives_enabled: false,
    };
  },

  sync_project: async () => {
    return {
      synced_count: mockDocuments.length,
      errors: [],
    };
  },

  get_available_parents: async () => {
    return [];
  },

  create_document: async (args) => {
    const newDoc: DocumentInfo = {
      id: String(mockDocuments.length + 1),
      title: args.title as string,
      document_type: args.documentType as string,
      short_code: `TEST-T-${String(mockDocuments.length + 1).padStart(4, '0')}`,
      filepath: `tasks/TEST-T-${String(mockDocuments.length + 1).padStart(4, '0')}.md`,
      phase: 'todo',
      archived: false,
      created_at: Date.now() / 1000,
      updated_at: Date.now() / 1000,
      tags: [],
    };
    mockDocuments.push(newDoc);
    return newDoc;
  },

  update_document_content: async () => {
    return { success: true };
  },

  archive_document: async (args) => {
    const shortCode = args.shortCode as string;
    const doc = mockDocuments.find((d) => d.short_code === shortCode);
    if (doc) {
      doc.archived = true;
    }
    return { success: true };
  },

  get_app_version: async () => {
    return '1.0.0-test';
  },

  install_cli: async () => {
    return { success: true };
  },
};

// Mock invoke function
export async function mockInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const handler = commandHandlers[cmd];
  if (!handler) {
    console.warn(`[Mock] Unknown command: ${cmd}`);
    throw new Error(`Mock not implemented for command: ${cmd}`);
  }

  // Simulate network delay
  await new Promise((resolve) => setTimeout(resolve, 50));

  const result = await handler(args || {});
  console.log(`[Mock] ${cmd}`, args, '->', result);
  return result as T;
}

// Helper to add test project to mock data
export function addMockProject(path: string, info: Partial<ProjectInfo> = {}) {
  mockProjects[path] = {
    path,
    is_valid: true,
    vision_exists: true,
    ...info,
  };
}

// Helper to add test document
export function addMockDocument(doc: Partial<DocumentInfo>) {
  const newDoc: DocumentInfo = {
    id: String(mockDocuments.length + 1),
    title: 'Untitled',
    document_type: 'task',
    short_code: `TEST-T-${String(mockDocuments.length + 1).padStart(4, '0')}`,
    filepath: '',
    phase: 'todo',
    archived: false,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    tags: [],
    ...doc,
  };
  mockDocuments.push(newDoc);
  return newDoc;
}

// Check if we're running in Tauri
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}
