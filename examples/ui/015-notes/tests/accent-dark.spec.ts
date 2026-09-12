/**
 * T12 + T12-DARK: 主题色切换 + 暗色模式联动（PLAN-616 重基线）
 *
 * 本文件是 dark-mode accent bug 的回归测试。
 * 历史教训（Plan 360）：applyAccent 把 --primary 写到 <html>，但 .dark class
 * 在 #app > div 上，导致暗色模式下 CSS 变量子元素覆盖，accent 失效。
 * 契约依据：C-DARK-1（acceptance.atd）。
 *
 * PLAN-616 重基线（旧值已随 PLAN-503/601 漂移）：
 * - coral 校准值 `4 43% 59%`（Plan 503，旧常量还是 Plan 503 之前的 #e85d75）
 * - 暗色亮度补偿 **+10**（Plan 601 统一值，旧常量是 +4）
 * - ocean 色板 class 是 `sky-500`（生成器用的是 sky 而非 blue）
 * - 色板移入侧栏「Settings」外观面板 → 点击前需展开面板
 * - 主按钮文案 `New` → `New note`
 *
 * 前置条件：dev server + 后端 API 已就绪（auto run）。
 */
import { test, expect, type Page } from '@playwright/test'

// 5 色对应的预期 RGB。light 为 ACCENT_PALETTES 原值；
// dark = lightness +10（applyAccent 的暗色补偿，上限 85%）。
const ACCENT_COLORS = {
  indigo: { light: [100, 103, 242], dark: [147, 149, 246] },
  coral: { light: [195, 111, 105], dark: [210, 146, 142] },
  ocean: { light: [60, 131, 246], dark: [109, 162, 248] },
  sage: { light: [16, 183, 127], dark: [20, 230, 160] },
  amber: { light: [245, 159, 10], dark: [247, 178, 59] },
} as const

// 色板按钮的 CSS class 后缀（Tailwind 颜色，与 sidebar.at 的 swatch 一致）
const SWATCH_CLASS: Record<string, string> = {
  indigo: 'indigo-500',
  coral: 'rose-500',
  ocean: 'sky-500',
  sage: 'emerald-500',
  amber: 'amber-500',
}

const swatch = (page: Page, accent: string) =>
  page.locator(`button[class*="rounded-full"][class*="${SWATCH_CLASS[accent]}"]`).first()

/** 展开外观面板（幂等：面板已开时 swatch 可见，直接返回）。 */
async function openSettings(page: Page) {
  if (!(await swatch(page, 'indigo').isVisible().catch(() => false))) {
    await page.getByRole('button', { name: 'Settings' }).click()
    await page.waitForTimeout(400)
  }
}

async function clickSwatch(page: Page, accent: string) {
  await openSettings(page)
  await swatch(page, accent).click()
  await page.waitForTimeout(600) // 等 applyAccent + Vue 刷新
}

async function getNewButtonBg(page: Page): Promise<[number, number, number]> {
  const bg = await page
    .getByRole('button', { name: 'New note' })
    .evaluate(el => getComputedStyle(el).backgroundColor)
  const m = bg.match(/\d+/g)!
  return [Number(m[0]), Number(m[1]), Number(m[2])]
}

// 颜色近似匹配（允许 ±15 的偏差：HSL→RGB 转换与浏览器渲染有微小差异）
function expectColorClose(
  actual: [number, number, number],
  expected: readonly [number, number, number],
  accent: string,
  mode: string,
) {
  const tolerance = 15
  for (let i = 0; i < 3; i++) {
    const diff = Math.abs(actual[i] - expected[i])
    expect(
      diff,
      `${accent} in ${mode} mode: channel ${['R', 'G', 'B'][i]} = ${actual[i]}, expected ~${expected[i]} (diff ${diff} > ${tolerance})`,
    ).toBeLessThanOrEqual(tolerance)
  }
}

function isNearWhite(c: [number, number, number]) {
  return c[0] > 240 && c[1] > 240 && c[2] > 240
}

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'New note' }).waitFor({ timeout: 10000 })
  await page.waitForTimeout(600)
  // 归位到亮色，避免上一轮的 dark 残留影响 light 断言
  await openSettings(page)
  await page.getByRole('button', { name: 'Light' }).click()
  await page.waitForTimeout(400)
})

// ============================================================================
// T12: 亮色模式 5 色主题切换
// ============================================================================

test('T12-LIGHT: 5 色板依次驱动 --primary', async ({ page }) => {
  for (const accent of Object.keys(ACCENT_COLORS) as (keyof typeof ACCENT_COLORS)[]) {
    await clickSwatch(page, accent)
    const bg = await getNewButtonBg(page)
    expectColorClose(bg, ACCENT_COLORS[accent].light, accent, 'light')
  }
})

test('T12-LIGHT: 选中色写入 localStorage', async ({ page }) => {
  await clickSwatch(page, 'coral')
  const saved = await page.evaluate(() => localStorage.getItem('notes-accent-color'))
  expect(saved).toBe('coral')
})

// ============================================================================
// T12-DARK: 暗色模式 5 色切换（关键回归：暗色下 accent 不能被近白默认吞掉）
// ============================================================================

test('T12-DARK: 暗色模式下 5 色板仍然生效', async ({ page }) => {
  await page.getByRole('button', { name: 'Dark' }).click()
  await page.waitForTimeout(600)

  for (const accent of Object.keys(ACCENT_COLORS) as (keyof typeof ACCENT_COLORS)[]) {
    await clickSwatch(page, accent)
    const bg = await getNewButtonBg(page)
    // ① 不能是暗色默认的近白
    expect(isNearWhite(bg), `${accent} dark: bg ${bg} 看起来是近白默认值`).toBe(false)
    // ② 应当匹配 +10 亮度补偿后的颜色
    expectColorClose(bg, ACCENT_COLORS[accent].dark, accent, 'dark')
  }
})

test('T12-ROUNDTRIP: coral 在 light → dark → light 间无残留', async ({ page }) => {
  await clickSwatch(page, 'coral')
  expectColorClose(await getNewButtonBg(page), ACCENT_COLORS.coral.light, 'coral', 'light')

  await page.getByRole('button', { name: 'Dark' }).click()
  await page.waitForTimeout(600)
  expectColorClose(await getNewButtonBg(page), ACCENT_COLORS.coral.dark, 'coral', 'dark')

  await page.getByRole('button', { name: 'Light' }).click()
  await page.waitForTimeout(600)
  expectColorClose(
    await getNewButtonBg(page),
    ACCENT_COLORS.coral.light,
    'coral',
    'light-after-dark',
  )
})

// ============================================================================
// 契约验证：C-DARK-1（.dark 元素与 --primary inline 的一致性）
// ============================================================================

test('C-DARK-1: 暗色模式下 <html> 和 .dark 元素的 --primary 一致', async ({ page }) => {
  await clickSwatch(page, 'ocean')
  await page.getByRole('button', { name: 'Dark' }).click()
  await page.waitForTimeout(1000) // 等 setTimeout(0) 的 applyToDark 兜底执行

  const htmlPrimary = await page.evaluate(() =>
    document.documentElement.style.getPropertyValue('--primary'),
  )
  const darkElPrimary = await page
    .locator('#app > div')
    .evaluate(el => getComputedStyle(el).getPropertyValue('--primary'))

  expect(
    htmlPrimary.trim(),
    'C-DARK-1 违反: <html> 和 .dark 元素的 --primary 不一致',
  ).toBe(darkElPrimary.trim())

  // 且应该是 ocean（217 …），不是暗色默认（… 98%）
  expect(darkElPrimary).toContain('217')
  expect(darkElPrimary).not.toContain('98%')
})
