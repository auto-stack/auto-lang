import { test, expect } from '@playwright/test'

for (const prefix of ['', '/zh']) {
  test(`${prefix || 'en'} AutoJade canonical page and legacy URL preserve locale and location`, async ({ page, request }) => {
    const response = await request.get(`${prefix}/apps/auto-jade/`)
    expect(response.ok()).toBeTruthy()
    expect(await response.text()).toContain('AutoJade')
    await page.goto(`${prefix}/apps/jadeedit/?from=legacy#connect-pages-as-well-as-files`)
    await expect(page).toHaveURL(`${prefix}/apps/auto-jade/?from=legacy#connect-pages-as-well-as-files`)
    await expect(page.locator('h1')).toContainText('AutoJade')
    for (const route of ['/', '/v05/', '/apps']) {
      await page.goto(prefix + route)
      await expect(page.locator('a[href="' + prefix + '/apps/auto-jade/"]').first()).toBeVisible()
      await expect(page.locator('[data-capture-id="SHOT-07"] .evidence-image img')).toHaveAttribute('data-original-src', '/apps/auto-jade/p1-main-dark.png')
      await expect(page.locator('a[href*="/apps/jadeedit/"]')).toHaveCount(0)
    }
  })

  test(`${prefix || 'en'} AutoJade originals, gallery navigation and pending recovery`, async ({ page, request }) => {
    await page.goto(`${prefix}/apps/auto-jade/`)
    const hero = page.locator('[data-capture-id="SHOT-07"] .evidence-image img')
    await expect.poll(() => hero.evaluate((img: HTMLImageElement) => [img.naturalWidth, img.naturalHeight])).toEqual([1933, 1245])
    const gallery = page.locator('[data-capture-id="SHOT-08"]')
    const files = ['p2-backlinks-dark.png', 'p3-wikilink-dark.png', 'p5-quick-open-dark.png', 'p6-dangling-create-dark.png', 'p7-filetree-dark.png']
    await gallery.getByRole('tab').first().focus()
    for (let i = 0; i < files.length; i++) {
      const tab = gallery.getByRole('tab').nth(i)
      await expect(tab).toHaveAttribute('aria-selected', 'true')
      await expect(tab).toBeFocused()
      const trigger = gallery.locator('.evidence-image')
      const src = `/apps/auto-jade/${files[i]}`
      await expect(trigger.locator('img')).toHaveAttribute('data-original-src', src)
      const expected = i === 4 ? [1933, 1245] : [1440, 900]
      await expect.poll(() => trigger.locator('img').evaluate((img: HTMLImageElement) => [img.naturalWidth, img.naturalHeight])).toEqual(expected)
      await trigger.focus()
      await page.keyboard.press('Enter')
      await expect(gallery.locator('dialog[open]')).toBeVisible()
      const original = gallery.locator('.dialog-footer a')
      await expect(original).toHaveAttribute('href', src)
      expect((await request.get(src)).ok()).toBeTruthy()
      // Native close first restores focus, then emits the close event; wait for
      // the completed close before issuing the next user action.
      await gallery.locator('dialog').evaluate(el => {
        el.removeAttribute('data-close-complete')
        el.addEventListener('close', () => el.setAttribute('data-close-complete', 'true'), { once: true })
      })
      await page.keyboard.press('Escape')
      await expect(gallery.locator('dialog')).toHaveAttribute('data-close-complete', 'true')
      await expect(trigger).toBeFocused()
      await tab.focus()
      if (i < files.length - 1) await page.keyboard.press('ArrowRight')
    }
    await expect(page.locator('[data-capture-id="SHOT-09"]')).toContainText(prefix ? '截图待补' : 'Capture pending')
    await expect(page.locator('[data-capture-id="SHOT-09"] img')).toHaveCount(0)
    await expect(page.locator('img[src*="p4-tags"]')).toHaveCount(0)
  })
}
