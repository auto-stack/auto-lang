import { test, expect } from '@playwright/test'

test('Chinese home example keeps the Playground language', async ({ page }) => {
  await page.goto('/zh/')
  await page.locator('.demo-playground-link').click()
  await expect(page).toHaveURL(/\/zh\/playground$/)
  await expect(page.locator('.playground-header')).toContainText('运行与转译需本地后端')
})

test('both Chinese docs gallery entrances resolve to the shared application', async ({ page }) => {
  await page.goto('/zh/docs/')
  const links = page.getByRole('link', { name: 'UI 画廊（共享演示）', exact: true })
  await expect(links).toHaveCount(2)
  for (const link of await links.all()) await expect(link).toHaveAttribute('href', '/ui/gallery/index.html')
  await links.last().click()
  await expect(page).toHaveURL(/\/ui\/gallery\/(?:index\.html)?(?:#.*)?$/)
  await expect(page).toHaveTitle(/widgets-gallery/)
  await expect(page.getByText('PAGE NOT FOUND', { exact: true })).toHaveCount(0)
})

for (const prefix of ['', '/zh']) {
  test(`${prefix || 'en'} AutoEdit captures load original pixels and retain keyboard zoom`, async ({ page }) => {
    await page.goto(`${prefix}/apps/autoedit/`)
    for (const id of ['SHOT-01', 'SHOT-02', 'SHOT-03']) {
      const slot = page.locator(`[data-capture-id="${id}"]`)
      const trigger = slot.locator('.evidence-image')
      await trigger.scrollIntoViewIfNeeded()
      await expect.poll(() => trigger.locator('img').evaluate((img: HTMLImageElement) => [img.naturalWidth, img.naturalHeight])).toEqual(id === 'SHOT-03' ? [3840, 2419] : [2560, 1600])
      await trigger.focus()
      await page.keyboard.press('Enter')
      const dialog = slot.locator('dialog[open]')
      await expect(dialog).toBeVisible()
      const original = dialog.locator('.dialog-footer a')
      const originalPath = await original.getAttribute('href')
      const popupReady = page.waitForEvent('popup')
      await original.click()
      const popup = await popupReady
      await popup.waitForLoadState()
      expect(new URL(popup.url()).pathname).toBe(originalPath)
      expect((await popup.request.get(popup.url())).ok()).toBeTruthy()
      await popup.close()
      await page.keyboard.press('Escape')
      await expect(dialog).not.toBeAttached()
      await expect(trigger).toBeFocused()
    }
    await expect(page.locator('[data-capture-id="SHOT-03"]')).toContainText('2026-10-11')
  })

  test(`${prefix || 'en'} release keeps pending scenes distinct from existing evidence`, async ({ page }) => {
    await page.goto(`${prefix}/v05/`)
    const names = await page.locator('#flagship h3').allTextContents()
    expect(names.map(name => name.replace(/^\S+\s+/, ''))).toEqual(['AutoEdit', 'AutoShell', 'AutoMusk', 'AutoJade'])
    const workspace = page.locator('[data-capture-id="SHOT-01"]')
    await expect(workspace.locator('.evidence-image img')).toHaveAttribute('src', '/v05/candidate/autoedit-workspace.png')
    await expect(workspace).not.toContainText('Capture pending')
    await expect(workspace).not.toContainText('截图待补')
    await expect(page.locator('[data-capture-id="SHOT-07"] .evidence-image img')).toHaveAttribute('src', '/apps/auto-jade/p1-main-dark.png')
    for (const id of ['SHOT-04', 'SHOT-10', 'SHOT-11', 'SHOT-12']) {
      const slot = page.locator(`[data-capture-id="${id}"]`)
      await expect(slot).toHaveCount(1)
      await expect(slot.locator('img')).toHaveCount(0)
    }
    await expect(page.locator('img[src="/v05/automusk-app.png"], img[src="/v05/autodown-desktop.png"]')).toHaveCount(0)
    await expect(page.locator('.hero-shot img[src="/desktop-showcase/02-desktop-dark.png"]').first()).toHaveCount(1)
    await expect(page.locator('#flagship img[src="/apps/autoshell/ash-01.png"]').first()).toHaveCount(1)
    await expect(page.locator('#desktop a[href="' + prefix + '/autoos/"]')).toHaveCount(1)
    await expect(page.locator('#ecosystem a[href="' + prefix + '/playground"]')).toHaveCount(1)
    await page.locator('#get-started a').first().click()
    await expect(page.locator('.vp-doc h1')).toBeVisible()
    await expect(page.locator('.vp-doc')).toContainText('cargo build -p auto --release')
  })
}

test('Chinese release points retain the label and body distinction', async ({ page }) => {
  await page.goto('/zh/v05/')
  await expect(page.locator('#desktop .rel-points strong').first()).toHaveText('WM-as-App')
  for (const point of await page.locator('.rel-points li').all()) {
    const text = await point.innerText()
    expect(text.trim()).not.toMatch(/—$/)
    expect((await point.locator('strong').innerText()).length).toBeLessThan(text.length)
  }
})
