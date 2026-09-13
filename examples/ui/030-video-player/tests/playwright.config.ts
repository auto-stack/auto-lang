import { defineConfig, devices } from '@playwright/test'

// PLAN-617：本套件断言的是**真实播放**（`videoWidth>0`、`currentTime` 递增、
// seek 落点、音量/倍速落到元素属性）。这两点对环境有硬要求：
//
// 1. **必须用本机真实 Chrome**（`channel: 'chrome'`）。Playwright 自带的
//    Chromium 不含 H.264/AAC/HEVC 专有编解码器，实测 `videoWidth` 恒为 0 ——
//    在它上面 T2 永远不可能通过，而这与应用的实现无关。
// 2. **autoplay 策略显式关掉**，让「起播」在测试里是确定的，而不是取决于
//    浏览器对页面的用户激活判定。
//
// 另需 `AUTO_MEDIA_ROOT` 指向真实媒体根（本仓的开发机为 `E:\Video`）；
// 媒体根为空时 T1/T2 一类断言会退化——那属于环境问题，不是实现问题。
export default defineConfig({
  testDir: '.',
  testMatch: '*.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  // 真实播放断言要等加载与回灌（4K MKV 的 metadata 也不例外）
  timeout: 60000,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  outputDir: 'test-results/',
  use: {
    baseURL: process.env.VIDEO_PLAYER_URL || 'http://localhost:3030',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 15000,
    navigationTimeout: 20000,
  },
  projects: [
    {
      name: 'chrome',
      use: {
        ...devices['Desktop Chrome'],
        channel: 'chrome',
        launchOptions: { args: ['--autoplay-policy=no-user-gesture-required'] },
      },
    },
  ],
  webServer: {
    command: 'pnpm run preview --port 3030',
    cwd: '../gen/front/vue',
    port: 3030,
    reuseExistingServer: !process.env.CI,
    timeout: 15000,
  },
})
