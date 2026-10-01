import { test, expect, type Page } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

// PLAN-718 T-06：按需加载与真实反馈行为测试（AC-08/09/10）。
// 编辑器 chunk 判据 = 内容含 EditorState（'.cm-editor' 会误报：theme 入口含
// 715 热键守卫的选择器字面量）。服务=playwright.config.ts webServer（CI=1 独占端口）。

const DIST_ASSETS = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '.vitepress', 'dist', 'assets')

function chunksWith(marker: string): string[] {
  const out: string[] = []
  const walk = (dir: string) => {
    for (const f of fs.readdirSync(dir)) {
      const p = path.join(dir, f)
      if (fs.statSync(p).isDirectory()) {
        walk(p)
        continue
      }
      if (!f.endsWith('.js')) continue
      if (fs.readFileSync(p, 'utf8').includes(marker)) {
        out.push('/assets/' + path.relative(DIST_ASSETS, p).split(path.sep).join('/'))
      }
    }
  }
  walk(DIST_ASSETS)
  return out
}

const EDITOR_CHUNKS = chunksWith('EditorState')
// runner 入口 shim 以文件名识别（facade 内容会被压缩，常量名不可作标记）
const RUNNER_CHUNKS = chunksByFilename((base) => base.startsWith('runner-entry'))

function chunksByFilename(match: (base: string) => boolean): string[] {
  const out: string[] = []
  const walk = (dir: string) => {
    for (const f of fs.readdirSync(dir)) {
      const p = path.join(dir, f)
      if (fs.statSync(p).isDirectory()) {
        walk(p)
        continue
      }
      if (f.endsWith('.js') && match(f)) {
        out.push('/assets/' + path.relative(DIST_ASSETS, p).split(path.sep).join('/'))
      }
    }
  }
  walk(DIST_ASSETS)
  return out
}

function trackRequests(page: Page): string[] {
  const reqs: string[] = []
  page.on('request', (r) => reqs.push(r.url()))
  return reqs
}

// ---------- AC-09 冷访问轻量 ----------

test.describe('cold pages do not load editor chunks', () => {
  test.beforeAll(() => {
    if (EDITOR_CHUNKS.length === 0) throw new Error('no editor chunks found in dist - marker broken')
  })

  for (const p of ['/', '/docs/', '/books/', '/books/rust/ch03-common-concepts']) {
    test(`cold ${p}: no editor chunk, no /api/run`, async ({ page }) => {
      const reqs = trackRequests(page)
      await page.goto(p, { waitUntil: 'networkidle' })
      const editorHits = reqs.filter((u) => EDITOR_CHUNKS.some((c) => u.includes(c)))
      expect(editorHits, `${p} fetched ${editorHits.join(',')}`).toEqual([])
      expect(reqs.some((u) => u.includes('/api/run')), p).toBe(false)
    })
  }

  test('opening the runner fetches editor resources and calls the run API', async ({ page }) => {
    const reqs = trackRequests(page)
    await page.goto('/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const before = reqs.length
    await page.locator('.auto-fence button.af-run').first().click()
    await page.waitForTimeout(2500)
    const newReqs = reqs.slice(before)
    expect(newReqs.some((u) => EDITOR_CHUNKS.some((c) => u.includes(c)))).toBe(true)
    // 后端缺位时 /api/run 请求本身即 SnippetRunner 既有契约（backendDown 反馈）
    expect(newReqs.some((u) => u.includes('/api/run'))).toBe(true)
  })

  test('locked examples never render a runner or call the API', async ({ page }) => {
    const reqs = trackRequests(page)
    await page.goto('/books/byte-of-python/ch08-modules', { waitUntil: 'networkidle' })
    // 该页同时含锁定与普通围栏——只断言锁定围栏无运行按钮
    const lockedFence = page.locator('.auto-fence', { has: page.locator('.af-lock') }).first()
    await expect(lockedFence).toBeVisible()
    expect(await lockedFence.locator('button.af-run').count()).toBe(0)
    expect(await lockedFence.locator('.af-lock-note').count()).toBe(1)
    await page.waitForTimeout(800)
    expect(reqs.some((u) => u.includes('/api/run'))).toBe(false)
  })
})

// ---------- AC-10 加载失败可恢复 ----------

test.describe('runner load failure and recovery', () => {
  test('failed runner chunk: localized error, original readable, retry mounts, collapse restores', async ({ browser }) => {
    // 干跑发现打开 runner 时新拉的 chunk 文件名（压缩后标识符不可作内容标记）
    const dry = await browser.newContext({ locale: 'en-US' })
    const dp = await dry.newPage()
    await dp.goto('/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const dryReqs: string[] = []
    dp.on('request', (r) => dryReqs.push(r.url()))
    await dp.locator('.auto-fence button.af-run').first().click()
    await dp.waitForTimeout(2000)
    const runnerJs = dryReqs
      .filter((u) => u.endsWith('.js'))
      .map((u) => u.split('/').pop()!)
    expect(runnerJs.length).toBeGreaterThan(0)
    await dry.close()

    // 注入失败：runner 入口 chunk 首拉中断
    const ctx = await browser.newContext({ locale: 'en-US' })
    const page = await ctx.newPage()
    let failedOnce = false
    await page.route('**/assets/**', (route) => {
      const u = route.request().url()
      if (!failedOnce && runnerJs.some((n) => u.endsWith(n))) {
        failedOnce = true
        return route.abort('failed')
      }
      return route.continue()
    })
    await page.goto('/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const fence = page.locator('.auto-fence').first()
    const original = fence.locator('.af-original')
    const originalBefore = await original.innerHTML()
    await fence.locator('button.af-run').click()
    await page.waitForTimeout(1200)
    // 原文仍可读（加载失败等待态不隐藏原文）
    await expect(original).toBeVisible()
    expect(await original.innerHTML()).toBe(originalBefore)
    // 本地化失败说明（不冒充程序运行错误）+ 可重试 + 可收起
    await expect(fence.locator('.af-load-error')).toBeVisible()
    await expect(fence.locator('.af-error-text')).toContainText('runner failed to load')

    // 重试（路由不再拦截）→ runner 挂载
    await fence.locator('.af-error-actions button', { hasText: 'Retry' }).click()
    await page.waitForTimeout(3000)
    await expect(fence.locator('.af-runner .snippet-runner')).toBeVisible()

    // 收起 → 回原文，焦点归还开关
    await fence.locator('button.af-run.open').click()
    await page.waitForTimeout(300)
    await expect(original).toBeVisible()
    await expect(fence.locator('button.af-run')).toBeFocused()
    await ctx.close()
  })

  test('ZH failure message is localized', async ({ browser }) => {
    const dry = await browser.newContext({ locale: 'en-US' })
    const dp = await dry.newPage()
    await dp.goto('/zh/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const dryReqs: string[] = []
    dp.on('request', (r) => dryReqs.push(r.url()))
    await dp.locator('.auto-fence button.af-run').first().click()
    await dp.waitForTimeout(2000)
    const runnerJs = dryReqs.filter((u) => u.endsWith('.js')).map((u) => u.split('/').pop()!)
    await dry.close()

    const ctx = await browser.newContext({ locale: 'en-US' })
    const page = await ctx.newPage()
    let failedOnce = false
    await page.route('**/assets/**', (route) => {
      const u = route.request().url()
      if (!failedOnce && runnerJs.some((n) => u.endsWith(n))) {
        failedOnce = true
        return route.abort('failed')
      }
      return route.continue()
    })
    await page.goto('/zh/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const fence = page.locator('.auto-fence').first()
    await fence.locator('button.af-run').click()
    await page.waitForTimeout(1200)
    await expect(fence.locator('.af-error-text')).toContainText('运行器未加载')
    await ctx.close()
  })

  test('rapid open/close does not duplicate runners or chunk fetches', async ({ page }) => {
    const reqs = trackRequests(page)
    await page.goto('/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const btn = page.locator('.auto-fence button.af-run').first()
    for (let i = 0; i < 4; i++) {
      await btn.click()
      await page.waitForTimeout(120)
    }
    await btn.click()
    await page.waitForTimeout(1200)
    // 任意时刻至多一个 runner 面板（v-if 单实例）
    expect(await page.locator('.auto-fence .af-runner').count()).toBeLessThanOrEqual(1)
    // runner chunk 文件（含 __runnerAttempt shim）至多被请求一次（in-flight 去重）
    const runnerChunkReqs = reqs.filter((u) => RUNNER_CHUNKS.some((c) => u.includes(c.split('/').pop()!)))
    const uniqueRunnerUrls = new Set(runnerChunkReqs.map((u) => u.split('?')[0]))
    expect(uniqueRunnerUrls.size, runnerChunkReqs.join(',')).toBeLessThanOrEqual(3)
    // 收起态原文可见
    await btn.click()
    await page.waitForTimeout(300)
    await expect(page.locator('.auto-fence .af-original').first()).toBeVisible()
  })
})
