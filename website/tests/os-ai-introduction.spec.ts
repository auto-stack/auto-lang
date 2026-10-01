import { test, expect } from '@playwright/test'

const paths = ['/os', '/ai', '/autoos/', '/ui-desktop', '/articles/autoos-history', '/articles/auto-ai-history']
for (const zh of [false, true]) {
  for (const path of paths) {
    const route = (zh ? '/zh' : '') + path
    test(`readable introduction, sources, outline and locale: ${route}`, async ({ page, request }) => {
      const errors: string[] = []
      page.on('pageerror', error => errors.push(error.message))
      page.on('console', message => { if (message.text().includes('[Vue warn]')) errors.push(message.text()) })
      const response = await request.get(route)
      expect(response.ok()).toBe(true)
      const html = await response.text()
      // Content survives without client execution; prose must be emitted by SSG.
      expect(html).toContain('2026')
      expect(html).toContain('<h2 id=')
      await page.goto(route)
      await expect(page.locator('.VPDoc h1')).toHaveCount(1)
      await expect(page.locator('.VPNotFound')).toHaveCount(0)
      expect((await page.locator('.vp-doc').innerText()).length).toBeGreaterThan(500)
      await expect(page.locator('.vp-doc h2').first()).toBeVisible()
      const article = path.includes('/articles/')
      if (path === '/os' || path === '/ai') {
        const diagram = page.locator('.system-relation ol')
        await expect(diagram).toHaveAttribute('aria-label', /.+/)
        await expect(diagram.locator('li')).toHaveCount(5)
        await expect(page.locator('.system-relation figcaption')).toBeVisible()
      }
      if (article) {
        const firstHeading = page.locator('.vp-doc h2').first()
        const id = await firstHeading.getAttribute('id')
        await expect(page.locator(`.VPDocAsideOutline a[href="#${id}"]`)).toBeAttached()
        await expect(page.locator('.vp-doc a[href^="https://github.com/"]').first()).toBeAttached()
        const overview = path.includes('autoos') ? '/os' : '/ai'
        await page.locator(`.vp-doc a[href="${zh ? '/zh' : ''}${overview}"]`).first().click()
        await expect(page).toHaveURL(new RegExp(`${zh ? '/zh' : ''}${overview}$`))
        const target = (zh ? '/zh' : '') + path
        await page.locator(`.intro-links a[href="${target}"]`).click()
        await expect(page).toHaveURL(new RegExp(`${target}$`))
      } else {
        for (const link of await page.locator('.intro-links a').all()) {
          const href = await link.getAttribute('href')
          expect(href?.startsWith(zh ? '/zh/' : '/')).toBe(true)
          const result = await request.get(href!)
          expect(result.ok(), href!).toBe(true)
          expect(await result.text()).not.toContain('class="VPNotFound"')
        }
      }
      // The navbar language switch must retain this topic, including new articles.
      const localeLink = page.locator('header.site-navbar a').filter({ hasText: zh ? 'English' : '中文' }).first()
      await expect(localeLink).toHaveAttribute('href', (zh ? '' : '/zh') + path)
      await localeLink.click()
      await expect(page).toHaveURL(new RegExp(`${zh ? '' : '/zh'}${path.replace(/\/$/, '/?')}$`))
      await expect(page.locator('.VPDoc h1')).toBeVisible()
      expect(errors).toEqual([])
    })
  }
  test(`search finds overview and history in ${zh ? 'Chinese' : 'English'}`, async ({ page }) => {
    await page.goto(zh ? '/zh/os' : '/os')
    const input = page.locator('.VPLocalSearchBox input')
    for (let i = 0; i < 8 && !await input.isVisible(); i++) {
      await page.getByRole('button', { name: /Search \(Ctrl\+K\)|搜索（Ctrl\+K）/ }).first().click()
      await page.waitForTimeout(300)
    }
    await expect(input).toBeVisible()
    await input.fill('Language as OS')
    const results = page.locator('.VPLocalSearchBox .results')
    await expect(results).toBeVisible({ timeout: 20000 })
    await expect(results.locator(`a[href^="${zh ? '/zh' : ''}/articles/autoos-history"]`).first()).toBeVisible()
    await input.fill('aaid')
    await expect(results.locator(`a[href^="${zh ? '/zh' : ''}/ai"]`).first()).toBeVisible()
  })
}

for (const prefix of ['', '/zh']) {
  test(`home and release keep product entry points and the corrected gallery: ${prefix || 'English'}`, async ({ page }) => {
    await page.goto(prefix + '/')
    await page.locator(`.features-section a[href="${prefix}/os"]`).click()
    await expect(page.locator('.introduction-header h1')).toBeVisible()
    await page.goto(prefix + '/v05/')
    await expect(page.locator('#desktop-showcase').getByRole('tab')).toHaveCount(6)
    await expect(page.locator('.hero-shot .evidence-image img')).toHaveAttribute('src', '/desktop-showcase/02-desktop-dark.png')
    const philosophy = page.locator('#philosophy')
    await page.locator('#ph-laos summary').click()
    await expect(philosophy).toContainText('LaOS → OS over OS → AI + Lang + OS')
  })
}
