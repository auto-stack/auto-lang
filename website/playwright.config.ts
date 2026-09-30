import { defineConfig, devices } from '@playwright/test'

// Plan 582 T16：website e2e 最小配置。前置：npm run build（vitepress preview 服
// dist 静态产物——无后端路径正是被测形态；本地 3030 后端在位时由 spec 内探测启用
// 后端相关断言）。
// PLAN-715 T-01：端口改为 AUTO_WEBSITE_TEST_PORT 可配置（默认 4173 保持兼容）。
// 并行 worktree 各选独占空闲端口（本计划用 4185），baseURL 与 webServer 同源，
// 防止误连未知服务；strictPort 保证端口被占即失败而非错测。
const port = Number(process.env.AUTO_WEBSITE_TEST_PORT || 4173)
const baseURL = `http://localhost:${port}`

export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  retries: 0,
  use: {
    baseURL,
    // PLAN-715 T-07：钉住 locale——否则宿主 OS 中文 locale 泄入 navigator.language，
    // 首页的浏览器语言自动跳转会把 '/ '测试整个搬到 /zh/ 上（实测复现）。
    locale: 'en-US',
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `npx vitepress preview --port ${port} --strictPort`,
    url: `${baseURL}/`,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
})
