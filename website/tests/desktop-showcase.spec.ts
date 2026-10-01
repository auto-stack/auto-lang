import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'

const routes = ['/v05/', '/zh/v05/', '/autoos/', '/zh/autoos/', '/os', '/zh/os']
const files = ['01-desktop-light.png', '02-desktop-dark.png', '03-launcher-light.png', '04-launcher-dark.png', '05-games-dark.png', '06-productivity-dark.png']

for (const route of routes) {
  test(`real desktop sequence, source and keyboard zoom: ${route}`, async ({ page }) => {
    const zh = route.startsWith('/zh/')
    const preview = route.endsWith('/os')
    await page.goto(route)
    const story = page.locator('#desktop-showcase')
    await expect(story).toBeVisible()
    const tabs = story.getByRole('tab')
    await expect(tabs).toHaveCount(preview ? 2 : 6)
    const image = story.locator('.evidence-image img')
    for (let i = 0; i < (preview ? 2 : 6); i++) {
      await tabs.nth(i).click()
      await expect(image).toHaveAttribute('src', `/desktop-showcase/${files[i]}`)
      await expect(image).toHaveAttribute('width', '2560')
      await expect(image).toHaveAttribute('height', '1600')
      await expect.poll(() => image.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth === 2560)).toBe(true)
    }
    await tabs.first().click()
    await expect(story.locator('figcaption')).toContainText(zh ? '最小化' : 'minimized')
    await expect(image).toHaveAttribute('loading', 'lazy')
    if (preview) {
      await expect(story.locator('.desktop-more')).toHaveAttribute('href', `${zh ? '/zh' : ''}/autoos/#desktop-showcase`)
    } else {
      await expect(story.locator('.desktop-steps li')).toHaveCount(4)
      await expect(story.locator('.desktop-steps li').nth(2)).toContainText(zh ? '纸牌接龙' : 'Solitaire')
      await expect(story.locator('.desktop-steps li').last()).toContainText(zh ? '左半屏' : 'left half')
    }
    const trigger = story.locator('.evidence-image')
    await trigger.focus()
    await page.keyboard.press('Enter')
    const dialog = story.locator('dialog[open]')
    await expect(dialog).toBeVisible()
    await expect(dialog.getByRole('link', { name: zh ? '打开原图' : 'Open original' })).toHaveAttribute('href', '/desktop-showcase/01-desktop-light.png')
    await page.keyboard.press('Escape')
    await expect(dialog).toHaveCount(0)
    await expect(trigger).toBeFocused()
    expect(await page.locator('.vp-doc pre').count()).toBe(0)
  })
}

test('six pages: five widths, both themes, and 24 verified visual captures', async ({ page }) => {
  test.setTimeout(180_000)
  const dir = path.resolve(process.cwd(), '../docs/reports/p720-desktop-showcase')
  fs.mkdirSync(dir, { recursive: true })
  const manifest: unknown[] = []
  await page.goto('/')
  for (const theme of ['light', 'dark']) {
    await page.evaluate(t => localStorage.setItem('vitepress-theme-appearance', t), theme)
    for (const width of [360, 390, 768, 1024, 1440]) {
      await page.setViewportSize({ width, height: 1000 })
      for (const route of routes) {
        await page.goto(route)
        const story = page.locator('#desktop-showcase')
        await expect(story).toBeVisible()
        await story.scrollIntoViewIfNeeded()
        await expect.poll(() => story.locator('.evidence-image img').evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth === 2560)).toBe(true)
        expect(await page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(theme === 'dark')
        expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth), `${route} ${width} ${theme}`).toBeLessThanOrEqual(1)
        expect(await page.locator('.VPNotFound, .NotFound').count()).toBe(0)
        expect(new URL(page.url()).pathname.replace(/\/$/, '')).toBe(route.replace(/\/$/, ''))
        if (width === 390 || width === 1440) {
          const name = `${route.replace(/^\//, '').replace(/\//g, '-') || 'root'}-${width}-${theme}.png`
          await story.screenshot({ path: path.join(dir, name) })
          manifest.push({ route, url: page.url(), width, theme, locale: route.startsWith('/zh/') ? 'zh' : 'en', body: await story.innerText(), screenshot: `docs/reports/p720-desktop-showcase/${name}`, image: files[0], valid: true })
        }
      }
    }
  }
  expect(manifest).toHaveLength(24)
  fs.writeFileSync(path.join(dir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
})
