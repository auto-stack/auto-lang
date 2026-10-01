// PLAN-718 T-01/T-07：阅读矩阵截图脚本（学习入口 + 长文阅读验收）。
// 用法：node scripts/p718-shot.cjs <outDir> [baseline|final]
// 矩阵：docs hub / books hub / 真实长文 / 真实书章节 × EN/ZH × 390/1440 × 深/浅
// = 32 图；记录 pathname/locale/404 落地断言（失败退出码非 0）、移动端
// VPLocalNav 与统一顶栏的叠层几何、auto-fence 数量与页面级横向溢出。
// 360/768/1024 行为断言由 tests/reader-loading.spec.ts 覆盖，不在此重复。
// 前置：npm run build 完成；服务由本脚本以 vitepress preview 启动（独占端口）。
const { chromium } = require('@playwright/test')
const fs = require('fs')
const path = require('path')
const { startPreview, stopPreview, waitAlive, validateLanding } = require('./shot-common.cjs')

const outDir = process.argv[2] || 'docs/p718-shots'
const mode = process.argv[3] || 'final'
const port = Number(process.env.AUTO_WEBSITE_TEST_PORT || 4198)
const base = `http://localhost:${port}`

// T-01 固定的真实内容 URL（文件在案：docs/features/actor-concurrency(.cn).md、
// book 仓 tapl/ch01-getting-started(.cn).md；ZH 侧未译时为既定 EN 回退）。
const PAGES = [
  { name: 'docs-hub-en', url: '/docs/' },
  { name: 'docs-hub-zh', url: '/zh/docs/' },
  { name: 'books-hub-en', url: '/books/' },
  { name: 'books-hub-zh', url: '/zh/books/' },
  { name: 'article-en', url: '/docs/features/actor-concurrency' },
  { name: 'article-zh', url: '/zh/docs/features/actor-concurrency' },
  { name: 'chapter-en', url: '/books/tapl/ch01-getting-started' },
  { name: 'chapter-zh', url: '/zh/books/tapl/ch01-getting-started' },
]

const VIEWPORTS = [
  { w: 390, h: 844, tag: '390' },
  { w: 1440, h: 1000, tag: '1440' },
]

;(async () => {
  fs.mkdirSync(outDir, { recursive: true })
  const server = await startPreview(port)
  await waitAlive(base)
  const browser = await chromium.launch()
  const report = []
  const hardFails = []
  try {
    for (const vp of VIEWPORTS) {
      for (const theme of ['dark', 'light']) {
        const ctx = await browser.newContext({
          viewport: { width: vp.w, height: vp.h },
          locale: 'en-US',
        })
        await ctx.addInitScript((t) => {
          localStorage.setItem('vitepress-theme-appearance', t)
        }, theme)
        const page = await ctx.newPage()
        for (const p of PAGES) {
          await page.goto(base + p.url, { waitUntil: 'networkidle' })
          const landing = await validateLanding(page, p.url)
          for (const f of landing.fails) hardFails.push(`${p.name} [${vp.tag}/${theme}]: ${f}`)
          const file = path.join(outDir, `${mode}-${p.name}-${vp.tag}-${theme}.png`)
          await page.screenshot({ path: file, fullPage: true })
          const row = {
            page: p.name,
            url: p.url,
            landedPathname: landing.pathname,
            viewport: vp.tag,
            theme,
            file,
          }
          // 阅读几何：统一顶栏底边 vs VPLocalNav 顶边（<960px 出现；sticky top=0
          // 时与顶栏重叠——基线应记录重叠事实，修正后应记录让位事实）。
          row.localNav = await page.evaluate(() => {
            const nav = document.querySelector('.site-navbar')
            const local = document.querySelector('.VPLocalNav')
            // display:none（≥1280px 隐藏）时元素仍在 DOM，几何全 0——先判可见再算叠压。
            const visible = !!local && local.offsetParent !== null
            const nb = nav ? Math.round(nav.getBoundingClientRect().bottom) : null
            const lt = visible ? Math.round(local.getBoundingClientRect().top) : null
            return {
              present: !!local,
              visible,
              navbarBottom: nb,
              localNavTop: lt,
              overlapsNavbar: !!(nav && visible) && lt >= 0 && lt < nb,
            }
          })
          row.autoFenceCount = await page.locator('.auto-fence').count()
          row.outlineItems = await page.locator('.VPDocAsideOutline .outline-link').count()
          // 滚动态叠层：两个 sticky 元素静止时按文档流排列不重叠，滚动后同时钉在
          // top:0 才暴露叠压——滚动 800px 后再测（AC-04 的真实场景）。
          await page.evaluate(() => window.scrollTo(0, 800))
          await page.waitForTimeout(300)
          row.localNavScrolled = await page.evaluate(() => {
            const nav = document.querySelector('.site-navbar')
            const local = document.querySelector('.VPLocalNav')
            const visible = !!local && local.offsetParent !== null
            const nb = nav ? Math.round(nav.getBoundingClientRect().bottom) : null
            const lt = visible ? Math.round(local.getBoundingClientRect().top) : null
            return { visible, navbarBottom: nb, localNavTop: lt, overlapsNavbar: !!(nav && visible) && lt >= 0 && lt < nb }
          })
          await page.evaluate(() => window.scrollTo(0, 0))
          row.docScrollW = await page.evaluate(() => document.documentElement.scrollWidth)
          row.docClientW = await page.evaluate(() => document.documentElement.clientWidth)
          row.hasHScroll = row.docScrollW > row.docClientW + 1
          report.push(row)
        }
        await ctx.close()
      }
    }
  } finally {
    await browser.close()
    stopPreview(server)
  }
  fs.writeFileSync(path.join(outDir, `${mode}-report.json`), JSON.stringify(report, null, 2))
  const bad = report.filter(r => r.hasHScroll || r.localNavScrolled.overlapsNavbar)
  console.log('shots done:', report.length, 'landing fails:', hardFails.length, 'problem rows:', bad.length)
  for (const f of hardFails) console.log('LANDING-FAIL', f)
  for (const b of bad) console.log('PROBLEM', JSON.stringify(b))
  if (hardFails.length) process.exit(1)
})().catch((e) => { console.error(e); process.exit(1) })
