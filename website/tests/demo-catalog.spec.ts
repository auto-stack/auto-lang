import { test, expect } from '@playwright/test'
import fs from 'node:fs'
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
  test(`${prefix || 'en'} overview contains the SSR catalog, filters and keyboard image controls`, async ({ page, request }) => {
    const errors: string[] = [], business: string[] = []
    page.on('pageerror', error => errors.push(error.message))
    page.on('request', req => { if (new URL(req.url()).pathname.includes('/api/')) business.push(req.url()) })
    const html = await (await request.get(prefix + '/apps')).text()
    for (const row of rows) {
      expect(html).toContain(`data-demo-id="${row.id}"`)
      expect(html).toContain(row[zh ? 'zh' : 'en'].requirements)
    }
    await page.goto(prefix + '/apps')
    await expect(page.locator('h1')).toHaveCount(1)
    await expect(page.locator('.main-app-card')).toHaveCount(4)
    await expect(page.locator('.demo-card')).toHaveCount(28)
    await expect(page.locator('.demo-category')).toHaveCount(6)
    const filters = page.locator('.demo-filters button')
    for (const [index, group] of groups.entries()) {
      await filters.nth(index + 1).click()
      await expect(filters.nth(index + 1)).toHaveAttribute('aria-pressed', 'true')
      await expect(page.locator('.demo-card')).toHaveCount(rows.filter(row => row.category === group).length)
    }
    await filters.first().focus()
    await page.keyboard.press('Enter')
    await expect(page.locator('.demo-card')).toHaveCount(28)
    const trigger = page.locator('.demo-card .evidence-image').first()
    await trigger.focus()
    await page.keyboard.press('Enter')
    await expect(page.locator('dialog[open]')).toHaveCount(1)
    await expect(page.locator('dialog[open]').getByRole('button', { name: zh ? '关闭' : 'Close', exact: false })).toBeFocused()
    await page.keyboard.press('Escape')
    await expect(page.locator('dialog[open]')).toHaveCount(0)
    await expect(trigger).toBeFocused()
    for (const row of rows) {
      const card = page.locator(`#demo-${row.slug}`)
      const summary = card.locator('summary')
      await summary.focus()
      await page.keyboard.press('Enter')
      await expect(card.locator('details')).toHaveAttribute('open', '')
      await expect(card.locator('ol li')).toHaveCount(3)
      await expect(card.locator('.demo-expanded')).toContainText(row[zh ? 'zh' : 'en'].requirements)
      await expect(card.locator('.demo-expanded')).toContainText(row[zh ? 'zh' : 'en'].state)
      await expect(card.locator(`a[href^="https://github.com/auto-stack/${row.sourceRepo}/"]`)).toHaveCount(1)
      const image = card.locator('.evidence-image img')
      await image.scrollIntoViewIfNeeded()
      await expect.poll(() => image.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)).toBe(true)
      await summary.click()
    }
    await expect(page.locator('.demo-card a[href*="/apps/demos/"]:not([href$=".png"])')).toHaveCount(0)
    await expect(page.locator('iframe')).toHaveCount(0)
    await page.locator('header.site-navbar').getByRole('link', { name: zh ? 'English' : '中文', exact: true }).click()
    await expect(page).toHaveURL(new RegExp(`${zh ? '' : '/zh'}/apps$`))
    await expect(page.locator('.demo-card')).toHaveCount(28)
    expect(errors).toEqual([])
    expect(business).toEqual([])
  })

  test(`${prefix || 'en'} legacy directory and 28 topics lead to expanded overview anchors`, async ({ page, request }) => {
    test.setTimeout(180_000)
    const directory = prefix + '/apps/demos/'
    await page.goto(directory)
    await expect(page).toHaveURL(new RegExp(`${prefix}/apps#system-apps$`))
    for (const row of rows) {
      const target = `${prefix}/apps#demo-${row.slug}`
      const old = `${prefix}/apps/demos/${row.slug}/`
      const html = await (await request.get(old)).text()
      expect(html).toContain(target)
      expect(html).toContain('noindex')
      expect(html).not.toContain('class="demo-card"')
      await page.goto(old)
      await expect(page).toHaveURL(new RegExp(`${target}$`))
      await expect(page.locator(`#demo-${row.slug} details`)).toHaveAttribute('open', '')
      await expect(page.locator(`#demo-${row.slug}`)).toBeInViewport()
    }
    await page.goto(`${prefix}/apps#demo-terminal`)
    await expect(page.locator('#demo-terminal details')).toHaveAttribute('open', '')
    await page.locator('.demo-filters button').nth(1).click()
    await expect(page.locator('#demo-terminal')).toHaveCount(0)
    await page.evaluate(() => { location.hash = 'demo-settings' })
    await expect(page.locator('#demo-settings details')).toHaveAttribute('open', '')
    await expect(page.locator('.demo-card')).toHaveCount(28)
    await page.locator('#demo-calculator .demo-permalink').click()
    await expect(page.locator('#demo-calculator details')).toHaveAttribute('open', '')
  })

  test(`${prefix || 'en'} merged overview and expanded cards fit five widths and both themes`, async ({ page }) => {
    test.setTimeout(120_000)
    fs.mkdirSync('.p723-runtime/r2-pages', { recursive: true })
    for (const dark of [false, true]) {
      await page.goto(prefix + '/apps')
      await page.evaluate(value => localStorage.setItem('vitepress-theme-appearance', value), dark ? 'dark' : 'light')
      for (const width of [360, 390, 768, 1024, 1440]) {
        await page.setViewportSize({ width, height: 900 })
        await page.goto(prefix + '/apps')
        await expect(page.locator('html')).toHaveClass(dark ? /dark/ : /^(?!.*\bdark\b)/)
        const columns = await page.locator('.demo-grid').first().evaluate(el => getComputedStyle(el).gridTemplateColumns.split(' ').length)
        expect(columns).toBe(width <= 640 ? 1 : width < 1024 ? 2 : 3)
        await page.locator('.demo-more').evaluateAll(nodes => nodes.forEach((node: HTMLDetailsElement) => { node.open = true }))
        expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${prefix}/apps ${width} ${dark}`).toBe(true)
        await page.locator('#demo-calculator').scrollIntoViewIfNeeded()
        await page.locator('#demo-calculator .evidence-image img').evaluate((img: HTMLImageElement) => img.decode())
        await page.screenshot({ path: `.p723-runtime/r2-pages/${zh ? 'zh' : 'en'}-${width}-${dark ? 'dark' : 'light'}.png` })
      }
    }
  })
}
