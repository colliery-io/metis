import { test, expect, MetisPage } from './fixtures';

test.describe('Search Functionality', () => {
  test('search bar should not be visible without project', async ({ page }) => {
    await page.goto('/');

    // On home screen without project, search should not be visible
    const searchInput = page.locator('input[placeholder="Search documents..."]');
    await expect(searchInput).not.toBeVisible();
  });

  test.describe('With Project Loaded', () => {
    test.beforeEach(async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.goto();
      await metis.setupTestProject();
      await metis.loadTestProject();
    });

    test('search input should be visible', async ({ page }) => {
      const metis = new MetisPage(page);
      await expect(metis.searchInput).toBeVisible();
    });

    test('should show dropdown when typing', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('test');

      // Dropdown should appear with results
      await metis.expectSearchDropdownVisible();
    });

    test('should show results matching query', async ({ page }) => {
      const metis = new MetisPage(page);

      // Search for a term that matches mock data
      await metis.search('Feature');

      await metis.expectSearchDropdownVisible();
      // Should find "Implement Feature X"
      await expect(metis.searchResults.first()).toContainText('Feature');
    });

    test('should show no results message for non-matching query', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('xyznonexistent123');

      await metis.expectNoSearchResults();
    });

    test('should navigate results with keyboard', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('test');
      await metis.expectSearchDropdownVisible();

      // Press down arrow to select first result
      await metis.navigateSearchResults('down');

      // First result should be highlighted (has .selected class)
      const selectedResult = page.locator('.search-result-item.selected');
      await expect(selectedResult).toBeVisible();
    });

    test('should clear search on Escape', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('test');
      await metis.expectSearchDropdownVisible();

      // Press Escape to clear
      await metis.clearSearch();

      // Input should be cleared
      await expect(metis.searchInput).toHaveValue('');

      // Dropdown should be hidden
      await metis.expectSearchDropdownHidden();
    });

    test('should close dropdown when clicking outside', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('test');
      await metis.expectSearchDropdownVisible();

      // Click outside the search area (on the main content area)
      await page.locator('.home-icon-topbar').click();

      // Dropdown should be hidden
      await metis.expectSearchDropdownHidden();
    });

    test('should select result on Enter', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('Feature');
      await metis.expectSearchDropdownVisible();

      // Select first result
      await metis.navigateSearchResults('down');
      await metis.searchInput.press('Enter');

      // Search should be cleared after selection
      await expect(metis.searchInput).toHaveValue('');
    });

    test('should display document type colors', async ({ page }) => {
      const metis = new MetisPage(page);

      await metis.search('test');
      await metis.expectSearchDropdownVisible();

      // Results should have type badges
      const typeBadge = page.locator('.type-badge').first();
      await expect(typeBadge).toBeVisible();
    });
  });
});
