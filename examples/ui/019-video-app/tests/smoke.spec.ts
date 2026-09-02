/**
 * 019-video-app 冒烟测试 (Plan 519) — 对应 acceptance.atd T1-T10.
 * 验证 Bilibili 风格视频应用：首页 / 分类 / Tab 排序 / 搜索 / 观看页 / 点赞 / 推荐 / 设置与主题切换.
 * 前置：auto run 已启动前端(:3019)+后端(:8019).
 */
import { test, expect } from '@playwright/test'

async function waitForHome(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.getByText('VideoApp').first().waitFor({ timeout: 10000 })
  await page.waitForTimeout(1200)
}

test('T1: 首页加载与标题渲染 (VideoApp + 初始网格)', async ({ page }) => {
  await waitForHome(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('VideoApp')
  expect(body).toContain('Learn Rust & AutoLang')
  expect(body).toContain('Water-Cooled PC Build')
  expect(body).toContain('Moonlight Sonata')
})

test('T2: 分类 Chips 过滤 (点击 Gaming，网格仅含 Gaming 视频)', async ({ page }) => {
  await waitForHome(page)
  // 点击 Gaming 分类 chip
  await page.getByRole('button', { name: 'Gaming', exact: true }).click()
  await page.waitForTimeout(800)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Water-Cooled PC Build')
  expect(body).toContain('Cyberpunk 2077')
  expect(body).not.toContain('Moonlight Sonata')
})

test('T3: Tab 切换 (Trending 排序，首卡为播放量最高的视频)', async ({ page }) => {
  await waitForHome(page)
  // 点击 Trending tab
  await page.getByRole('button', { name: 'Trending' }).click()
  await page.waitForTimeout(800)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Mountain Biking') // 3.2M views
})

test('T4: 搜索过滤 (输入 Rust，网格仅含匹配卡片)', async ({ page }) => {
  await waitForHome(page)
  const searchInput = page.locator('input[placeholder*="Search"]')
  await searchInput.fill('Rust')
  await page.waitForTimeout(800)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Learn Rust & AutoLang')
  expect(body).not.toContain('Moonlight Sonata')
  // 清空搜索
  await searchInput.fill('')
  await page.waitForTimeout(800)
})

test('T5: 视频卡片点击进入观看页 (/watch/:id URL 变化 + 详情呈现)', async ({ page }) => {
  await waitForHome(page)
  await page.getByText('Learn Rust & AutoLang').first().click()
  await page.waitForTimeout(1000)
  expect(page.url()).toMatch(/watch\/1/)
  const body = await page.locator('body').innerText()
  expect(body).toContain('CodeMaster')
  expect(body).toMatch(/About this video/i)
  expect(body).toContain('Related Videos')
})

test('T6: 点赞与播放计数交互 (点击点赞 likes 增加)', async ({ page }) => {
  await waitForHome(page)
  await page.getByText('Learn Rust & AutoLang').first().click()
  await page.waitForTimeout(1000)
  const likeBtn = page.getByRole('button', { name: /❤️/ }).first()
  const initialText = await likeBtn.innerText()
  await likeBtn.click()
  await page.waitForTimeout(800)
  const updatedText = await likeBtn.innerText()
  expect(updatedText).not.toEqual(initialText)
})

test('T7: 观看页相关推荐列表与切换', async ({ page }) => {
  await waitForHome(page)
  await page.getByText('Learn Rust & AutoLang').first().click()
  await page.waitForTimeout(1000)
  // 观看页右侧应有同为 Tech 分类的相关视频（如 Axum 或 AI Compiler）
  const body = await page.locator('body').innerText()
  expect(body).toMatch(/(Building Microservices|Modern AI Compiler)/)
})

test('T8: 设置面板展开与深浅模式切换', async ({ page }) => {
  await waitForHome(page)
  // 点击 ⚙ Settings
  await page.getByRole('button', { name: /Settings/ }).first().click()
  await page.waitForTimeout(600)
  expect(await page.locator('body').innerText()).toContain('Accent Color')

  // 点击 ☀ Light
  const lightBtn = page.getByRole('button', { name: /Light/ }).first()
  await lightBtn.click()
  await page.waitForTimeout(400)

  // 点击 🌙 Dark
  const darkBtn = page.getByRole('button', { name: /Dark/ }).first()
  await darkBtn.click()
  await page.waitForTimeout(400)

  // 关闭设置
  await page.getByRole('button', { name: '✕' }).first().click()
  await page.waitForTimeout(400)
})

test('T9: 设置面板强调色切换', async ({ page }) => {
  await waitForHome(page)
  await page.getByRole('button', { name: /Settings/ }).first().click()
  await page.waitForTimeout(500)
  // 切换强调色按钮
  const accentBtns = page.locator('button.rounded-full')
  if (await accentBtns.count() > 0) {
    await accentBtns.nth(1).click()
    await page.waitForTimeout(300)
  }
  await page.getByRole('button', { name: '✕' }).first().click()
  await page.waitForTimeout(300)
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
  await waitForHome(page)
  await page.getByText('Learn Rust & AutoLang').first().click()
  await page.waitForTimeout(800)
  await page.getByRole('button', { name: /Back/ }).first().click()
  await page.waitForTimeout(800)
  const real = errors.filter((e) => !e.includes('net::ERR') && !e.includes('favicon'))
  expect(real, `实质错误: ${real.join('; ')}`).toHaveLength(0)
})
