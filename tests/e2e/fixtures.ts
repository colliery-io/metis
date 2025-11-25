import { test as base, expect, Page } from '@playwright/test';

/**
 * Custom test fixtures for Metis E2E tests.
 */

// Test project data that matches the mock
const TEST_PROJECT = {
  path: '/test/project',
  is_valid: true,
  vision_exists: true,
};

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

// Page object for common operations
export class MetisPage {
  constructor(private page: Page) {}

  // Navigation
  async goto() {
    await this.page.goto('/');
    await this.page.waitForSelector('.home-icon-topbar');
  }

  // Set up test project
  async setupTestProject() {
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
