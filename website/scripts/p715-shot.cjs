// PLAN-715 T-01/T-07：基线与验收截图脚本。
// 用法：node scripts/p715-shot.cjs <outDir> [baseline|final]
// 对固定页面矩阵（EN/ZH × 390/1440 × 深/浅）截整页图，并输出首图位置测量 JSON。
// 前置：npm run build 完成；服务由本脚本以 vitepress preview 启动（独占端口）。
// PLAN-718 T-01 修正：PAGES.url 即真实 URL（不再从 zh 布尔值二次拼装 /zh——
// 旧逻辑对已含 /zh/ 的 URL 产生 /zh/zh/...）；goto 后校验 pathname/locale/
// 正文非 404；浏览器 locale 固定 en-US；URL/404 断言失败使脚本退出码非 0。
const { chromium } = require('@playwright/test')
const fs = require('fs')
const path = require('path')
const { startPreview, stopPreview, waitAlive, validateLanding } = require('./shot-common.cjs')

const outDir = process.argv[2] || 'docs/p715-shots'
const mode = process.argv[3] || 'final'
const port = Number(process.env.AUTO_WEBSITE_TEST_PORT || 4185)
const base = `http://localhost:${port}`

// url 即最终访问 URL（zh 页面已带 /zh 前缀）。
const PAGES = [
  { name: 'home-en', url: '/' },
  { name: 'home-zh', url: '/zh/' },
  { name: 'v05-en', url: '/v05/', heroImg: '/v05/desktop-hero.png' },
  { name: 'v05-zh', url: '/zh/v05/', heroImg: '/v05/desktop-hero.png' },
  { name: 'apps-en', url: '/apps' },
  { name: 'apps-zh', url: '/zh/apps' },
  { name: 'autoshell-en', url: '/apps/autoshell/' },
  { name: 'autoshell-zh', url: '/zh/apps/autoshell/' },
  { name: 'automusk-en', url: '/apps/automusk/' },
  { name: 'automusk-zh', url: '/zh/apps/automusk/' },
  { name: 'autodown-en', url: '/apps/autodown/' },
  { name: 'autodown-zh', url: '/zh/apps/autodown/' },
  { name: 'autoui-en', url: '/apps/autoui/' },
  { name: 'autoui-zh', url: '/zh/apps/autoui/' },
  { name: 'rust-en', url: '/rust' },
  { name: 'rust-zh', url: '/zh/rust' },
  { name: 'docs-en', url: '/docs/' },
  { name: 'docs-zh', url: '/zh/docs/' },
  { name: 'books-en', url: '/books/' },
  { name: 'books-zh', url: '/zh/books/' },
  { name: 'playground-en', url: '/playground' },
  { name: 'playground-zh', url: '/zh/playground' },
  { name: 'uidesktop-en', url: '/ui-desktop' },
  { name: 'uidesktop-zh', url: '/zh/ui-desktop' },
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
          // 宿主 OS locale 不得泄入：语言跳转按 URL 判定，不按 navigator.language。
          locale: 'en-US',
        })
        await ctx.addInitScript((t) => {
          localStorage.setItem('vitepress-theme-appearance', t)
        }, theme)
        const page = await ctx.newPage()
        for (const p of PAGES) {
          const url = p.url
          await page.goto(base + url, { waitUntil: 'networkidle' })
          const landing = await validateLanding(page, url)
          for (const f of landing.fails) hardFails.push(`${p.name} [${vp.tag}/${theme}]: ${f}`)
          const file = path.join(outDir, `${mode}-${p.name}-${vp.tag}-${theme}.png`)
          await page.screenshot({ path: file, fullPage: true })
          const row = { page: p.name, url, landedPathname: landing.pathname, viewport: vp.tag, theme, file }
          // 首图位置：AC-09 主图 top < 1.5×视口高（仅 v05 页在 390/1440 上记录）
          if (p.heroImg && (vp.tag === '390' || vp.tag === '1440')) {
            const img = page.locator(`img[src="${p.heroImg}"]`).first()
            if (await img.count()) {
              const box = await img.boundingBox()
              row.heroImgTop = box ? Math.round(box.y) : null
              row.viewportH = vp.h
              row.within15 = box ? box.y < 1.5 * vp.h : null
            }
          }
          // 首屏横向溢出检测：AC-05
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
  const bad = report.filter(r => r.hasHScroll || r.within15 === false)
  console.log('shots done:', report.length, 'landing fails:', hardFails.length, 'problem rows:', bad.length)
  for (const f of hardFails) console.log('LANDING-FAIL', f)
  for (const b of bad) console.log('PROBLEM', JSON.stringify(b))
  if (hardFails.length) process.exit(1)
})().catch((e) => { console.error(e); process.exit(1) })
