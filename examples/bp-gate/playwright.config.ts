import { defineConfig } from '@playwright/test'
import { GEOMETRY } from './scripts/units.mjs'

// playwright.config.ts — bp-gate vue 臂（PLAN-075 T-03，jade gallery 配置
// 同形）。单页同定 + per-unit clip 区域截图基线（e2e/baselines/），
// maxDiffPixelRatio 与 jade gallery 对齐（0.02）。视口/单元几何来自
// units.mjs GEOMETRY（clip 计算同源）。
export default defineConfig({
  testDir: './e2e',
  outputDir: './e2e/test-results',
  workers: 1,
  retries: 0,
  timeout: 30_000,
  reporter: [['list']],
  snapshotPathTemplate: '{testDir}/baselines/{arg}{ext}',
  use: {
    viewport: GEOMETRY.viewport,
  },
  expect: {
    toHaveScreenshot: {
      animations: 'disabled',
      caret: 'hide',
      maxDiffPixelRatio: 0.02,
    },
  },
})
