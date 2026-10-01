const { chromium } = require('@playwright/test')
const fs = require('node:fs')
const path = require('node:path')
const assert = require('node:assert/strict')
const routes = ['/os', '/ai', '/autoos/', '/ui-desktop', '/articles/autoos-history', '/articles/auto-ai-history']
  .flatMap(p => [p, '/zh'+p])
const dir = path.resolve(__dirname, '../../docs/reports/p719-os-ai-introduction')
const base = process.env.AUTO_WEBSITE_SHOT_URL || 'http://127.0.0.1:4225'
;(async () => {
  fs.mkdirSync(dir, { recursive: true })
  const browser = await chromium.launch()
  try {
    const page = await browser.newPage({ locale: 'en-US' })
    const manifest = []
    await page.goto(base)
    for (const theme of ['light','dark']) {
      await page.evaluate(t => localStorage.setItem('vitepress-theme-appearance', t), theme)
      for (const width of [360,390,768,1024,1440]) {
        await page.setViewportSize({ width, height: 1000 })
        for (const route of routes) {
          await page.goto(base+route)
          await page.locator('.VPDoc h1').waitFor()
          assert.equal(new URL(page.url()).pathname.replace(/\/$/,''), route.replace(/\/$/,''))
          assert.equal(await page.locator('.VPDoc h1').count(), 1)
          assert.equal(await page.locator('.VPNotFound').count(), 0)
          assert.ok((await page.locator('.vp-doc').innerText()).length > 500)
          assert.equal(await page.locator('html').getAttribute('lang'), route.startsWith('/zh') ? 'zh-CN' : 'en-US')
          assert.equal(await page.evaluate(() => document.documentElement.classList.contains('dark')), theme === 'dark')
          const overflow = await page.evaluate(() => document.documentElement.scrollWidth-innerWidth)
          assert.ok(overflow <= 1, `${route} ${width} ${theme}: overflow ${overflow}`)
          const title = await page.locator('.VPDoc h1').boundingBox()
          const nav = await page.locator('header.site-navbar').boundingBox()
          assert.ok(title && nav && title.y >= nav.y+nav.height, `${route}: heading under navigation`)
          for (const link of await page.locator('.intro-links a').all()) {
            assert.ok((await link.boundingBox()).height >= 44, `${route}: small intro target`)
          }
          const item = { route, actualUrl: page.url(), width, theme, overflow, h1: await page.locator('.VPDoc h1').innerText() }
          if ([390,1440].includes(width)) {
            // Bring each lazy image into view, then decode before returning to the top.
            for (const image of await page.locator('.vp-doc img:visible').all()) {
              await image.scrollIntoViewIfNeeded()
              await image.evaluate(el => el.decode())
              assert.ok(await image.evaluate(el => el.naturalWidth > 0))
            }
            await page.evaluate(() => scrollTo(0,0))
            await page.evaluate(() => document.fonts.ready)
            const name = `${route.replace(/^\//,'').replace(/\//g,'-').replace(/-$/,'')}-${width}-${theme}.png`
            await page.screenshot({ path: path.join(dir,name), fullPage: true, animations: 'disabled' })
            item.screenshot = name
          }
          manifest.push(item)
        }
      }
    }
    assert.equal(manifest.filter(r => r.screenshot).length, 48)
    fs.writeFileSync(path.join(dir,'manifest.json'),JSON.stringify({ checkedAt: new Date().toISOString(), checks: manifest },null,2)+'\n')
    console.log('120 route/width/theme checks; 48 decoded full-page captures passed.')
  } finally { await browser.close() }
})().catch(error => { console.error(error); process.exitCode=1 })
