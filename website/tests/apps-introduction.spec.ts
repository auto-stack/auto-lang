import { test, expect } from '@playwright/test'

const topics = ['autoedit', 'autoshell', 'automusk', 'jadeedit']
const widths = [360, 390, 768, 1024, 1440]

for (const prefix of ['', '/zh']) {
  const zh = !!prefix
  for (const topic of ['overview', ...topics]) {
    const route = topic === 'overview' ? `${prefix}/apps` : `${prefix}/apps/${topic}/`
    test(`static application introduction and navigation: ${route}`, async ({ page, request }) => {
      const backendRequests: string[] = []
      const errors: string[] = []
      page.on('request', req => { if (new URL(req.url()).pathname.startsWith('/api/')) backendRequests.push(req.url()) })
      page.on('pageerror', error => errors.push(error.message))
      page.on('console', msg => { if (msg.type() === 'error' || msg.text().includes('[Vue warn]')) errors.push(msg.text()) })
      const response = await request.get(route)
      expect(response.ok()).toBe(true)
      const html = await response.text()
      expect(html).toContain('2026-10-01')
      expect(html).toContain(topic === 'overview' ? 'AutoEdit' : '<h2 id=')
      await page.goto(route)
      const content = page.locator(topic === 'overview' ? '.apps-overview' : '.vp-doc')
      await expect(content.locator('h1')).toHaveCount(1)
      await expect(content.locator('iframe')).toHaveCount(0)
      const imageCount = topic === 'overview' ? 60 : ['autoedit', 'autoshell'].includes(topic) ? 2 : 0
      await expect(content.locator('img')).toHaveCount(imageCount)
      expect((await content.innerText()).length).toBeGreaterThan(600)
      expect(await content.innerText()).not.toMatch(/TODO|Screenshot slot|截图待补|100% Auto/)
      const links = await content.locator('a').evaluateAll(nodes => nodes.map(node => node.getAttribute('href')!).filter(href => href.startsWith('/')))
      for (const href of new Set(links)) {
        const dest = await request.get(href)
        expect(dest.status(), href).toBe(200)
        const body = await dest.text()
        expect(body, href).not.toContain('class="VPNotFound"')
        // Shared galleries and original image assets keep their unprefixed route.
        if (!href.startsWith('/ui/') && !href.endsWith('.png')) expect(href.startsWith(zh ? '/zh/' : '/') && (!zh || href !== '/apps')).toBe(true)
        const hash = href.split('#')[1]
        if (hash) expect(body, `missing anchor ${href}`).toContain(`id="${hash}"`)
      }
      if (topic === 'overview') {
        const cards = content.locator('.main-app-card')
        await expect(cards).toHaveCount(4)
        for (const key of topics.slice(0, 4)) await expect(cards.locator(`a[href="${prefix}/apps/${key}/"]`)).toHaveCount(1)
      } else {
        const id = await content.locator('h2').first().getAttribute('id')
        await expect(page.locator(`.VPDocAsideOutline a[href="#${id}"]`)).toBeAttached()
      }
      await page.locator('header.site-navbar').getByRole('link', { name: zh ? 'English' : '中文', exact: true }).click()
      const other = topic === 'overview' ? `${zh ? '' : '/zh'}/apps` : `${zh ? '' : '/zh'}/apps/${topic}/`
      await expect(page).toHaveURL(new RegExp(other.replace(/\//g, '\\/') + '$'))
      expect(backendRequests).toEqual([])
      expect(errors).toEqual([])
    })
  }

  test(`${prefix || 'en'} introductions remain readable at five widths in both themes`, async ({ page }) => {
    test.setTimeout(120_000)
    for (const dark of [false, true]) {
      await page.goto(prefix + '/apps')
      await page.evaluate(value => localStorage.setItem('vitepress-theme-appearance', value ? 'dark' : 'light'), dark)
      for (const width of widths) {
        await page.setViewportSize({ width, height: 900 })
        for (const route of ['/apps', ...topics.map(topic => `/apps/${topic}/`)]) {
          await page.goto(prefix + route)
          await expect(page.locator('.apps-overview h1, .demo-directory h1, .vp-doc h1')).toBeVisible()
          await expect(page.locator('html')).toHaveClass(dark ? /dark/ : /^(?!.*\bdark\b)/)
          const dimensions = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth }))
          expect(dimensions.scroll, `${prefix + route}, ${width}, ${dark}`).toBeLessThanOrEqual(dimensions.width + 1)
        }
      }
    }
  })

  test(`${prefix || 'en'} Shell introduction and retained evidence guide link both ways`, async ({ page }) => {
    await page.goto(`${prefix}/apps/autoshell/`)
    await page.locator('.vp-doc').getByRole('link', { name: zh ? '使用指南与实跑示例' : 'Usage guide and recorded examples', exact: true }).click()
    await expect(page).toHaveURL(new RegExp(`${prefix}/apps/autoshell/guide/$`))
    await expect(page.locator('img[src="/apps/autoshell/ash-01.png"]').first()).toBeVisible()
    const tabs = page.locator('.script-tabs [role="tab"]')
    await tabs.nth(2).click()
    await expect(page.locator('.script-panel .code-title').first()).toHaveText('user-report.ash')
    await page.getByRole('link', { name: zh ? '← AutoShell 应用介绍' : '← AutoShell introduction', exact: true }).click()
    await expect(page).toHaveURL(new RegExp(`${prefix}/apps/autoshell/$`))
  })
}
