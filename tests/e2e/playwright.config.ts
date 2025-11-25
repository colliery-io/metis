import { defineConfig, devices } from '@playwright/test';
import path from 'path';

/**
 * Playwright configuration for Metis GUI E2E tests.
 *
 * We test against the Vite dev server which covers the Vue UI.
 * Tauri commands are invoked through the webview.
 */
export default defineConfig({
  testDir: '.',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { open: 'never', outputFolder: 'playwright-report' }],
    ['list']
  ],

  use: {
    // Base URL for the dev server
    baseURL: 'http://localhost:1420',

    // Collect trace on failure for debugging
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Start the Vite dev server before running tests
  webServer: {
    command: 'npm run dev',
    cwd: path.join(__dirname, '../../crates/metis-docs-gui'),
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  // Output directory for test artifacts
  outputDir: 'test-results',
});
