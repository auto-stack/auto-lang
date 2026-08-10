/**
 * 023-realworld 冒烟测试 (Plan 405 阶段1) — 对应 acceptance.atd T1-T8.
 * 前端 vite(auto run 启动, 端口见 RW_URL) + 后端 8023.
 *
 * 端口缺口(Plan 401): auto run 不读 pac.at front_port, vite 端口会变。
 * 测试前先 auto run, 看 "Local:" 行的端口, 设 RW_URL=http://localhost:<port>
 * 再跑 npm test。
 */
import { test, expect } from '@playwright/test'

// 等首页加载: store.Init 异步拉 articles。
async function waitForHome(page) {
  await page.goto('/#/')
  await page.locator('text=Global Feed').waitFor({ timeout: 10000 })
  await page.waitForTimeout(1500)
}

test('T1: 首页 feed 渲染 — 3 篇种子文章', async ({ page }) => {
  await waitForHome(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Understanding React Server Components')
  expect(body).toContain('Building Type-Safe APIs with tRPC')
  expect(body).toContain('The State of CSS in 2026')
})

test('T2: 标签过滤 — 点 React 后只显示含 React 的文章', async ({ page }) => {
  await waitForHome(page)
  // 点 Popular Tags 里的 React
  await page.locator('text=React').first().click()
  await page.waitForTimeout(800)
  const body = await page.locator('body').innerText()
  // React 文章仍在
  expect(body).toContain('Understanding React Server Components')
  // 不含 React 的文章消失 (tRPC 文章 tagList 是 TypeScript,tRPC,API)
  expect(body).not.toContain('Building Type-Safe APIs with tRPC')
})

test('T3: 进文章详情 — 正文 + 评论', async ({ page }) => {
  await waitForHome(page)
  // 点第一篇文章标题
  await page.locator('text=Understanding React Server Components').first().click()
  await page.waitForTimeout(1200)
  const body = await page.locator('body').innerText()
  // 正文(含"faster initial loads")
  expect(body).toContain('faster initial loads')
  // 评论
  expect(body).toContain('Comments')
})

test('T4: 注册 — 成功后 nav 显示用户名', async ({ page }) => {
  await page.goto('/#/register')
  await page.waitForLoadState('domcontentloaded')
  const name = 'TestUser' + Date.now()
  await page.locator('input[placeholder="Username"]').fill(name)
  await page.locator('input[placeholder="Email"]').fill(name + '@test.com')
  await page.locator('input[placeholder="Password"]').fill('password123')
  await page.locator('button:has-text("Sign up")').click()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain(name)
})

test('T5: 登录 — 已知种子用户成功', async ({ page }) => {
  await page.goto('/#/login')
  await page.waitForLoadState('domcontentloaded')
  // 默认值已是 sarah@vercel.com / password
  await page.locator('button:has-text("Sign in")').click()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Sarah Chen')
})

test('T6: 设置页 — 登录后显示资料 + 登出', async ({ page }) => {
  // 先登录
  await page.goto('/#/login')
  await page.waitForLoadState('domcontentloaded')
  await page.locator('button:has-text("Sign in")').click()
  await page.waitForTimeout(1500)
  // 进设置
  await page.goto('/#/settings')
  await page.waitForTimeout(1000)
  let body = await page.locator('body').innerText()
  expect(body).toContain('Your Settings')
  expect(body).toContain('Sarah Chen')
  // 登出
  await page.locator('button:has-text("logout")').click()
  await page.waitForTimeout(1000)
  body = await page.locator('body').innerText()
  // 登出后 nav 显示 Sign in
  expect(body).toContain('Sign in')
})

test('T7: 未登录访问设置页 — 显示 Sign in 提示', async ({ page }) => {
  await page.goto('/#/settings')
  await page.waitForTimeout(1000)
  const body = await page.locator('body').innerText()
  expect(body).toContain('not logged in')
})

test('T8: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })
  await waitForHome(page)
  await page.waitForTimeout(1000)
  const real = errors.filter((e) => !e.includes('favicon') && !e.includes('Failed to load resource'))
  expect(real, `console errors: ${real.join('; ')}`).toEqual([])
})

// --- Stage 2 tests (T9-T14) ---

test('T9: 新建文章 — 编辑器填表 → 发布 → 出现在 feed', async ({ page }) => {
  await page.goto('/#/editor/new')
  await page.waitForTimeout(800)
  const title = 'Stage2 Test Article ' + Date.now()
  const slug = 'stage2-test-' + Date.now()
  await page.locator('input[placeholder="Article Title"]').fill(title)
  await page.locator('input[placeholder="url-slug (lowercase, dashes)"]').fill(slug)
  await page.locator('input[placeholder="What\'s this article about?"]').fill('A test description')
  await page.locator('textarea[placeholder*="Write your article"]').fill('The body of the test article.')
  await page.locator('input[placeholder*="Tags"]').fill('Test,Stage2')
  await page.locator('button:has-text("Publish Article")').click()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain(title)
})

test('T10: 编辑文章 — 改 title → 保存 → 详情更新', async ({ page }) => {
  await page.goto('/#/editor/new')
  await page.waitForTimeout(800)
  const slug = 'edit-target-' + Date.now()
  await page.locator('input[placeholder="Article Title"]').fill('Original Title')
  await page.locator('input[placeholder="url-slug (lowercase, dashes)"]').fill(slug)
  await page.locator('input[placeholder="What\'s this article about?"]').fill('desc')
  await page.locator('textarea[placeholder*="Write your article"]').fill('body')
  await page.locator('button:has-text("Publish Article")').click()
  await page.waitForTimeout(1500)
  await page.goto('/#/editor/' + slug)
  await page.waitForTimeout(1500)
  await page.locator('input[placeholder="Article Title"]').fill('Edited Title')
  await page.locator('button:has-text("Publish Article")').click()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Edited Title')
})

test('T11: 删除文章 — feed 列表正常(删按钮 stage3)', async ({ page }) => {
  await page.goto('/')
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Global Feed')
})

test('T12: 发评论 — 文章详情提交评论 → 出现', async ({ page }) => {
  await page.goto('/#/article/understanding-react-server-components')
  await page.waitForTimeout(1500)
  const commentText = 'A stage2 test comment ' + Date.now()
  await page.locator('textarea[placeholder="Write a comment..."]').fill(commentText)
  await page.locator('button:has-text("Post Comment")').click()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain(commentText)
})

test('T13: 收藏文章 — ♥ 数增加', async ({ page }) => {
  await page.goto('/#/article/building-type-safe-apis-with-trpc')
  await page.waitForTimeout(1500)
  const beforeBody = await page.locator('body').innerText()
  const beforeMatch = beforeBody.match(/♥\s*(\d+)/)
  const before = beforeMatch ? parseInt(beforeMatch[1]) : 0
  await page.locator('button:has-text("♥")').click()
  await page.waitForTimeout(1500)
  const afterBody = await page.locator('body').innerText()
  const afterMatch = afterBody.match(/♥\s*(\d+)/)
  const after = afterMatch ? parseInt(afterMatch[1]) : 0
  expect(after).toBe(before + 1)
})

test('T14: 资料页 — 显示用户名 + My Articles', async ({ page }) => {
  await page.goto('/#/profile/Sarah%20Chen')
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Sarah Chen')
  expect(body).toContain('My Articles')
})
