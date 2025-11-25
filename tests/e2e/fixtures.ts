import { test as base, expect } from '@playwright/test';

/**
 * Custom test fixtures for Metis E2E tests.
 *
 * These fixtures provide:
 * - Common page object patterns
 * - Test data setup/teardown
 * - Helper methods for common operations
 */

// Extend base test with custom fixtures
export const test = base.extend<{
  // Add custom fixtures here as needed
}>({
  // Example: could add a fixture for test project setup
});

export { expect };

// Page object helpers
export class MetisPage {
  constructor(private page: any) {}

  // Navigation
  async goto() {
    await this.page.goto('/');
  }

  // Selectors
  get searchInput() {
    return this.page.locator('input[placeholder="Search documents..."]');
  }

  get themeButton() {
    return this.page.locator('.theme-button');
  }

  get projectSidebar() {
    return this.page.locator('.project-sidebar, [class*="sidebar"]').first();
  }

  get kanbanBoard() {
    return this.page.locator('.kanban-board, [class*="kanban"]').first();
  }

  get mascotImage() {
    return this.page.locator('.home-icon-main');
  }

  // Actions
  async search(query: string) {
    await this.searchInput.fill(query);
    // Wait for debounce
    await this.page.waitForTimeout(400);
  }

  async clearSearch() {
    await this.searchInput.press('Escape');
  }

  async selectSearchResult(index: number) {
    const results = this.page.locator('.search-result-item');
    await results.nth(index).click();
  }

  async toggleTheme() {
    await this.themeButton.click();
  }

  async selectTheme(theme: 'Light' | 'Dark' | 'Hyper') {
    await this.themeButton.click();
    await this.page.locator('.theme-option', { hasText: theme }).click();
  }

  async selectProject(projectName: string) {
    await this.page.locator('.project-item, [class*="project"]', { hasText: projectName }).click();
  }

  // Assertions
  async expectHomeScreen() {
    await expect(this.mascotImage).toBeVisible();
  }

  async expectKanbanBoard() {
    await expect(this.kanbanBoard).toBeVisible();
  }

  async expectSearchResults() {
    await expect(this.page.locator('.search-dropdown')).toBeVisible();
  }

  async expectNoSearchResults() {
    await expect(this.page.locator('.search-status')).toContainText('No documents found');
  }
}
