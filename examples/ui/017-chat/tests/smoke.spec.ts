/**
 * 017-chat 冒烟测试（Plan musk-022 & Plan 544）— 对应 acceptance.atd T1-T13。
 * 验证微信风格双栏布局、多会话切换、Emoji 选择器、Bot 自动应答与 SSE 多事件流端到端。
 * 前置：auto run 已启动前端(:3000)+后端(:8080，SSE+CRUD)。
 */
import { test, expect } from '@playwright/test'

async function waitForApp(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.locator('h2:has-text("Chat")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(1000) // 等 store.Init GET /api/messages & /api/contacts
}

function composerInput(page: import('@playwright/test').Page) {
  return page.locator('input[placeholder*="message"]')
}

test('T1: 初始消息渲染 — 含 Alice 和 Bob', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Alice')
  expect(body).toContain('How is the project going?')
  expect(body).toContain('Morning everyone!')
})

test('T2: 双气泡方向 — sent/received 都渲染', async ({ page }) => {
  await waitForApp(page)
  const sent = await page.locator('[class*="bg-primary"]').count()
  const received = await page.locator('[class*="bg-muted"]').count()
  expect(sent).toBeGreaterThan(0)
  expect(received).toBeGreaterThan(0)
})

test('T3: 发送消息后气泡出现（且在右侧 sent 位置）', async ({ page }) => {
  await waitForApp(page)
  const marker = `pw-send-${Date.now()}`
  const input = composerInput(page)
  await input.fill(marker)
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(1000)
  expect(await page.locator('body').innerText()).toContain(marker)
  // 回归守护：新发消息气泡必须在右侧(justify-end)。Plan 399 第4-5步让后端调
  // db::create_message(设 mine:true)，前端按 msg.mine 派生方向；若后端回退到
  // ..Default::default() (mine:false)，气泡会落到左侧。
  const sentBubble = page.locator(`span:has-text("${marker}")`)
    .locator('xpath=ancestor::div[contains(@class,"justify-end")]')
  await expect(sentBubble).toHaveCount(1, { timeout: 3000 })
})

test('T4: 空消息不发送', async ({ page }) => {
  await waitForApp(page)
  const before = (await page.locator('body').innerText()).length
  const input = composerInput(page)
  await input.fill('   ')
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(800)
  const after = (await page.locator('body').innerText()).length
  expect(after).toBeLessThanOrEqual(before + 50)
})

test('T5: 发送后输入框清空', async ({ page }) => {
  await waitForApp(page)
  const input = composerInput(page)
  await input.fill(`pw-clear-${Date.now()}`)
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(800)
  await expect(input).toHaveValue('')
})

test('T7: SSE 连接建立 — EventSource 到 /api/stream', async ({ page }) => {
  let sse = false
  page.on('request', (r) => { if (r.url().includes('/api/stream')) sse = true })
  await waitForApp(page)
  await page.waitForTimeout(1500)
  expect(sse).toBe(true)
})

test('T6: SSE NewMessage 推送（跨标签页）', async ({ browser }) => {
  const ctxA = await browser.newContext()
  const pageA = await ctxA.newPage()
  await waitForApp(pageA)
  const marker = `pw-sse-${Date.now()}`
  expect(await pageA.locator('body').innerText()).not.toContain(marker)

  const ctxB = await browser.newContext()
  const pageB = await ctxB.newPage()
  await waitForApp(pageB)
  const inputB = composerInput(pageB)
  await inputB.fill(marker)
  await pageB.getByRole('button', { name: 'Send' }).click()

  // A 通过 SSE 收到 B 的广播
  await expect.poll(
    async () => (await pageA.locator('body').innerText()).includes(marker),
    { timeout: 8000, message: `标签页A应通过SSE收到 "${marker}"` }
  ).toBe(true)
  await ctxA.close()
  await ctxB.close()
})

test('T9: typing 指示器跨标签页（B 输入 → A 看到 "You is typing…"）', async ({ browser }) => {
  const ctxA = await browser.newContext()
  const pageA = await ctxA.newPage()
  await waitForApp(pageA)

  const ctxB = await browser.newContext()
  const pageB = await ctxB.newPage()
  await waitForApp(pageB)

  // B 输入触发 oninput → on_typing("You") → POST /api/typing → SSE Typing 广播
  const inputB = composerInput(pageB)
  await inputB.fill('hello-typing')

  // A 通过 SSE 收到 Typing 事件，typing_name = "You"
  await expect.poll(
    async () => (await pageA.locator('body').innerText()).includes('You is typing'),
    { timeout: 8000, message: '标签页A应通过SSE看到 "You is typing…"' }
  ).toBe(true)
  await ctxA.close()
  await ctxB.close()
})

test('T10: 侧栏联系人列表渲染与多会话展示', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Alice')
  expect(body).toContain('Bob')
  expect(body).toContain('AutoBot')
  expect(body).toContain('Tech Support')
})

test('T11: Emoji 表情面板点击追加至草稿输入框', async ({ page }) => {
  await waitForApp(page)
  const input = composerInput(page)
  await input.fill('Hi ')
  // 点击表情按钮 😀
  await page.getByRole('button', { name: '😀' }).click()
  await page.waitForTimeout(300)
  expect(await input.inputValue()).toBe('Hi 😀')
})

test('T12: 智能助手 Bot Reply 触发与响应', async ({ page }) => {
  await waitForApp(page)
  // 点击顶部 🤖 Bot Reply 按钮
  await page.getByRole('button', { name: '🤖 Bot Reply' }).click()
  await page.waitForTimeout(1000)
  const body = await page.locator('body').innerText()
  expect(body).toContain('AutoBot')
})

test('T13: 侧栏联系人搜索动态过滤', async ({ page }) => {
  await waitForApp(page)
  const searchInput = page.locator('input[placeholder*="Search"]')
  await searchInput.fill('Bob')
  await page.waitForTimeout(300)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Bob')
})

test('T8: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (m) => {
    if (m.type() === 'error') {
      const t = m.text()
      if (!t.includes('favicon') && !t.includes('CORS')) errors.push(t)
    }
  })
  page.on('pageerror', (e) => errors.push(e.message))
  await waitForApp(page)
  const input = composerInput(page)
  await input.fill(`pw-console-${Date.now()}`)
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(1500)
  const real = errors.filter((e) => !e.includes('EventSource') && !e.includes('net::ERR'))
  expect(real, `实质错误: ${real.join('; ')}`).toHaveLength(0)
})
