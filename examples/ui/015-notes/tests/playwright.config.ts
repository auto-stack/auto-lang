import { defineConfig, devices } from '@playwright/test'

/**
 * Playwright 配置（Plan 366a）
 *
 * 测试文件在 tests/ 目录，对应 acceptance.atd 的 T1-T13 契约。
 * 运行方式：
 *   pnpm test          # 跑所有测试（需先启动 dev server）
 *   pnpm test:headed   # 有头模式（看浏览器）
 *   pnpm test:ui       # 打开 Playwright UI 模式
 *
 * 环境变量：
 *   NOTES_URL — 被测应用 URL（默认 http://localhost:3000）
 */
export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  fullyParallel: false, // 015-notes 是有状态的应用，串行更稳定
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],
  outputDir: 'test-results/',
  use: {
    baseURL: process.env.NOTES_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 5000,
    navigationTimeout: 10000,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
