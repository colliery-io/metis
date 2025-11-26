import { test as base, expect, Page } from '@playwright/test';

/**
 * Custom test fixtures for Metis E2E tests.
 * Uses Tauri's built-in mockIPC for intercepting backend calls.
 */

// Test project data that matches the mock
const TEST_PROJECT = {
  path: '/test/project',
  is_valid: true,
  vision_exists: true,
};

// Mock document data
const MOCK_DOCUMENTS = [
  {
    id: '1',
    title: 'Test Vision Document',
    document_type: 'vision',
    short_code: 'TEST-V-0001',
    filepath: 'visions/TEST-V-0001.md',
    phase: 'published',
    archived: false,
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
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
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
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
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
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
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
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
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
    tags: [],
  },
];

// Extended test with custom fixtures
export const test = base.extend<{
  pageWithProject: Page;
}>({
  // Fixture that sets up a test project in localStorage
  pageWithProject: async ({ page }, use) => {
    await page.goto('/');

    // Inject test project into localStorage
    await page.evaluate((project) => {
      localStorage.setItem('metis-recent-projects', JSON.stringify([project]));
    }, TEST_PROJECT);

    // Reload to pick up the localStorage change
    await page.reload();

    // Wait for app to load
    await page.waitForSelector('.home-icon-topbar');

    await use(page);
  },
});

export { expect };

/**
 * Setup Tauri mockIPC in the browser context.
 * This intercepts all invoke() calls and returns mock data.
 */
async function setupTauriMocks(page: Page) {
  await page.addInitScript({
    content: `
      // Mock documents data
      const mockDocuments = ${JSON.stringify(MOCK_DOCUMENTS)};

      const mockDocumentContent = {
        'TEST-V-0001': {
          id: '1',
          title: 'Test Vision Document',
          content: '# Test Vision\\n\\nThis is a test vision document.',
          frontmatter_json: JSON.stringify({ phase: 'published' }),
        },
        'TEST-T-0001': {
          id: '2',
          title: 'Implement Feature X',
          content: '# Implement Feature X\\n\\nDescription of the task.',
          frontmatter_json: JSON.stringify({ phase: 'todo' }),
        },
      };

      // Create fake __TAURI_INTERNALS__ to intercept invoke calls
      window.__TAURI_INTERNALS__ = {
        invoke: async (cmd, args) => {
          // Simulate network delay
          await new Promise(r => setTimeout(r, 50));

          switch (cmd) {
            case 'load_project':
              const path = args?.path;
              if (path === '/test/project') {
                return { path, is_valid: true, vision_exists: true };
              }
              return { path, is_valid: false, vision_exists: false };

            case 'list_documents':
              return mockDocuments;

            case 'read_document':
              return mockDocumentContent[args?.shortCode] || null;

            case 'search_documents':
              const query = (args?.query || '').toLowerCase();
              return mockDocuments.filter(doc =>
                doc.title.toLowerCase().includes(query) ||
                doc.short_code.toLowerCase().includes(query)
              );

            case 'initialize_project':
              return {
                metis_dir: args?.path + '/.metis',
                database_path: args?.path + '/.metis/metis.db',
                vision_path: args?.path + '/.metis/visions/PROJ-V-0001.md',
              };

            case 'transition_phase':
              const doc = mockDocuments.find(d => d.short_code === args?.shortCode);
              if (doc) doc.phase = args?.phase;
              return { success: true };

            case 'get_project_config':
              return {
                prefix: 'TEST',
                preset: 'direct',
                preset_name: 'direct',
                strategies_enabled: false,
                initiatives_enabled: false,
              };

            case 'sync_project':
              return {
                imported: 0,
                updated: 0,
                deleted: 0,
                up_to_date: mockDocuments.length,
                errors: 0,
                messages: [],
              };

            case 'get_available_parents':
              return [];

            case 'create_document':
              const newDoc = {
                id: String(mockDocuments.length + 1),
                short_code: 'TEST-T-' + String(mockDocuments.length + 1).padStart(4, '0'),
                filepath: 'tasks/TEST-T-' + String(mockDocuments.length + 1).padStart(4, '0') + '.md',
              };
              return newDoc;

            case 'update_document':
              return { success: true };

            case 'archive_document':
              return { success: true, total_archived: 1, archived_documents: [] };

            case 'get_app_version':
              return '1.0.0-test';

            default:
              console.warn('[Mock] Unknown command:', cmd);
              throw new Error('Mock not implemented: ' + cmd);
          }
        },
        transformCallback: (callback, once) => {
          // Required for event system
          return 0;
        },
        metadata: {
          currentWindow: { label: 'main' },
          currentWebviewWindow: { label: 'main' },
        },
      };

      // Also set __TAURI__ for environment detection
      window.__TAURI__ = window.__TAURI_INTERNALS__;
    `,
  });
}

// Page object for common operations
export class MetisPage {
  constructor(private page: Page) {}

  // Navigation
  async goto() {
    await setupTauriMocks(this.page);
    await this.page.goto('/');
    await this.page.waitForSelector('.home-icon-topbar');
  }

  // Set up test project
  async setupTestProject() {
    await setupTauriMocks(this.page);
    await this.page.evaluate((project) => {
      localStorage.setItem('metis-recent-projects', JSON.stringify([project]));
    }, TEST_PROJECT);
    await this.page.reload();
    await this.page.waitForSelector('.home-icon-topbar');
  }

  // Load the test project
  async loadTestProject() {
    // Click on the project in the sidebar (uses sidebar-project-card class)
    const projectCard = this.page.locator('.sidebar-project-card').first();
    await projectCard.click();

    // Wait for project to load (search bar becomes visible)
    await this.page.waitForSelector('input[placeholder="Search documents..."]', { timeout: 10000 });
  }

  // Selectors
  get searchInput() {
    return this.page.locator('input[placeholder="Search documents..."]');
  }

  get searchDropdown() {
    return this.page.locator('.search-dropdown');
  }

  get searchResults() {
    return this.page.locator('.search-result-item');
  }

  get themeButton() {
    return this.page.locator('.theme-button');
  }

  get kanbanBoard() {
    return this.page.locator('.kanban-board');
  }

  get kanbanColumns() {
    return this.page.locator('.kanban-column');
  }

  get kanbanCards() {
    return this.page.locator('.kanban-card');
  }

  get boardTabs() {
    return this.page.locator('.board-tab');
  }

  get projectSidebar() {
    return this.page.locator('.w-1\\/5').first();
  }

  get mascotImage() {
    return this.page.locator('.home-icon-main');
  }

  get homeIcon() {
    return this.page.locator('.home-icon-topbar');
  }

  // Actions
  async search(query: string) {
    await this.searchInput.fill(query);
    // Wait for debounce (300ms) + a bit extra
    await this.page.waitForTimeout(400);
  }

  async clearSearch() {
    await this.searchInput.press('Escape');
  }

  async selectSearchResult(index: number) {
    await this.searchResults.nth(index).click();
  }

  async navigateSearchResults(direction: 'up' | 'down') {
    await this.searchInput.press(direction === 'down' ? 'ArrowDown' : 'ArrowUp');
  }

  async selectTheme(theme: 'Light' | 'Dark' | 'Hyper') {
    await this.themeButton.click();
    await this.page.locator('.theme-option', { hasText: theme }).click();
  }

  async goHome() {
    await this.homeIcon.click();
  }

  // Assertions
  async expectHomeScreen() {
    await expect(this.mascotImage).toBeVisible();
  }

  async expectProjectLoaded() {
    await expect(this.searchInput).toBeVisible();
  }

  async expectSearchDropdownVisible() {
    await expect(this.searchDropdown).toBeVisible();
  }

  async expectSearchDropdownHidden() {
    await expect(this.searchDropdown).not.toBeVisible();
  }

  async expectSearchResultsCount(count: number) {
    await expect(this.searchResults).toHaveCount(count);
  }

  async expectNoSearchResults() {
    await expect(this.page.locator('.search-status')).toContainText('No documents found');
  }

  async expectKanbanColumnsVisible() {
    await expect(this.kanbanColumns.first()).toBeVisible();
  }

  async expectKanbanBoardVisible() {
    await expect(this.kanbanBoard).toBeVisible();
  }

  async switchToTasksBoard() {
    // Click on the Tasks tab to see kanban columns
    await this.page.locator('.board-tab', { hasText: 'Tasks' }).click();
    // Wait for columns to appear
    await this.page.waitForSelector('.kanban-column', { timeout: 5000 });
  }
}
