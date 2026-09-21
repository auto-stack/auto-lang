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

// PLAN-665 AC-04：sandwich 三层壳弹性语义自动化断言——041"只保留内容高"
// 塌缩回归的正向锚（renderer 高度语义再变动时此测先红）。
test('vue gate: sandwich geometry (AC-04) — 三层壳弹性/贴底/分栏断言', async ({ page }) => {
  await page.goto(BASE)
  await page.waitForSelector('text=unit: sandwich', { timeout: 15_000 })
  const box = async (needle: string) => {
    const el = page.locator(`text=${needle}`).first()
    await el.waitFor({ state: 'visible', timeout: 10_000 })
    const b = await el.boundingBox()
    if (!b) throw new Error(`${needle}: no boundingBox`)
    return b
  }
  const toolbar = await box('sw toolbar')
  const sidebar = await box('sw sidebar')
  const status = await box('sw status')
  const content = await box('swf full content')
  const nestedStatus = await box('swf status')

  // ① statusbar 底缘贴单元底（PLAN-676：sandwich 之下新增 gallery_shell
  // 单元，"贴视口底"性质随最底单元转移——改为贴下一单元顶缘，同一弹性
  // 语义：满窗壳不被内容高反向决定。gap=下一标记行顶缘-statusbar 底，
  // 含 pt-2 内边距 8px 与文本基线容差）
  const gsMarker = await box('unit: gallery_shell')
  const statusBottom = status.y + status.height
  const gap = gsMarker.y - statusBottom
  if (gap < -8 || gap > 24) {
    throw new Error(`sandwich statusbar 未贴单元底（下一单元顶缘 gap=${gap}px）`)
  }

  // ② content 弹性区高 ≥ 300px（default 壳 toolbar 底 → statusbar 顶）
  const contentTop = toolbar.y + toolbar.height
  const contentBottom = status.y
  if (contentBottom - contentTop < 300) {
    throw new Error(`content 弹性区高 ${contentBottom - contentTop}px < 300px（塌缩回归形态）`)
  }

  // ③ 三层序：toolbar 顶 / content 中 / statusbar 底
  if (!(toolbar.y < content.y && content.y + content.height < status.y)) {
    throw new Error('三层序断裂：toolbar/content/statusbar 垂直序不符')
  }

  // ④ 分栏落位：sidebar 左列（x 显著小于主区内容）
  if (sidebar.x >= content.x - 100) {
    throw new Error(`sidebar 未落左列: sidebar.x=${sidebar.x}, content.x=${content.x}`)
  }

  // ⑤ 组合形态：嵌套 full 壳 statusbar 在外层 statusbar 之上
  if (!(nestedStatus.y + nestedStatus.height <= status.y + 8)) {
    throw new Error('嵌套 full 壳 statusbar 越过外层 statusbar')
  }
})
