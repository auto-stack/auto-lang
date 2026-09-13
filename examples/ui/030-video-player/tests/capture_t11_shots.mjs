// capture_t11_shots.mjs — T-09..T-11 完成后的双主题/音轨标注截图取证。
// 前置：auto run（vite :3030 + 后端 :8330）已在跑；本地判据是 e2e 套件本身。
import { chromium } from '@playwright/test'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const OUT = resolve(dirname(fileURLToPath(import.meta.url)), 'screenshots')
const base = process.env.VIDEO_PLAYER_URL || 'http://localhost:3030'

const browser = await chromium.launch({ channel: 'chrome', args: ['--autoplay-policy=no-user-gesture-required'] })
const page = await (await browser.newContext({ viewport: { width: 1280, height: 720 } })).newPage()
await page.goto(base)
await page.locator('.queue-item').first().waitFor({ timeout: 20000 })

// 深色（pac.at 默认 dark）
await page.screenshot({ path: resolve(OUT, 'after_t11_vue_dark.png') })

// 选 caelestia 播放（有声轨，不出现标注）
await page.locator('.queue-item', { hasText: 'caelestia' }).first().click()
await page.waitForTimeout(2500)
await page.screenshot({ path: resolve(OUT, 'after_t11_vue_playing.png') })

// 选 4K MKV：播放 >1s → 探针 → 「音轨不受支持」标注（AC-16b 的视觉证据）
// （队列条目显示的是 title —— display_title 会裁掉扩展名，故从 scan 接口取）
const scan = await (await fetch(`${base}/api/media/scan`)).json()
const mkvEntry = (scan.entries ?? []).find((e) => (e.name ?? '').toLowerCase().endsWith('.mkv'))
if (mkvEntry) {
  await page.locator('.queue-item', { hasText: mkvEntry.title }).first().click()
  await page.locator('.audio-note').waitFor({ timeout: 30000 })
  await page.screenshot({ path: resolve(OUT, 'after_t11_vue_mkv_audio_note.png') })
  console.log('[shots] audio-note captured')
} else {
  console.log('[shots] no .mkv entry — skipped audio-note shot')
}

// 浅色（顶栏主题键 = 第一个无文本图标按钮）
const iconBtns = page.locator('button.w-8.h-7')
await iconBtns.first().click()
await page.waitForTimeout(600)
await page.screenshot({ path: resolve(OUT, 'after_t11_vue_light.png') })

await browser.close()
console.log('[shots] done →', OUT)
