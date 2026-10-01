const { chromium } = require('@playwright/test')
const fs = require('node:fs')
const path = require('node:path')
const assert = require('node:assert/strict')
const routes = ['/apps', '/apps/autoedit/', '/apps/autoshell/', '/apps/automusk/', '/apps/jadeedit/', '/apps/demos/', '/apps/autoshell/guide/'].flatMap(route => [route, '/zh' + route])
const dir = path.resolve(__dirname, '../../docs/reports/p722-apps-introduction')
const base = process.env.AUTO_WEBSITE_SHOT_URL || 'http://127.0.0.1:4227'
;(async () => {
  fs.mkdirSync(dir, { recursive: true })
  const browser = await chromium.launch()
  try {
    const page = await browser.newPage({ locale: 'en-US' })
    const manifest = []
    await page.goto(base + '/apps')
    for (const theme of ['light', 'dark']) {
      await page.evaluate(value => localStorage.setItem('vitepress-theme-appearance', value), theme)
      for (const width of [360, 390, 768, 1024, 1440]) {
        await page.setViewportSize({ width, height: 1000 })
        for (const route of routes) {
          await page.goto(base + route)
          const heading = page.locator('main h1')
          await heading.waitFor()
          assert.equal(await heading.count(), 1)
          assert.equal(new URL(page.url()).pathname.replace(/\/$/, ''), route.replace(/\/$/, ''))
          assert.equal(await page.locator('.VPNotFound').count(), 0)
          assert.equal(await page.locator('html').getAttribute('lang'), route.startsWith('/zh') ? 'zh-CN' : 'en-US')
          assert.equal(await page.evaluate(() => document.documentElement.classList.contains('dark')), theme === 'dark')
          const overflow = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)
          assert.ok(overflow <= 1, `${route} ${width} ${theme}: overflow ${overflow}`)
          const headingBox = await heading.boundingBox()
          const navBox = await page.locator('header.site-navbar').boundingBox()
          assert.ok(headingBox.y >= navBox.y + navBox.height, `${route}: heading under navigation`)
          const item = { route, actualUrl: page.url(), width, theme, overflow, h1: await heading.innerText() }
          if ([390, 1440].includes(width)) {
            for (const img of await page.locator('main img:visible').all()) {
              await img.scrollIntoViewIfNeeded()
              await img.evaluate(el => el.decode())
              assert.ok(await img.evaluate(el => el.naturalWidth > 0))
            }
            await page.evaluate(() => scrollTo(0, 0))
            await page.evaluate(() => document.fonts.ready)
            const file = `${route.slice(1).replace(/\//g, '-').replace(/-$/, '')}-${width}-${theme}.png`
            await page.screenshot({ path: path.join(dir, file), fullPage: true, animations: 'disabled' })
            item.screenshot = file
          }
          manifest.push(item)
        }
      }
    }
    assert.equal(manifest.filter(item => item.screenshot).length, 56)
    fs.writeFileSync(path.join(dir, 'manifest.json'), JSON.stringify({ checkedAt: new Date().toISOString(), checks: manifest }, null, 2) + '\n')
    console.log('140 route/width/theme checks; 56 decoded full-page captures passed.')
  } finally { await browser.close() }
})().catch(error => { console.error(error); process.exitCode = 1 })
