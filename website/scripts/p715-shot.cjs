// PLAN-715 T-01/T-07：基线与验收截图脚本。
// 用法：node scripts/p715-shot.cjs <outDir> [baseline|final]
// 对固定页面矩阵（EN/ZH × 390/1440 × 深/浅）截整页图，并输出首图位置测量 JSON。
// 前置：npm run build 完成；服务由本脚本以 vitepress preview 启动（独占端口）。
const { chromium } = require('@playwright/test')
const { spawn } = require('child_process')
const fs = require('fs')
const path = require('path')

const outDir = process.argv[2] || 'docs/p715-shots'
const mode = process.argv[3] || 'final'
const port = Number(process.env.AUTO_WEBSITE_TEST_PORT || 4185)
const base = `http://localhost:${port}`

const PAGES = [
  { name: 'home-en', url: '/', zh: false },
  { name: 'home-zh', url: '/zh/', zh: true },
  { name: 'v05-en', url: '/v05/', zh: false, heroImg: '/v05/desktop-dark-apps.jpg' },
  { name: 'v05-zh', url: '/zh/v05/', zh: true, heroImg: '/v05/desktop-dark-apps.jpg' },
  { name: 'apps-en', url: '/apps', zh: false },
  { name: 'apps-zh', url: '/zh/apps', zh: true },
  { name: 'autoshell-en', url: '/apps/autoshell/', zh: false },
  { name: 'autoshell-zh', url: '/zh/apps/autoshell/', zh: true },
  { name: 'automusk-en', url: '/apps/automusk/', zh: false },
  { name: 'automusk-zh', url: '/zh/apps/automusk/', zh: true },
  { name: 'autodown-en', url: '/apps/autodown/', zh: false },
  { name: 'autodown-zh', url: '/zh/apps/autodown/', zh: true },
  { name: 'autoui-en', url: '/apps/autoui/', zh: false },
  { name: 'autoui-zh', url: '/zh/apps/autoui/', zh: true },
  { name: 'rust-en', url: '/rust', zh: false },
  { name: 'rust-zh', url: '/zh/rust', zh: true },
  { name: 'docs-en', url: '/docs/', zh: false },
  { name: 'docs-zh', url: '/zh/docs/', zh: true },
  { name: 'books-en', url: '/books/', zh: false },
  { name: 'books-zh', url: '/zh/books/', zh: true },
  { name: 'playground-en', url: '/playground', zh: false },
  { name: 'playground-zh', url: '/zh/playground', zh: true },
  { name: 'uidesktop-en', url: '/ui-desktop', zh: false },
  { name: 'uidesktop-zh', url: '/zh/ui-desktop', zh: true },
]

const VIEWPORTS = [
  { w: 390, h: 844, tag: '390' },
  { w: 1440, h: 1000, tag: '1440' },
]

function startPreview() {
  return new Promise((resolve, reject) => {
    const proc = spawn('npx', ['vitepress', 'preview', '--port', String(port), '--strictPort'], {
      cwd: path.resolve(__dirname, '..'),
      shell: true,
      stdio: 'pipe',
    })
    let ready = false
    proc.stdout.on('data', (d) => {
      if (String(d).includes(String(port)) && !ready) { ready = true; resolve(proc) }
    })
    proc.stderr.on('data', (d) => process.stderr.write(d))
    proc.on('exit', (code) => { if (!ready) reject(new Error('preview exited ' + code)) })
    setTimeout(() => { if (!ready) reject(new Error('preview timeout')) }, 60000)
  })
}

async function waitAlive() {
  for (let i = 0; i < 60; i++) {
    try { await fetch(base + '/'); return } catch { await new Promise(r => setTimeout(r, 500)) }
  }
  throw new Error('preview not reachable')
}

;(async () => {
  fs.mkdirSync(outDir, { recursive: true })
  const server = await startPreview()
  await waitAlive()
  const browser = await chromium.launch()
  const report = []
  try {
    for (const vp of VIEWPORTS) {
      for (const theme of ['dark', 'light']) {
        const ctx = await browser.newContext({ viewport: { width: vp.w, height: vp.h } })
        await ctx.addInitScript((t) => {
          localStorage.setItem('vitepress-theme-appearance', t)
        }, theme)
        const page = await ctx.newPage()
        for (const p of PAGES) {
          const url = p.zh ? ('/zh' + (p.url === '/' ? '/' : p.url)) : p.url
          await page.goto(base + url, { waitUntil: 'networkidle' })
          const file = path.join(outDir, `${mode}-${p.name}-${vp.tag}-${theme}.png`)
          await page.screenshot({ path: file, fullPage: true })
          const row = { page: p.name, url, viewport: vp.tag, theme, file }
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
    server.kill()
  }
  fs.writeFileSync(path.join(outDir, `${mode}-report.json`), JSON.stringify(report, null, 2))
  const bad = report.filter(r => r.hasHScroll || r.within15 === false)
  console.log('shots done:', report.length, 'problem rows:', bad.length)
  for (const b of bad) console.log('PROBLEM', JSON.stringify(b))
})().catch((e) => { console.error(e); process.exit(1) })
