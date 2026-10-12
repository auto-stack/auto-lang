import { test, expect } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import crypto from 'node:crypto'
import { fileURLToPath } from 'node:url'

const websiteDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const publicDir = path.join(websiteDir, 'public')
const catalog = JSON.parse(fs.readFileSync(path.resolve(websiteDir, '../docs/reports/p769-image-catalog.json'), 'utf8'))
const sourcePaths = new Set<string>(catalog.assets.map((row: { source: string }) => row.source))
const hash = (data: Buffer) => crypto.createHash('sha256').update(data).digest('hex')

test('delivery budget and immutable capture provenance', async () => {
  expect(catalog.assets.length).toBe(35)
  for (const row of catalog.assets) {
    const capture = fs.readFileSync(path.join(publicDir, row.source))
    const delivery = fs.readFileSync(path.join(publicDir, row.output))
    expect(hash(capture)).toBe(row.source_sha256)
    expect(hash(delivery)).toBe(row.sha256)
    expect(delivery.length).toBe(row.bytes)
    expect(delivery.length).toBeLessThan(1_000_000)
    expect(delivery.length).toBeLessThan(capture.length)
  }
})

for (const route of ['/zh/os', '/v05/', '/zh/apps']) {
  test(`no original preloads; compressed zoom loads and unloads: ${route}`, async ({ page }) => {
    const originals: string[] = []
    page.on('request', request => {
      const pathname = new URL(request.url()).pathname
      if (sourcePaths.has(pathname)) originals.push(pathname)
    })
    await page.goto(route)
    await expect(page.locator('.evidence-dialog img')).toHaveCount(0)
    const image = page.locator('.evidence-image img[src$=".webp"]').first()
    await image.scrollIntoViewIfNeeded()
    await image.evaluate(async (img: HTMLImageElement) => { await img.decode() })
    const delivery = await image.getAttribute('src')
    const original = await image.getAttribute('data-original-src')
    const trigger = image.locator('..').locator('..')
    await trigger.focus()
    await page.keyboard.press('Enter')
    const dialog = page.locator('.evidence-dialog[open]')
    await expect(dialog).toBeVisible()
    await expect(dialog.locator('img')).toHaveAttribute('src', delivery!)
    await dialog.locator('img').evaluate(async (img: HTMLImageElement) => { await img.decode() })
    await expect(dialog.locator('a')).toHaveAttribute('href', original!)
    expect(originals).toEqual([])
    await page.keyboard.press('Escape')
    await expect(page.locator('.evidence-dialog img')).toHaveCount(0)
    await expect(trigger).toBeFocused()
    await page.keyboard.press('Enter')
    await expect(dialog).toBeVisible()
    await dialog.locator('button').click()
    await expect(page.locator('.evidence-dialog img')).toHaveCount(0)
    await expect(trigger).toBeFocused()
    expect(originals).toEqual([])
  })
}
