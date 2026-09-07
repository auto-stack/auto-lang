import { defineConfig, devices } from '@playwright/test'

// Plan 582 T16：website e2e 最小配置。前置：npm run build（vitepress preview 服
// dist 静态产物——无后端路径正是被测形态；本地 3030 后端在位时由 spec 内探测启用
// 后端相关断言）。reuseExistingServer 允许与手动 preview/dev 实例共存。
export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:4173',
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'npx vitepress preview --port 4173 --strictPort',
    url: 'http://localhost:4173/',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
})
