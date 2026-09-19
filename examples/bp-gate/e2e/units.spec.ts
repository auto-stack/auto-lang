// units.spec.ts — bp-gate vue 臂 gate（PLAN-075 T-03，jade gallery
// units.spec.ts 模式改裁：单页同定 + clip 区域截图）。
// 断言面来自 scripts/units.mjs：全页 needle（fixture 三单元互斥词表）+
// per-unit clip 截图基线（e2e/baselines/<id>.png）。
import { expect, test } from '@playwright/test'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { UNITS } from '../scripts/units.mjs'
import { startDistServer } from '../scripts/serve-dist.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const GATE = path.resolve(here, '..')
const PORT = Number(process.env.BP_GATE_PORT ?? 3100)
const BASE = `http://127.0.0.1:${PORT}`
// dist 在 gate.mjs 的沙箱内（gen/front/vue/dist），路径经 env 注入。
const DIST = process.env.BP_GATE_DIST

let server: any

test.beforeAll(async () => {
  if (!DIST) throw new Error('BP_GATE_DIST not set — run via scripts/gate.mjs')
  server = await startDistServer(DIST, PORT)
})

test.afterAll(async () => {
  await new Promise((r) => server.close(r))
})

test('vue gate: host page renders all unit markers', async ({ page }) => {
  await page.goto(BASE)
  for (const marker of UNITS.map((u) => `unit: ${u.id}`)) {
    await page.waitForSelector(`text=${marker}`, { timeout: 15_000 })
  }
})

for (const u of UNITS) {
  test(`vue gate: ${u.id}（${u.title}）`, async ({ page }) => {
    await page.goto(BASE)
    await page.waitForSelector(`text=unit: ${u.id}`, { timeout: 15_000 })
    const text = await page.evaluate(() => document.body.textContent ?? '')
    for (const needle of u.vue.needles ?? []) {
      if (!text.includes(needle)) {
        throw new Error(`${u.id}: 页面缺 "${needle}"（got: ${text.slice(0, 200)}）`)
      }
    }
    await expect(page).toHaveScreenshot(`${u.id}.png`, { clip: u.vue.clip })
  })
}
