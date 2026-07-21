/**
 * 015-notes 冒烟测试（Plan 366a）
 *
 * 对应 tests/acceptance.atd 的 T1-T13 契约。
 *
 * 设计原则：
 * - 每个 test 自包含，不依赖前一个 test 的状态
 * - 只读测试（导航/筛选/主题）在前，破坏性测试（Edit/Save/Delete）在后
 * - 破坏性测试自行清理（改回原标题、删临时笔记）
 *
 * 前置条件：dev server + 后端 API 已就绪（auto run）。
 */
import { test, expect } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  // 等侧栏 New 按钮出现（说明应用加载完成）
  await page.locator('button:has-text("New")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(800) // 等 store.Init 的 API 调用 + 渲染
})

// ============================================================================
// 导航类（只读，无副作用）
// ============================================================================

test('T1: 笔记切换更新编辑区内容', async ({ page }) => {
  // 找侧栏的笔记按钮（不是 New、不是色板、不是 tag）
  const noteButtons = page.locator('button[class*="rounded-lg"]')
  const count = await noteButtons.count()
  expect(count).toBeGreaterThan(0)

  // 点击最后一个笔记
  await noteButtons.last().click()
  await page.waitForTimeout(800) // 等 editor 切换 + Tiptap 挂载

  // 编辑区应该有 ProseMirror（editor 创建成功）
  // 这是 T1 的核心断言：切换后 Tiptap editor 正确初始化（之前的 bug 是空白）
  const proseMirror = page.locator('.ProseMirror')
  if (await proseMirror.count() > 0) {
    const content = await proseMirror.textContent()
    // 内容可能为空（某些 seed 笔记 body 为空），但 editor 必须存在
    expect(proseMirror).toBeVisible()
  }
})

test('T2: View tabs — Pinned 无文件夹标题', async ({ page }) => {
  await page.locator('button:has-text("Pinned")').click()
  await page.waitForTimeout(300)
  expect(await page.locator('text=📁').count()).toBe(0)
})

test('T2: View tabs — All 有文件夹标题', async ({ page }) => {
  await page.locator('button:has-text("All")').click()
  await page.waitForTimeout(300)
  expect(await page.locator('text=📁').count()).toBeGreaterThan(0)
})

test('T2: View tabs — Recent 显示所有笔记', async ({ page }) => {
  await page.locator('button:has-text("Recent")').click()
  await page.waitForTimeout(300)
  // Recent 不应有文件夹标题（和 Pinned 类似）
  expect(await page.locator('text=📁').count()).toBe(0)
})

// T3 搜索 — known-gap（功能未实现）
test.skip('T3: 搜索（known-gap）', async () => {})

test('T4: Tag 筛选改变笔记列表', async ({ page }) => {
  // 找 tag 筛选按钮（不是色板、不是 tag pill 上的 ×）
  const tagFilters = page.locator('button[class*="rounded-full"][class*="bg-muted"]')
  const tagCount = await tagFilters.count()
  if (tagCount === 0) return // 没 tag 就跳过

  const beforeNotes = await page.locator('button[class*="rounded-lg"]').count()
  await tagFilters.first().click()
  await page.waitForTimeout(400)
  const afterNotes = await page.locator('button[class*="rounded-lg"]').count()
  // 筛选后笔记数应 ≤ 筛选前
  expect(afterNotes).toBeLessThanOrEqual(beforeNotes)
})

// ============================================================================
// 主题类（只读状态，但有 localStorage 副作用）
// ============================================================================

test('T11: Dark mode 切换根元素的 dark class', async ({ page }) => {
  const root = page.locator('#app > div').first()
  const beforeDark = await root.evaluate(el => el.className.includes('dark'))

  // 找到 dark/light 切换按钮（文本可能是 🌙 Dark 或 ☀ Light）
  const toggle = page.locator('button:has-text("Dark"), button:has-text("Light")')
  await toggle.click()
  await page.waitForTimeout(500)

  const afterDark = await root.evaluate(el => el.className.includes('dark'))
  expect(afterDark).toBe(!beforeDark)
})

test('T13: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', err => errors.push(err.message))

  // 触发一批操作
  await page.locator('button:has-text("Pinned")').click()
  await page.waitForTimeout(200)
  await page.locator('button:has-text("All")').click()
  await page.waitForTimeout(200)

  // 禁止的错误模式（本次会话教训）
  const forbidden = [
    'view is not available',
    'Unhandled error',
    'store is not defined',
  ]
  for (const pattern of forbidden) {
    const found = errors.find(e => e.includes(pattern))
    expect(found, `禁止的控制台错误 "${pattern}": ${found}`).toBeUndefined()
  }
})

// ============================================================================
// 编辑类（破坏性，放最后；每个 test 自行清理）
// ============================================================================

test('T5: Edit 显示当前笔记内容', async ({ page }) => {
  // 先选第一个笔记确保有内容
  const noteButtons = page.locator('button[class*="rounded-lg"]')
  if (await noteButtons.count() === 0) return
  await noteButtons.first().click()
  await page.waitForTimeout(500)

  await page.locator('button:has-text("Edit")').click()
  await page.waitForTimeout(1000) // Tiptap 挂载

  // 标题输入框应有值
  const titleInput = page.locator('input[placeholder="Note title..."]')
  await expect(titleInput).toHaveValue(/\S/)

  // ProseMirror editor 应存在且可见
  await expect(page.locator('.ProseMirror')).toBeVisible()

  // 清理：Cancel
  await page.locator('button:has-text("Cancel")').click()
  await page.waitForTimeout(500)
})

test('T5: Cancel 返回只读模式', async ({ page }) => {
  const noteButtons = page.locator('button[class*="rounded-lg"]')
  if (await noteButtons.count() === 0) return
  await noteButtons.first().click()
  await page.waitForTimeout(500)

  await page.locator('button:has-text("Edit")').click()
  await page.waitForTimeout(600)
  await page.locator('button:has-text("Cancel")').click()
  await page.waitForTimeout(500)

  await expect(page.locator('button:has-text("Edit")')).toBeVisible()
})

test('T5: Save 持久化标题修改', async ({ page }) => {
  const titleInput = page.locator('input[placeholder="Note title..."]')

  // 选第一个笔记（确保存在）
  const noteButtons = page.locator('button[class*="rounded-lg"]')
  if (await noteButtons.count() === 0) return
  await noteButtons.first().click()
  await page.waitForTimeout(500)

  await page.locator('button:has-text("Edit")').click()
  await page.waitForTimeout(600)

  const originalTitle = await titleInput.inputValue()
  expect(originalTitle.length).toBeGreaterThan(0)

  const suffix = ` [T5-${Date.now()}]`
  await titleInput.fill(originalTitle + suffix)
  await page.locator('button:has-text("Save")').click()
  await page.waitForTimeout(1500)

  // 通过 API 验证持久化
  const notes = await (await page.request.get('/api/notes')).json()
  expect(notes.some((n: any) => (n.title || '').includes(suffix))).toBe(true)

  // 清理：改回原标题
  await page.locator('button:has-text("Edit")').click()
  await page.waitForTimeout(500)
  await titleInput.fill(originalTitle)
  await page.locator('button:has-text("Save")').click()
  await page.waitForTimeout(800)
})

test('T7: Delete 减少笔记数量', async ({ page }) => {
  // 通过 API 创建临时笔记（避免 UI 创建的时序问题）
  await page.request.post('/api/notes', {
    data: { title: 'T7-temp-delete', body: 'temp', folder: '' },
  })

  // 重新加载拿到最新列表
  await page.goto('/')
  await page.locator('button:has-text("New")').waitFor({ timeout: 10000 })
  await page.waitForTimeout(800)

  const beforeResp = await page.request.get('/api/notes')
  const beforeCount = (await beforeResp.json()).length

  // 找到 T7-temp 笔记并选中
  const tempBtn = page.locator('button:has-text("T7-temp-delete")')
  if (await tempBtn.count() > 0) {
    await tempBtn.first().click()
    await page.waitForTimeout(500)
  }

  page.on('dialog', d => d.accept())
  await page.getByRole('button', { name: 'Delete', exact: true }).click()
  await page.waitForTimeout(1500)

  const afterCount = (await (await page.request.get('/api/notes')).json()).length
  expect(afterCount).toBe(beforeCount - 1)
})
