import { test, expect } from '@playwright/test';
import { MetisPage } from './fixtures';

test.describe('App Launch', () => {
  test('should display home screen on initial load', async ({ page }) => {
    await page.goto('/');

    // Should show the Metis mascot on home screen
    const mascot = page.locator('.home-icon-main');
    await expect(mascot).toBeVisible();

    // Should show the app title
    await expect(page.locator('h1')).toContainText('Metis');

    // Should show version number
    await expect(page.locator('text=/v\\d+\\.\\d+\\.\\d+/')).toBeVisible();
  });

  test('should have theme toggle visible', async ({ page }) => {
    await page.goto('/');

    const themeLabel = page.locator('text=Theme:');
    await expect(themeLabel).toBeVisible();

    const themeButton = page.locator('.theme-button');
    await expect(themeButton).toBeVisible();
  });

  test('should have mascot in top bar', async ({ page }) => {
    await page.goto('/');

    const topBarMascot = page.locator('.home-icon-topbar');
    await expect(topBarMascot).toBeVisible();
  });
});

test.describe('Theme Switching', () => {
  test('should switch between themes', async ({ page }) => {
    await page.goto('/');

    const themeButton = page.locator('.theme-button');

    // Click to open theme dropdown
    await themeButton.click();

    // Should show theme options
    const themeDropdown = page.locator('.theme-dropdown');
    await expect(themeDropdown).toBeVisible();

    // Select Dark theme
    await page.locator('.theme-option', { hasText: 'Dark' }).click();

    // Verify theme attribute changed
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  });

  test('should save theme selection to localStorage', async ({ page }) => {
    await page.goto('/');

    // Set to Hyper theme
    await page.locator('.theme-button').click();
    await page.locator('.theme-option', { hasText: 'Hyper' }).click();

    // Verify theme is applied immediately
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'hyper');

    // Verify localStorage was updated
    const storedTheme = await page.evaluate(() => localStorage.getItem('metis-theme'));
    expect(storedTheme).toBe('hyper');
  });
});
