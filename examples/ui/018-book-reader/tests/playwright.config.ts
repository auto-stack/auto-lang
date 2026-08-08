import { defineConfig, devices } from '@playwright/test'
export default defineConfig({
  testDir: '.', testMatch: '*.spec.ts', fullyParallel: false,
  forbidOnly: !!process.env.CI, retries: process.env.CI ? 1 : 0, workers: 1,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  outputDir: 'test-results/',
  use: {
    baseURL: process.env.BOOK_URL || 'http://localhost:3018',
    trace: 'on-first-retry', screenshot: 'only-on-failure', video: 'retain-on-failure',
    actionTimeout: 5000, navigationTimeout: 10000,
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
})
