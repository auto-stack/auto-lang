/**
 * T12 + T12-DARK: 主题色切换 + 暗色模式联动（Plan 366a）
 *
 * 本文件是 dark-mode accent bug 的回归测试。
 * 历史教训（Plan 360）：applyAccent 把 --primary 写到 <html>，但 .dark class
 * 在 #app > div 上，导致暗色模式下 CSS 变量子元素覆盖，accent 失效。
 * 契约依据：C-DARK-1（acceptance.atd）
 */
import { test, expect } from '@playwright/test'

// 5 色对应的预期 RGB（light 模式），dark 模式 lightness +4%
const ACCENT_COLORS = {
  indigo: { light: [100, 103, 242], dark: [119, 121, 243] },
  coral: { light: [232, 94, 117], dark: [235, 112, 133] },
  ocean: { light: [60, 131, 246], dark: [80, 144, 247] },
  sage: { light: [16, 183, 127], dark: [18, 202, 140] },
  amber: { light: [245, 159, 10], dark: [246, 166, 30] },
} as const

// 色板按钮的 CSS class 后缀（Tailwind 颜色）
const SWATCH_CLASS: Record<string, string> = {
  indigo: 'indigo-500',
  coral: 'rose-500',
  ocean: 'blue-500',
  sage: 'emerald-500',
  amber: 'amber-500',
}

async function clickSwatch(page: import('@playwright/test').Page, accent: string) {
  const cls = SWATCH_CLASS[accent]
  await page.locator(`button[class*="rounded-full"][class*="${cls}"]`).click()
  await page.waitForTimeout(500) // 等 applyAccent + Vue 刷新
}

async function getNewButtonBg(page: import('@playwright/test').Page): Promise<[number, number, number]> {
  const bg = await page.locator('button:has-text("New")')
    .evaluate(el => getComputedStyle(el).backgroundColor)
  const m = bg.match(/\d+/g)!
  return [Number(m[0]), Number(m[1]), Number(m[2])]
}

// 颜色近似匹配（允许 ±15 的偏差，因为 HSL→RGB 转换和浏览器渲染可能有微小差异）
function expectColorClose(actual: [number, number, number], expected: [number, number, number], accent: string, mode: string) {
  const tolerance = 15
  for (let i = 0; i < 3; i++) {
    const diff = Math.abs(actual[i] - expected[i])
    expect(diff, `${accent} in ${mode} mode: channel ${['R', 'G', 'B'][i]} = ${actual[i]}, expected ~${expected[i]} (diff ${diff} > ${tolerance})`).toBeLessThanOrEqual(tolerance)
  }
}

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await page.locator('button:has-text("New")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(500)
})

// ============================================================================
// T12: 亮色模式 5 色主题切换
// ============================================================================

test('T12-LIGHT: 5 色在亮色模式下全部生效', async ({ page }) => {
  // 确保在 light 模式
  const isDark = await page.locator('#app > div').evaluate(el => el.className.includes('dark'))
  if (isDark) {
    await page.locator('button:has-text("Light")').click()
    await page.waitForTimeout(500)
  }

  for (const accent of Object.keys(ACCENT_COLORS)) {
    await clickSwatch(page, accent)
    const bg = await getNewButtonBg(page)
    const expected = ACCENT_COLORS[accent as keyof typeof ACCENT_COLORS].light
    expectColorClose(bg, expected, accent, 'light')
  }
})

test('T12-LIGHT: localStorage 持久化当前选择', async ({ page }) => {
  await clickSwatch(page, 'coral')
  const stored = await page.evaluate(() => localStorage.getItem('notes-accent-color'))
  expect(stored).toBe('coral')
})

// ============================================================================
// T12-DARK: 暗色模式 5 色主题切换（⚠️ 关键回归测试）
// ============================================================================

test('T12-DARK: 暗色模式下 5 色全部生效（非默认近白色）', async ({ page }) => {
  // 切到 dark 模式
  await page.locator('button:has-text("Dark")').click()
  await page.waitForTimeout(800) // 等 dark class 应用 + body 过渡

  for (const accent of Object.keys(ACCENT_COLORS)) {
    await clickSwatch(page, accent)
    const bg = await getNewButtonBg(page)

    // 关键断言：不能是暗色模式默认的近白色 rgb(248, 250, 252)
    // 这是本次 bug 的症状——accent 没覆盖 .dark 的 --primary
    const isNearWhite = bg[0] > 240 && bg[1] > 240 && bg[2] > 240
    expect(isNearWhite, `${accent} 在暗色下显示为近白色（bug 回归！bg=${bg}）`).toBe(false)

    const expected = ACCENT_COLORS[accent as keyof typeof ACCENT_COLORS].dark
    expectColorClose(bg, expected, accent, 'dark')
  }
})

test('T12-ROUNDTRIP: light → dark → light 切换无残留', async ({ page }) => {
  // 选 coral
  await clickSwatch(page, 'coral')

  // light → dark
  await page.locator('button:has-text("Dark")').click()
  await page.waitForTimeout(800)
  let bg = await getNewButtonBg(page)
  expectColorClose(bg, ACCENT_COLORS.coral.dark, 'coral', 'dark')

  // dark → light
  await page.locator('button:has-text("Light")').click()
  await page.waitForTimeout(800)
  bg = await getNewButtonBg(page)
  // 回到 light 后应该是 coral 的 light 值，不是残留的 dark 值
  expectColorClose(bg, ACCENT_COLORS.coral.light, 'coral', 'light-after-dark')
})

// ============================================================================
// 契约验证：C-DARK-1（.dark 元素与 --primary inline 的一致性）
// ============================================================================

test('C-DARK-1: 暗色模式下 <html> 和 .dark 元素的 --primary 一致', async ({ page }) => {
  await clickSwatch(page, 'ocean')
  await page.locator('button:has-text("Dark")').click()
  await page.waitForTimeout(1000) // 等 setTimeout(0) 的 applyToDark 兜底执行

  // <html> 上的 inline --primary
  const htmlPrimary = await page.evaluate(() =>
    document.documentElement.style.getPropertyValue('--primary')
  )

  // .dark 元素上的 computed --primary（实际驱动 bg-primary 的那个）
  const darkElPrimary = await page.locator('#app > div').evaluate(el =>
    getComputedStyle(el).getPropertyValue('--primary')
  )

  // 两者应该一致（applyAccent 设到 html + applyToDark 覆盖到 .dark 元素）
  expect(htmlPrimary.trim(), 'C-DARK-1 违反: <html> 和 .dark 元素的 --primary 不一致').toBe(darkElPrimary.trim())

  // 且应该是 ocean（217 91% 64%），不是暗色默认（210 40% 98%）
  expect(darkElPrimary).toContain('217')
  expect(darkElPrimary).not.toContain('98%') // 98% 是暗色默认的 lightness
})
