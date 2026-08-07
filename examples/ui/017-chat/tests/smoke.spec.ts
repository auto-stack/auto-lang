/**
 * 017-chat 冒烟测试（Plan musk-022）— 对应 acceptance.atd T1-T8。
 * 验证首个 SSE 多事件示例端到端：加载 / 发送 / SSE 推送。
 * 前置：auto run 已启动前端(:3000)+后端(:8080，SSE+CRUD)。
 */
import { test, expect } from '@playwright/test'

async function waitForApp(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.locator('h2:has-text("Chat")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(1000) // 等 store.Init GET /api/messages
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
  await page.locator('input').fill(marker)
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
  await page.locator('input').fill('   ')
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(800)
  const after = (await page.locator('body').innerText()).length
  expect(after).toBeLessThanOrEqual(before + 50)
})

test('T5: 发送后输入框清空', async ({ page }) => {
  await waitForApp(page)
  const input = page.locator('input')
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
  await pageB.locator('input').fill(marker)
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
  await pageB.locator('input').fill('hello-typing')

  // A 通过 SSE 收到 Typing 事件，typing_name = "You"（Phase 12 协议: evt.name）
  // 回归守护: 若后端回退到广播整个 input(字段 sender) 或前端读整个对象,
  // typing_name 会变成 [object Object], 断言失败。
  await expect.poll(
    async () => (await pageA.locator('body').innerText()).includes('You is typing'),
    { timeout: 8000, message: '标签页A应通过SSE看到 "You is typing…"' }
  ).toBe(true)
  await ctxA.close()
  await ctxB.close()
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
  await page.locator('input').fill(`pw-console-${Date.now()}`)
  await page.getByRole('button', { name: 'Send' }).click()
  await page.waitForTimeout(1500)
  const real = errors.filter((e) => !e.includes('EventSource') && !e.includes('net::ERR'))
  expect(real, `实质错误: ${real.join('; ')}`).toHaveLength(0)
})
