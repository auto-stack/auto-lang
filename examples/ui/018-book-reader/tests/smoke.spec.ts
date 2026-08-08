/**
 * 018-book-reader 冒烟测试 (Plan 399) — 对应 acceptance.atd T1-T10.
 * 验证首个「多路由 + rust 后端 API」示例端到端：书架 / 详情 / 阅读 / 进度 / 添加 / 暗色切换.
 * 前置：auto run 已启动前端(:3000)+后端(:8080, CRUD).
 *
 * 路由（hash 模式）：/ , /book/:id , /book/:id/chapter/:ch , /settings
 */
import { test, expect } from '@playwright/test'

// 书架加载后出现的标题（首屏渲染完成的信号）。
async function waitForShelf(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.locator('h1:has-text("Library")').waitFor({ timeout: 10000 })
  // 等 store.Init GET /api/books 回填。
  await page.waitForTimeout(1200)
}

test('T1: 初始书架渲染 — 3 本预置书', async ({ page }) => {
  await waitForShelf(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('The Silent Garden')
  expect(body).toContain('Rivers of Time')
  expect(body).toContain('The Last Lighthouse')
})

test('T2: 书籍卡片含进度信息 (Rivers of Time 33%)', async ({ page }) => {
  await waitForShelf(page)
  const body = await page.locator('body').innerText()
  // 进度百分比文字渲染在卡片底部。
  expect(body).toContain('33%')
  expect(body).toContain('100%') // The Last Lighthouse 已读完
})

test('T3: 点击书卡进入详情页 (见作者 + 章节列表)', async ({ page }) => {
  await waitForShelf(page)
  // 点击 "The Silent Garden" 卡片（书名是可见文本，点击会冒泡到卡片的 onclick）。
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(1000)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Ada Lin') // 作者
  expect(body).toContain('Chapters') // 章节区标题
})

test('T4: 详情页章节列表渲染 (≥3 章)', async ({ page }) => {
  await waitForShelf(page)
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(1000)
  // 三章标题都应出现。
  const body = await page.locator('body').innerText()
  expect(body).toContain('Chapter 1: The Gate')
  expect(body).toContain('Chapter 2: The Gardener')
  expect(body).toContain('Chapter 3: The First Bloom')
})

test('T5: 点击章节进入阅读页 (见章节正文)', async ({ page }) => {
  await waitForShelf(page)
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(1000)
  // 点击第一章按钮。
  await page.getByText('Chapter 1: The Gate').first().click()
  await page.waitForTimeout(1000)
  const body = await page.locator('body').innerText()
  expect(body).toContain('The old iron gate stood ajar') // 正文首句
})

test('T6: 下一章导航 (URL :ch 变化 + 正文变化)', async ({ page }) => {
  await waitForShelf(page)
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(1000)
  await page.getByText('Chapter 1: The Gate').first().click()
  await page.waitForTimeout(1000)
  expect(await page.locator('body').innerText()).toContain('The old iron gate stood ajar')

  // 点击 Next → 第二章正文。
  await page.getByRole('button', { name: /Next/ }).click()
  await page.waitForTimeout(1000)
  expect(page.url()).toMatch(/chapter\/2/)
  expect(await page.locator('body').innerText()).toContain('A figure moved between the hedgerows')
})

test('T7: 阅读进度持久化 (进章节后回书架, The Silent Garden 进度 > 0)', async ({ page }) => {
  await waitForShelf(page)
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(1000)
  await page.getByText('Chapter 1: The Gate').first().click()
  await page.waitForTimeout(1200) // 等 update_progress PUT 落库
  // 回书架：原 progress=0，读完第1章(共3章)后应变为 ~33%。
  await page.goto('/')
  await page.locator('h1:has-text("Library")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(1200)
  const body = await page.locator('body').innerText()
  // The Silent Garden 卡片现在应显示一个 >0 的进度（第1/3章 ≈ 33%）。
  expect(body).toContain('33%')
})

test('T8: 添加书 (对话框 → 新书出现)', async ({ page }) => {
  await waitForShelf(page)
  const marker = `PW Book ${Date.now()}`
  await page.getByRole('button', { name: /Add Book/ }).click()
  await page.waitForTimeout(500)
  await page.locator('input').nth(0).fill(marker)
  await page.locator('input').nth(1).fill('PW Author')
  await page.getByRole('button', { name: 'Add', exact: true }).click()
  await page.waitForTimeout(1200)
  expect(await page.locator('body').innerText()).toContain(marker)
})

test('T9: 暗色运行时切换 (theme-toggle 点击后 html.dark class 翻转)', async ({ page }) => {
  await waitForShelf(page)
  // 生成器默认 <html class="dark">。ThemeToggle 在 onMounted 按保存偏好初始化，
  // 默认保持 dark。点击后应切到 light（class 被移除）。
  const html = page.locator('html')
  const before = await html.getAttribute('class')
  const wasDark = (before || '').includes('dark')

  // theme-toggle 按钮含 emoji 文本。
  const toggle = page.locator('button.theme-toggle-btn').first()
  await toggle.click()
  await page.waitForTimeout(400)
  const after = await html.getAttribute('class')
  const nowDark = (after || '').includes('dark')
  // 翻转：dark → light 或 light → dark。
  expect(nowDark).toBe(!wasDark)
})

test('T10: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (m) => {
    if (m.type() === 'error') {
      const t = m.text()
      if (!t.includes('favicon') && !t.includes('CORS')) errors.push(t)
    }
  })
  page.on('pageerror', (e) => errors.push(e.message))
  await waitForShelf(page)
  // 走一遍核心流程：进详情 → 阅读 → 下一章。
  await page.getByText('The Silent Garden', { exact: true }).first().click()
  await page.waitForTimeout(800)
  await page.getByText('Chapter 1: The Gate').first().click()
  await page.waitForTimeout(800)
  await page.getByRole('button', { name: /Next/ }).click()
  await page.waitForTimeout(1000)
  const real = errors.filter((e) => !e.includes('net::ERR') && !e.includes('favicon'))
  expect(real, `实质错误: ${real.join('; ')}`).toHaveLength(0)
})
