import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import crypto from 'node:crypto'

const rows = JSON.parse(fs.readFileSync('.vitepress/theme/data/demos.json', 'utf8'))
const receipt = JSON.parse(fs.readFileSync('../docs/reports/p723-capture-catalog.json', 'utf8'))
const frozen = JSON.parse(fs.readFileSync('../docs/reports/p722-source-baseline.json', 'utf8')).demo_candidates
const groups = ['daily', 'reading', 'media', 'system', 'games', 'structure']

test('frozen 28-entry catalog has real, versioned, unchanged image assets', async () => {
  expect(rows).toHaveLength(28)
  expect(new Set(rows.map(row => row.id)).size).toBe(28)
  expect(new Set(rows.map(row => row.slug)).size).toBe(28)
  expect(receipt.items.map(row => row.id).sort()).toEqual(rows.map(row => row.id).sort())
  expect(rows.map(row => `${row.sourceRepo}:${row.sourcePath}`).sort()).toEqual(frozen.map(row => `${row.repo}:${row.path}`).sort())
  for (const row of rows) {
    const capture = receipt.items.find(item => item.id === row.id)
    const image = fs.readFileSync('public' + row.image.src)
    expect(image.subarray(1, 4).toString()).toBe('PNG')
    expect(crypto.createHash('sha256').update(image).digest('hex')).toBe(capture.sha256)
    expect(row.image.width).toBe(image.readUInt32BE(16))
    expect(row.image.height).toBe(image.readUInt32BE(20))
    expect(capture.qualityCheck).toMatch(/^pass:/)
    expect(capture.runtime).toBeTruthy()
    expect(capture.captureDate || capture.reusedAt).toBeTruthy()
    expect(groups).toContain(row.category)
    for (const locale of ['en', 'zh']) {
      expect(row[locale].steps).toHaveLength(3)
      expect(row[locale].requirements.length).toBeGreaterThan(12)
      expect(row[locale].state.length).toBeGreaterThan(12)
    }
  }
})

for (const prefix of ['', '/zh']) {
  const zh = !!prefix
  test(`${prefix || 'en'} catalog SSR, category filtering, keyboard enlargement and focus return`, async ({ page, request }) => {
    const errors: string[] = [], business: string[] = []
    page.on('pageerror', error => errors.push(error.message))
    page.on('request', req => { if (/\/(api|apps\/[^/]+\/api)\//.test(new URL(req.url()).pathname)) business.push(req.url()) })
    const html = await (await request.get(prefix + '/apps/demos/')).text()
    for (const row of rows) expect(html).toContain(`data-demo-id="${row.id}"`)
    await page.goto(prefix + '/apps/demos/')
    await expect(page.locator('.demo-card')).toHaveCount(28)
    await expect(page.locator('.demo-category')).toHaveCount(6)
    const filters = page.locator('.demo-filters button')
    for (const [index, group] of groups.entries()) {
      await filters.nth(index + 1).click()
      await expect(filters.nth(index + 1)).toHaveAttribute('aria-pressed', 'true')
      await expect(page.locator('.demo-card')).toHaveCount(rows.filter(row => row.category === group).length)
      await expect(page.locator('.demo-category')).toHaveCount(1)
    }
    await filters.first().focus()
    await page.keyboard.press('Enter')
    await expect(page.locator('.demo-card')).toHaveCount(28)
    const trigger = page.locator('.demo-card .evidence-image').first()
    await trigger.focus()
    await page.keyboard.press('Enter')
    const dialog = page.locator('dialog[open]')
    await expect(dialog).toHaveCount(1)
    await expect(dialog.getByRole('button', { name: zh ? '关闭' : 'Close', exact: false })).toBeFocused()
    await page.keyboard.press('Escape')
    await expect(dialog).toHaveCount(0)
    await expect(trigger).toBeFocused()
    for (const image of await page.locator('.demo-card .evidence-image img').all()) {
      await image.scrollIntoViewIfNeeded()
      await expect.poll(() => image.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)).toBe(true)
    }
    expect(errors).toEqual([])
    expect(business).toEqual([])
  })

  test(`${prefix || 'en'} all 28 reading topics have loaded images, source, outline and reciprocal locale`, async ({ page, request }) => {
    test.setTimeout(180_000)
    const business: string[] = [], errors: string[] = []
    page.on('request', req => { if (new URL(req.url()).pathname.includes('/api/')) business.push(req.url()) })
    page.on('pageerror', error => errors.push(error.message))
    const captureDir = '.p723-runtime/pages'
    fs.mkdirSync(captureDir, { recursive: true })
    for (const row of rows) {
      const route = `${prefix}/apps/demos/${row.slug}/`
      const response = await request.get(route)
      expect(response.ok(), route).toBe(true)
      const html = await response.text()
      expect(html).toContain(row[zh ? 'zh' : 'en'].name)
      expect(html).toContain('v0.5.1')
      await page.goto(route)
      const content = page.locator('.vp-doc')
      await expect(content.locator('h1')).toHaveCount(1)
      await expect(content.locator('h2')).toHaveCount(4)
      await expect(content.locator('iframe')).toHaveCount(0)
      const image = content.locator('.evidence-image img')
      await expect(image).toHaveAttribute('src', row.image.src)
      await expect.poll(() => image.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)).toBe(true)
      await expect(content.locator('li')).toHaveCount(3)
      await expect(content.locator(`a[href^="https://github.com/auto-stack/${row.sourceRepo}/"]`)).toHaveCount(1)
      const links = await content.locator('a').evaluateAll(nodes => nodes.map(node => node.getAttribute('href')!).filter(href => href.startsWith('/') && !href.endsWith('.png')))
      for (const link of new Set(links)) expect((await request.get(link as string)).ok(), link as string).toBe(true)
      const id = await content.locator('h2').first().getAttribute('id')
      await expect(page.locator(`.VPDocAsideOutline a[href="#${id}"]`)).toBeAttached()
      await page.screenshot({ path: path.join(captureDir, `${zh ? 'zh' : 'en'}-${row.slug}.png`), fullPage: true })
      await page.locator('header.site-navbar').getByRole('link', { name: zh ? 'English' : '中文', exact: true }).click()
      await expect(page).toHaveURL(new RegExp(`${zh ? '' : '/zh'}/apps/demos/${row.slug}/$`))
    }
    expect(business).toEqual([])
    expect(errors).toEqual([])
  })

  test(`${prefix || 'en'} catalog and every detail fit five widths in both themes`, async ({ page }) => {
    test.setTimeout(300_000)
    for (const dark of [false, true]) {
      await page.goto(prefix + '/apps/demos/')
      await page.evaluate(value => localStorage.setItem('vitepress-theme-appearance', value), dark ? 'dark' : 'light')
      for (const width of [360, 390, 768, 1024, 1440]) {
        await page.setViewportSize({ width, height: 900 })
        for (const route of ['/apps/demos/', ...rows.map(row => `/apps/demos/${row.slug}/`)]) {
          await page.goto(prefix + route)
          await expect(page.locator('.demo-directory h1, .vp-doc h1')).toBeVisible()
          await expect(page.locator('html')).toHaveClass(dark ? /dark/ : /^(?!.*\bdark\b)/)
          expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${prefix + route} ${width} ${dark}`).toBe(true)
          if (route === '/apps/demos/') {
            const columns = await page.locator('.demo-grid').first().evaluate(el => getComputedStyle(el).gridTemplateColumns.split(' ').length)
            expect(columns).toBe(width <= 640 ? 1 : width < 1024 ? 2 : 3)
            for (const image of await page.locator('.demo-card .evidence-image img').all()) {
              await image.scrollIntoViewIfNeeded()
              await image.evaluate((img: HTMLImageElement) => img.decode())
            }
            await page.evaluate(() => scrollTo(0, 0))
            await page.screenshot({ path: `.p723-runtime/pages/${zh ? 'zh' : 'en'}-catalog-${width}-${dark ? 'dark' : 'light'}.png`, fullPage: true })
          }
        }
      }
    }
  })
}
