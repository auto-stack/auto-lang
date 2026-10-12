import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'

const routes = ['/v05/', '/zh/v05/', '/os', '/zh/os']
const files = ['01-desktop-light.png', '02-desktop-dark.png', '03-launcher-light.png', '04-launcher-dark.png', '05-games-dark.png', '06-productivity-dark.png']

for (const route of routes) {
  test(`all desktop cards, source and keyboard zoom: ${route}`, async ({ page }) => {
    const zh = route.startsWith('/zh/')
    await page.goto(route)
    const story = page.locator('#desktop-showcase')
    const cards = story.locator('.desktop-card')
    await expect(story).toBeVisible()
    await expect(story.getByRole('tab')).toHaveCount(0)
    await expect(cards).toHaveCount(6)
    await expect(story.locator('.desktop-more')).toHaveCount(0)
    let bottom = 0
    for (let i = 0; i < 6; i++) {
      const card = cards.nth(i)
      await expect(card).toBeVisible()
      const image = card.locator('.evidence-image img')
      await image.scrollIntoViewIfNeeded()
      await expect(image).toHaveAttribute('src', `/desktop-showcase/${files[i]}`)
      await expect(image).toHaveAttribute('width', '2560')
      await expect(image).toHaveAttribute('height', '1600')
      await expect(image).toHaveAttribute('loading', 'lazy')
      await expect.poll(() => image.evaluate((el: HTMLImageElement) => [el.naturalWidth, el.naturalHeight])).toEqual([2560, 1600])
      await expect(card.locator('h3')).toBeVisible()
      await expect(card.locator('figcaption')).toContainText('2026-10-01')
      expect((await card.locator('.shot-description').innerText()).length).toBeGreaterThan(15)
      const box = await card.boundingBox()
      const top = box!.y + await page.evaluate(() => scrollY)
      expect(top).toBeGreaterThanOrEqual(bottom)
      bottom = top + box!.height
      const imageBox = await image.boundingBox()
      const copyBox = await card.locator('.desktop-card-copy').boundingBox()
      expect(copyBox!.y).toBeGreaterThan(imageBox!.y + imageBox!.height)
    }
    await expect(cards.first()).toContainText(zh ? '最小化' : 'minimized')
    await expect(cards.nth(4)).toContainText(zh ? '纸牌接龙' : 'Solitaire')
    await expect(cards.last()).toContainText(zh ? '左半屏' : 'left half')
    for (const index of [0, 5]) {
      const card = cards.nth(index)
      const trigger = card.locator('.evidence-image')
      await trigger.scrollIntoViewIfNeeded()
      await trigger.focus()
      await page.keyboard.press('Enter')
      const dialog = card.locator('dialog[open]')
      await expect(dialog).toBeVisible()
      const original = dialog.getByRole('link', { name: zh ? '打开原图' : 'Open original' })
      await expect(original).toHaveAttribute('href', `/desktop-showcase/${files[index]}`)
      if (index === 0) {
        const popupReady = page.waitForEvent('popup')
        await original.click()
        const popup = await popupReady
        await popup.waitForLoadState()
        expect(new URL(popup.url()).pathname).toBe(`/desktop-showcase/${files[index]}`)
        await popup.close()
      }
      await page.keyboard.press('Escape')
      await expect(dialog).toHaveCount(0)
      await expect(trigger).toBeFocused()
    }
    expect(await page.locator('.vp-doc pre').count()).toBe(0)
  })
}

for (const prefix of ['', '/zh']) {
  test(`OS directly includes desktop, applications, settings and architecture: ${prefix || 'en'}`, async ({ page, request }) => {
    const route = prefix + '/os'
    const html = await (await request.get(route)).text()
    for (const file of [...files.map(f => '/desktop-showcase/' + f), '/v05/kanban-web.png', '/v05/kanban-desktop.png', '/v05/autoos-config-agents.png', '/v05/autoos-config-skills.png']) expect(html).toContain(file)
    await page.goto(route)
    await expect(page).toHaveURL(new RegExp(prefix + '/os$'))
    await expect(page.locator('.vp-doc h1')).toHaveCount(1)
    for (const heading of await page.locator('.vp-doc h2').all()) {
      const id = await heading.getAttribute('id')
      await expect(page.locator(`.VPDocAsideOutline a[href="#${id}"]`)).toBeAttached()
    }
    const content = await page.locator('.vp-doc').innerText()
    for (const text of prefix ? ['窗口、工作区与启动', '应用与工作工具', '共享设置', 'Language as OS', 'AI + Lang + OS', '自有内核', '捕获时'] : ['Windows, workspaces, and launch', 'Applications and work tools', 'Shared settings', 'Language as OS', 'AI + Lang + OS', 'own kernel', 'capture time']) expect(content).toContain(text)
    await expect(page.locator('.vp-doc a[href*="/autoos/"]')).toHaveCount(0)
    await page.locator(`.intro-links a[href="${prefix}/os#desktop-showcase"]`).click()
    await expect(page).toHaveURL(new RegExp(prefix + '/os#desktop-showcase$'))
  })
  for (const hash of ['', '#desktop-showcase']) {
    test(`old AutoOS route preserves locale and hash: ${prefix || 'en'}${hash}`, async ({ page, request }) => {
      const html = await (await request.get(prefix + '/autoos/')).text()
      expect(html).toContain('name="robots" content="noindex"')
      expect(html).toContain(`rel="canonical" href="${prefix}/os"`)
      await page.goto(prefix + '/autoos/' + hash)
      await expect(page).toHaveURL(new RegExp(prefix + '/os#desktop-showcase$'))
      await expect(page.locator('.desktop-card')).toHaveCount(6)
      await expect(page.locator('.vp-doc h1')).toHaveCount(1)
    })
  }
}

test('OS and release cards: five widths, both themes and 16 visual captures', async ({ page }) => {
  test.setTimeout(180_000)
  const dir = process.env.AUTO_WEBSITE_SHOT_DIR || path.resolve(process.cwd(), '../scratch/p767-os-cards')
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
        const cards = story.locator('.desktop-card')
        await expect(cards).toHaveCount(6)
        await expect(story.getByRole('tab')).toHaveCount(0)
        const boxes = await cards.evaluateAll(nodes => nodes.map(el => { const b = el.getBoundingClientRect(); return { x: b.x, y: b.y, width: b.width, height: b.height } }))
        for (let i = 1; i < boxes.length; i++) {
          expect(Math.abs(boxes[i].x - boxes[0].x)).toBeLessThan(1)
          expect(Math.abs(boxes[i].width - boxes[0].width)).toBeLessThan(1)
          expect(boxes[i].y).toBeGreaterThanOrEqual(boxes[i - 1].y + boxes[i - 1].height)
        }
        expect(await page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(theme === 'dark')
        expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth), `${route} ${width} ${theme}`).toBeLessThanOrEqual(1)
        expect(await page.locator('.VPNotFound, .NotFound').count()).toBe(0)
        expect(new URL(page.url()).pathname.replace(/\/$/, '')).toBe(route.replace(/\/$/, ''))
        if (width === 390 || width === 1440) {
          for (const image of await story.locator('.evidence-image img').all()) {
            await image.scrollIntoViewIfNeeded()
            await image.evaluate((el: HTMLImageElement) => el.decode())
          }
          const name = `${route.replace(/^\//, '').replace(/\//g, '-')}-${width}-${theme}.png`
          await story.screenshot({ path: path.join(dir, name) })
          await cards.first().screenshot({ path: path.join(dir, name.replace('.png', '-first.png')) })
          manifest.push({ route, url: page.url(), width, theme, locale: route.startsWith('/zh/') ? 'zh' : 'en', screenshot: name, images: files, valid: true })
        }
      }
    }
  }
  expect(manifest).toHaveLength(16)
  fs.writeFileSync(path.join(dir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
})
