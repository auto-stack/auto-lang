// Plan 562 增补:gallery 侧栏 ScrollArea 悬停+滚轮证据采集
// 用法: node scroll_probe.mjs <url> <out-prefix>
import { pathToFileURL } from 'url'

const candidates = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
  'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
]
let chromium
for (const p of candidates) {
  try { chromium = (await import(pathToFileURL(p).href)).chromium; break } catch (_) {}
}
if (!chromium) throw new Error('playwright not found')

const [url, prefix] = process.argv.slice(2)
let browser
try { browser = await chromium.launch({ headless: true, channel: 'msedge' }) }
catch { browser = await chromium.launch({ headless: true, channel: 'chrome' }) }
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })
await page.goto(url, { waitUntil: 'networkidle' })
// 侧栏 ScrollArea viewport
const sidebar = page.locator('aside [data-reka-scroll-area-viewport]').first()
await sidebar.hover()
await page.waitForTimeout(600)
await page.screenshot({ path: `${prefix}_hover.png` })
await page.mouse.wheel(0, 800)
await page.waitForTimeout(600)
await page.screenshot({ path: `${prefix}_scrolled.png` })
await browser.close()
console.log('OK')
