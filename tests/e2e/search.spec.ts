import { test, expect } from '@playwright/test';

test.describe('Search Functionality', () => {
  // Note: Search is only visible when a project is loaded.
  // These tests require a project to be selected first.

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // TODO: Select a test project when project selection is available
    // For now, we test the search component in isolation
  });

  test('search bar should not be visible without project', async ({ page }) => {
    // On home screen without project, search should not be visible
    const searchInput = page.locator('input[placeholder="Search documents..."]');
    await expect(searchInput).not.toBeVisible();
  });

  // The following tests require a project to be loaded
  // They will be skipped until we have proper test project setup

  test.describe('With Project Loaded', () => {
    test.skip('search input should be visible', async ({ page }) => {
      const searchInput = page.locator('input[placeholder="Search documents..."]');
      await expect(searchInput).toBeVisible();
    });

    test.skip('should show dropdown when typing', async ({ page }) => {
      const searchInput = page.locator('input[placeholder="Search documents..."]');
      await searchInput.fill('test');

      // Wait for debounce
      await page.waitForTimeout(400);

      // Dropdown should appear
      const dropdown = page.locator('.search-dropdown');
      await expect(dropdown).toBeVisible();
    });

    test.skip('should navigate results with keyboard', async ({ page }) => {
      const searchInput = page.locator('input[placeholder="Search documents..."]');
      await searchInput.fill('test');
      await page.waitForTimeout(400);

      // Press down arrow to select first result
      await searchInput.press('ArrowDown');

      // First result should be highlighted
      const selectedResult = page.locator('.search-result-item.selected');
      await expect(selectedResult).toBeVisible();
    });

    test.skip('should clear search on Escape', async ({ page }) => {
      const searchInput = page.locator('input[placeholder="Search documents..."]');
      await searchInput.fill('test');
      await page.waitForTimeout(400);

      await searchInput.press('Escape');

      // Input should be cleared
      await expect(searchInput).toHaveValue('');

      // Dropdown should be hidden
      const dropdown = page.locator('.search-dropdown');
      await expect(dropdown).not.toBeVisible();
    });
  });
});
