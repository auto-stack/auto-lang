import { test, expect } from '@playwright/test'

// Phase landing audit: T-01..T-04 only. Loading behavior remains T-05..T-08.
for (const prefix of ['', '/zh']) {
  const zh = !!prefix
  test(`${prefix || 'en'} learning hubs preserve real destinations and eight books`, async ({ page, request }) => {
    for (const section of ['docs', 'books']) {
      await page.goto(`${prefix}/${section}/`)
      const hub = page.locator('.learning-hub')
      await expect(hub).toBeVisible()
      await expect(page.locator('.VPDocFooter .next')).toHaveCount(0)
      await expect(page.locator('.edit-link')).toHaveCount(0)
      const links = await hub.locator('a').evaluateAll(nodes => nodes.map(n => n.getAttribute('href')!))
      expect(links.length).toBeGreaterThan(7)
      for (const href of new Set(links)) {
        const response = await request.get(href)
        expect(response.status(), href).toBe(200)
        expect(await response.text(), href).not.toContain('class="VPNotFound"')
      }
      if (section === 'books') {
        const destinations = new Set(links.map(href => href.match(/\/books\/([^/]+)/)?.[1]).filter(Boolean))
        expect(destinations.size).toBe(8)
      }
    }
  })

  test(`${prefix || 'en'} chapter navigation, editing source, clipboard and keyboard runner`, async ({ page, context }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write'])
    await page.goto(`${prefix}/books/tapl/ch01-getting-started`)
    await expect(page.locator('.rc-toc')).toHaveAttribute('href', `${prefix}/books/tapl/`)
    const footerLinks = await page.locator('.VPDocFooter .pager-link').evaluateAll(nodes => nodes.map(n => n.getAttribute('href')!))
    expect(footerLinks.length).toBeGreaterThan(0)
    expect(footerLinks.every(href => href.startsWith(`${prefix}/books/tapl/`))).toBeTruthy()
    await expect(page.locator('.edit-link a')).toHaveAttribute('href', /gitee\.com\/auto-stack\/book\/edit\/master\/tapl\/ch01-getting-started(\.cn)?\.md/)
    const fence = page.locator('.auto-fence').filter({ has: page.locator('.af-run') }).first()
    const run = fence.locator('.af-run')
    await expect(run).toHaveText(zh ? '运行' : 'Run')
    await expect(fence.locator('button.copy')).toHaveAttribute('title', zh ? '复制代码' : 'Copy Code')
    const source = (await fence.locator('pre code').innerText()).trimEnd()
    await fence.hover()
    await fence.locator('button.copy').click({ force: true })
    // Clipboard preserves Windows CRLF; rendered innerText uses LF.
    await expect.poll(async () => (await page.evaluate(() => navigator.clipboard.readText())).replace(/\r\n/g, '\n').trimEnd()).toBe(source)
    await run.focus()
    await page.keyboard.press('Enter')
    await expect(run).toHaveAttribute('aria-expanded', 'true')
    await expect(fence.locator('.af-runner')).toBeVisible()
    const controlled = await run.getAttribute('aria-controls')
    await expect(fence.locator('.af-runner')).toHaveAttribute('id', controlled!)
    await run.focus()
    await page.keyboard.press('Enter')
    await expect(run).toHaveAttribute('aria-expanded', 'false')
    await expect(fence.locator('pre code')).toBeVisible()
  })

  test(`${prefix || 'en'} mobile reader clears navbar and toolbar clears source`, async ({ page }) => {
    for (const width of [360, 390, 768, 1024, 1440]) {
      await page.setViewportSize({ width, height: 844 })
      await page.goto(`${prefix}/books/tapl/ch01-getting-started`)
      await expect(page.locator('.auto-fence').first()).toBeVisible()
      expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1)
      const fence = page.locator('.auto-fence').first()
      const toolbar = await fence.locator('.af-toolbar').boundingBox()
      const source = await fence.locator('.af-original').boundingBox()
      expect(toolbar!.y + toolbar!.height).toBeLessThanOrEqual(source!.y + 1)
      await page.evaluate(() => window.scrollTo(0, 800))
      if (width < 1280) {
        const nav = await page.locator('header.site-navbar').boundingBox()
        const local = await page.locator('.VPLocalNav').boundingBox()
        expect(local!.y).toBeGreaterThanOrEqual(nav!.y + nav!.height - 2)
      }
    }
  })
}
