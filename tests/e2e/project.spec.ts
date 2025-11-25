import { test, expect } from '@playwright/test';

test.describe('Project Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should show project sidebar', async ({ page }) => {
    // The sidebar should be visible on load
    // Look for the sidebar container
    const sidebar = page.locator('.w-1\\/5').first();
    await expect(sidebar).toBeVisible();
  });

  test('should show home icon that returns to home', async ({ page }) => {
    const homeIcon = page.locator('.home-icon-topbar');
    await expect(homeIcon).toBeVisible();

    // Click should return to home (if on a project)
    await homeIcon.click();

    // Mascot should be visible (home screen)
    const mascot = page.locator('.home-icon-main');
    await expect(mascot).toBeVisible();
  });

  // Project loading tests - require actual projects
  test.describe('Project Loading', () => {
    test.skip('should load project when selected from sidebar', async ({ page }) => {
      // TODO: Implement when we have test project setup
      // Click on a project in the sidebar
      // Verify kanban board appears
      // Verify search becomes available
    });

    test.skip('should show project name in header when loaded', async ({ page }) => {
      // TODO: Implement when we have test project setup
      // Select a project
      // Verify header shows project name instead of "Metis"
    });

    test.skip('should show kanban board columns when project loaded', async ({ page }) => {
      // TODO: Implement when we have test project setup
      // Select a project
      // Verify kanban columns are visible (Todo, Doing, Done, etc.)
    });
  });
});
