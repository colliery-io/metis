import { DocumentInfo } from './tauri-api';
import { BoardType } from '../components/BoardNavigation';

export interface PhaseConfig {
  key: string;
  title: string;
  description: string;
  emptyMessage: string;
}

export interface BoardConfig {
  id: BoardType;
  title: string;
  description: string;
  phases: PhaseConfig[];
  documentFilter: (doc: DocumentInfo) => boolean;
}

// Phase configurations for each board type
const visionPhases: PhaseConfig[] = [
  {
    key: 'draft',
    title: 'Draft',
    description: 'Vision being developed',
    emptyMessage: 'No visions in draft phase',
  },
  {
    key: 'review',
    title: 'Review',
    description: 'Vision under review',
    emptyMessage: 'No visions under review',
  },
  {
    key: 'published',
    title: 'Published',
    description: 'Active published visions',
    emptyMessage: 'No published visions',
  },
];

const initiativePhases: PhaseConfig[] = [
  {
    key: 'discovery',
    title: 'Discovery',
    description: 'Understanding the problem space',
    emptyMessage: 'No initiatives in discovery',
  },
  {
    key: 'design',
    title: 'Design',
    description: 'Defining the solution approach',
    emptyMessage: 'No initiatives in design',
  },
  {
    key: 'ready',
    title: 'Ready',
    description: 'Ready for task decomposition',
    emptyMessage: 'No initiatives ready',
  },
  {
    key: 'decompose',
    title: 'Decompose',
    description: 'Breaking down into tasks',
    emptyMessage: 'No initiatives being decomposed',
  },
  {
    key: 'active',
    title: 'Active',
    description: 'Tasks being executed',
    emptyMessage: 'No active initiatives',
  },
  {
    key: 'completed',
    title: 'Completed',
    description: 'Initiative outcomes delivered',
    emptyMessage: 'No completed initiatives',
  },
];

const taskPhases: PhaseConfig[] = [
  {
    key: 'todo',
    title: 'Todo',
    description: 'Ready for execution',
    emptyMessage: 'No tasks to do',
  },
  {
    key: 'doing',
    title: 'Doing',
    description: 'Currently being worked on',
    emptyMessage: 'No tasks in progress',
  },
  {
    key: 'completed',
    title: 'Completed',
    description: 'Task deliverables finished',
    emptyMessage: 'No completed tasks',
  },
];

const adrPhases: PhaseConfig[] = [
  {
    key: 'draft',
    title: 'Draft',
    description: 'Decision being proposed',
    emptyMessage: 'No draft decisions',
  },
  {
    key: 'discussion',
    title: 'Discussion',
    description: 'Under stakeholder review',
    emptyMessage: 'No decisions under discussion',
  },
  {
    key: 'decided',
    title: 'Decided',
    description: 'Final decision made',
    emptyMessage: 'No decisions made',
  },
  {
    key: 'superseded',
    title: 'Superseded',
    description: 'Replaced by newer decision',
    emptyMessage: 'No superseded decisions',
  },
];

const backlogPhases: PhaseConfig[] = [
  {
    key: 'general',
    title: 'General',
    description: 'Unassigned work items',
    emptyMessage: 'No general backlog items',
  },
  {
    key: 'bug',
    title: 'Bug',
    description: 'Issues that need fixing',
    emptyMessage: 'No bugs reported',
  },
  {
    key: 'feature',
    title: 'Feature',
    description: 'New functionality requests',
    emptyMessage: 'No feature requests',
  },
  {
    key: 'tech-debt',
    title: 'Tech Debt',
    description: 'Code improvement needs',
    emptyMessage: 'No tech debt items',
  },
];

// Board configurations
export const boardConfigs: BoardConfig[] = [
  {
    id: 'vision',
    title: 'Vision Board',
    description: 'Strategic direction and outcomes',
    phases: visionPhases,
    documentFilter: (doc) => doc.document_type === 'vision',
  },
  {
    id: 'initiative',
    title: 'Initiative Board',
    description: 'Concrete projects and capabilities',
    phases: initiativePhases,
    documentFilter: (doc) => doc.document_type === 'initiative',
  },
  {
    id: 'task',
    title: 'Task Board',
    description: 'Individual work items',
    phases: taskPhases,
    documentFilter: (doc) => doc.document_type === 'task',
  },
  {
    id: 'adr',
    title: 'ADR Board',
    description: 'Architectural decisions',
    phases: adrPhases,
    documentFilter: (doc) => doc.document_type === 'adr',
  },
  {
    id: 'backlog',
    title: 'Backlog Board',
    description: 'Unassigned work items',
    phases: backlogPhases,
    documentFilter: (doc) => {
      // Backlog items might be tasks without parent or have special tags
      // For now, let's identify them by having no parent initiative
      // This logic might need refinement based on actual backend data
      return doc.document_type === 'task' && !doc.filepath.includes('initiatives/');
    },
  },
];

export function getBoardConfig(boardType: BoardType): BoardConfig | undefined {
  return boardConfigs.find(config => config.id === boardType);
}

export function getDocumentsByPhase(documents: DocumentInfo[], boardType: BoardType) {
  const config = getBoardConfig(boardType);
  if (!config) return {};

  const filteredDocuments = documents.filter(config.documentFilter);
  const documentsByPhase: Record<string, DocumentInfo[]> = {};

  // Initialize all phases
  config.phases.forEach(phase => {
    documentsByPhase[phase.key] = [];
  });

  // Categorize documents by phase
  filteredDocuments.forEach(doc => {
    let phaseKey = doc.phase;
    
    // Handle backlog special case - categorize by tags
    if (boardType === 'backlog') {
      // This is a simplified approach - in real implementation, 
      // you'd examine document tags or other metadata
      phaseKey = 'general'; // Default to general for now
    }

    // Ensure phase exists in our configuration
    if (documentsByPhase[phaseKey] !== undefined) {
      documentsByPhase[phaseKey].push(doc);
    } else {
      // If phase not found, put in first phase as fallback
      const firstPhase = config.phases[0]?.key;
      if (firstPhase && documentsByPhase[firstPhase]) {
        documentsByPhase[firstPhase].push(doc);
      }
    }
  });

  return documentsByPhase;
}