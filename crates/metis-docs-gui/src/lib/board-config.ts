import { DocumentInfo } from './tauri-api';
import { BoardType } from '../types/board';

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
    key: 'blocked',
    title: 'Blocked',
    description: 'Waiting on dependencies',
    emptyMessage: 'No blocked tasks',
  },
  {
    key: 'todo',
    title: 'Todo',
    description: 'Ready for execution',
    emptyMessage: 'No tasks to do',
  },
  {
    key: 'active',
    title: 'Active',
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
    documentFilter: (doc) => doc.document_type === 'task' && (
      // Has a parent (assigned to initiative) OR has been picked up (todo/active/blocked/completed phase)
      (doc as any).parent || ['todo', 'active', 'blocked', 'completed'].includes(doc.phase)
    ),
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
    documentFilter: (doc) => doc.document_type === 'task' && (
      // No parent AND phase is backlog (or not picked up to todo/active/blocked/completed yet)
      !(doc as any).parent && (doc.phase === 'backlog' || !['todo', 'active', 'blocked', 'completed'].includes(doc.phase))
    ),
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
    
    // Handle backlog special case - categorize by tags or phase
    if (boardType === 'backlog') {
      // Check document tags to determine backlog category
      const tags = (doc as any).tags || [];
      if (tags.includes('#bug')) {
        phaseKey = 'bug';
      } else if (tags.includes('#feature')) {
        phaseKey = 'feature';
      } else if (tags.includes('#tech-debt')) {
        phaseKey = 'tech-debt';
      } else {
        // Default to general for items without type tags
        phaseKey = 'general';
      }
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