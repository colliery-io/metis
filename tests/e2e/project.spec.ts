import { test, expect, MetisPage } from './fixtures';

test.describe('Project Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should show project sidebar', async ({ page }) => {
    // The sidebar should be visible on load
    const sidebar = page.locator('.w-1\\/5').first();
    await expect(sidebar).toBeVisible();
  });

  test('should show home icon that returns to home', async ({ page }) => {
    const metis = new MetisPage(page);

    await expect(metis.homeIcon).toBeVisible();

    // Click should keep us on home screen
    await metis.homeIcon.click();

    // Mascot should be visible (home screen)
    await metis.expectHomeScreen();
  });

  test('should show Add Project button', async ({ page }) => {
    const addButton = page.locator('.add-project-button');
    await expect(addButton).toBeVisible();
    await expect(addButton).toContainText('Add Project');
  });

  test('should show empty state when no projects', async ({ page }) => {
    // Clear any existing projects from localStorage
    await page.evaluate(() => {
      localStorage.removeItem('metis-recent-projects');
    });
    await page.reload();

    // Should show empty state message
    await expect(page.locator('text=No projects yet')).toBeVisible();
  });

  test.describe('Project Loading', () => {
    test('should show project in sidebar after adding', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();

      // Project should appear in sidebar (uses sidebar-project-card class)
      const projectCard = page.locator('.sidebar-project-card').first();
      await expect(projectCard).toBeVisible();
    });

    test('should load project when clicked from sidebar', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();

      // Click on the project
      await metis.loadTestProject();

      // Search bar should now be visible (indicates project is loaded)
      await metis.expectProjectLoaded();
    });

    test('should show project name in header when loaded', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();
      await metis.loadTestProject();

      // Header should show project name instead of just "Metis"
      const header = page.locator('h1');
      await expect(header).toContainText('project');
    });

    test('should show kanban board when project loaded', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();
      await metis.loadTestProject();

      // Kanban board container should be visible
      await metis.expectKanbanBoardVisible();

      // Board tabs should be visible
      await expect(metis.boardTabs.first()).toBeVisible();
    });

    test('should return to home when clicking mascot', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();
      await metis.loadTestProject();

      // Verify we're on project view
      await metis.expectProjectLoaded();

      // Click home icon
      await metis.goHome();

      // Should be back on home screen
      await metis.expectHomeScreen();

      // Search should no longer be visible
      await expect(metis.searchInput).not.toBeVisible();
    });

    test('should display board tabs when project loaded', async ({ page }) => {
      const metis = new MetisPage(page);
      await metis.setupTestProject();
      await metis.loadTestProject();

      // Board should be visible
      await metis.expectKanbanBoardVisible();

      // Should have multiple board tabs (Vision, Tasks, etc.)
      const tabCount = await metis.boardTabs.count();
      expect(tabCount).toBeGreaterThan(1);
    });
  });
});
