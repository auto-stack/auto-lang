/**
 * 015-notes 冒烟测试（PLAN-616 重写）
 *
 * 对应 tests/acceptance.atd 的 T1-T13 契约（清爽化重做后的语义）。
 *
 * 与旧版的关键差异：
 * - 笔记行有稳定钩子类 `note-row`（sidebar.at 的 NoteRow），不再用
 *   `button[class*="rounded-lg"]` 这种会误命中「设置/编辑/删除」的代理选择器
 * - 正文是原生 `textarea`（无 Tiptap / 无 `.ProseMirror`）
 * - 编辑是「始终可编辑 + 显式保存」：直接输入 → 出现 Save → 点击落库
 * - 筛选是 All/Pinned/文件夹/标签 胶囊（无 All/Pinned/Recent 分段控件、无 📁 emoji）
 * - 搜索真正生效（旧契约是 known-gap / test.skip）
 *
 * 前置条件：dev server + 后端 API 已就绪（auto run）。
 */
import { test, expect, type Page } from '@playwright/test'

const TITLE = 'input[placeholder="Untitled"]'
const BODY = 'textarea[placeholder="Start writing..."]'
const SEARCH = 'input[placeholder="Search notes"]'

const rows = (page: Page) => page.locator('button.note-row')
const saveBtn = (page: Page) => page.getByRole('button', { name: 'Save' })
// exact 必需：临时笔记标题里含 "delete" 会撞上模糊名字匹配（strict mode）
const deleteBtn = (page: Page) => page.getByRole('button', { name: 'Delete', exact: true })

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'New note' }).waitFor({ timeout: 10000 })
  await page.waitForTimeout(800) // 等 store.Init 的 API 调用 + 渲染
})

// ============================================================================
// 导航类（只读）
// ============================================================================

test('T1: 点击列表切换笔记', async ({ page }) => {
  await rows(page).filter({ hasText: 'Shopping List' }).first().click()
  await page.waitForTimeout(600)
  await expect(page.locator(TITLE)).toHaveValue('Shopping List')
  await expect(page.locator(BODY)).toHaveValue(/Milk/)
})

test('T2a: Pinned 筛选只剩置顶笔记', async ({ page }) => {
  await page.getByRole('button', { name: 'Pinned' }).click()
  await page.waitForTimeout(500)
  await expect(rows(page)).toHaveCount(1)
  await expect(rows(page).filter({ hasText: 'Welcome' })).toHaveCount(1)
})

test('T2b: All 恢复全量（置顶与普通都在列表里）', async ({ page }) => {
  await page.getByRole('button', { name: 'All' }).click()
  await page.waitForTimeout(500)
  await expect(rows(page)).toHaveCount(6) // 6 条种子笔记
  await expect(rows(page).filter({ hasText: 'Welcome' })).toHaveCount(1)
  await expect(rows(page).filter({ hasText: 'Quick Ideas' })).toHaveCount(1)
})

test('T2c: 文件夹筛选（personal）', async ({ page }) => {
  await page.getByRole('button', { name: 'personal' }).click()
  await page.waitForTimeout(500)
  await expect(rows(page)).toHaveCount(2)
  await expect(rows(page).filter({ hasText: 'Shopping List' })).toHaveCount(1)
  await expect(rows(page).filter({ hasText: 'Quick Ideas' })).toHaveCount(0)
  await page.getByRole('button', { name: 'All' }).click()
})

// ============================================================================
// 筛选类：搜索（旧契约的 known-gap，现在必须真过滤）
// ============================================================================

test('T3: 搜索命中且大小写不敏感', async ({ page }) => {
  await page.locator(SEARCH).fill('milk')
  await page.waitForTimeout(700)
  await expect(rows(page)).toHaveCount(1)
  await expect(rows(page).filter({ hasText: 'Shopping List' })).toHaveCount(1)

  await page.locator(SEARCH).fill('')
  await page.waitForTimeout(700)
  await expect(rows(page)).toHaveCount(6)
})

test('T3b: 搜索无命中显示空态', async ({ page }) => {
  await page.locator(SEARCH).fill('zzzzz')
  await page.waitForTimeout(700)
  await expect(rows(page)).toHaveCount(0)
  await expect(page.getByText('No matching notes')).toBeVisible()
  await page.locator(SEARCH).fill('')
  await page.waitForTimeout(500)
})

// T4 标签筛选（旧契约里标签行恒空、断言形同虚设）
test('T4: 标签筛选生效', async ({ page }) => {
  await page.locator('button').filter({ hasText: /#\s*home/ }).first().click()
  await page.waitForTimeout(600)
  await expect(rows(page)).toHaveCount(2)
  await expect(rows(page).filter({ hasText: 'Shopping List' })).toHaveCount(1)
  await page.getByRole('button', { name: 'All' }).click()
  await page.waitForTimeout(400)
  await expect(rows(page)).toHaveCount(6)
})

// ============================================================================
// 主题类
// ============================================================================

test('T11: 主题切换翻转根元素 dark class', async ({ page }) => {
  const root = page.locator('#app > div').first()
  const before = await root.evaluate(el => el.className.includes('dark'))

  await page.getByRole('button', { name: 'Theme' }).click()
  await page.waitForTimeout(600)
  const after = await root.evaluate(el => el.className.includes('dark'))
  expect(after).toBe(!before)

  await page.getByRole('button', { name: 'Theme' }).click() // 复原
  await page.waitForTimeout(400)
})

test('T13: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', err => errors.push(err.message))

  await page.getByRole('button', { name: 'Pinned' }).click()
  await page.waitForTimeout(200)
  await page.getByRole('button', { name: 'All' }).click()
  await page.waitForTimeout(200)
  await page.getByRole('button', { name: 'Settings' }).click()
  await page.waitForTimeout(200)
  await page.getByRole('button', { name: 'Light' }).click()
  await page.waitForTimeout(300)
  await page.getByRole('button', { name: 'Dark' }).click()
  await page.waitForTimeout(300)
  await page.getByRole('button', { name: 'Settings' }).click()

  const forbidden = ['view is not available', 'Unhandled error', 'store is not defined']
  for (const pattern of forbidden) {
    const found = errors.find(e => e.includes(pattern))
    expect(found, `禁止的控制台错误 "${pattern}": ${found}`).toBeUndefined()
  }
})

// ============================================================================
// 编辑类（破坏性，放最后；每个 test 自行清理）
// ============================================================================

test('T5a/T5b: 直接编辑 → Save 出现 → 落库', async ({ page }) => {
  await rows(page).first().click()
  await page.waitForTimeout(500)
  const original = await page.locator(TITLE).inputValue()
  expect(original.length).toBeGreaterThan(0)

  const suffix = ` [T5-${Date.now()}]`
  await page.locator(TITLE).fill(original + suffix)
  await page.waitForTimeout(300)
  await expect(saveBtn(page)).toBeVisible() // 有改动才出现 Save

  await saveBtn(page).click()
  await page.waitForTimeout(1500)
  const notes = await (await page.request.get('/api/notes')).json()
  expect(notes.some((n: any) => (n.title || '').includes(suffix))).toBe(true)

  // 清理：改回原标题
  await page.locator(TITLE).fill(original)
  await page.waitForTimeout(200)
  await saveBtn(page).click()
  await page.waitForTimeout(1000)
})

test('T5c: 切换笔记自动落盘未保存草稿', async ({ page }) => {
  await rows(page).first().click()
  await page.waitForTimeout(500)
  const original = await page.locator(TITLE).inputValue()

  const suffix = ` [T5c-${Date.now()}]`
  await page.locator(TITLE).fill(original + suffix)
  await page.waitForTimeout(300)
  // 不点 Save，直接切到另一条笔记
  await rows(page).nth(1).click()
  await page.waitForTimeout(1500)

  const notes = await (await page.request.get('/api/notes')).json()
  expect(notes.some((n: any) => (n.title || '').includes(suffix))).toBe(true)

  // 清理：切回被改名的笔记并恢复
  await rows(page).filter({ hasText: suffix }).first().click()
  await page.waitForTimeout(500)
  await page.locator(TITLE).fill(original)
  await page.waitForTimeout(200)
  await saveBtn(page).click()
  await page.waitForTimeout(1000)
  await expect(rows(page).filter({ hasText: suffix })).toHaveCount(0)
})

test('T6: New note 新建并自动选中空笔记', async ({ page }) => {
  await page.getByRole('button', { name: 'New note' }).click()
  await page.waitForTimeout(1200)

  // 自动选中：标题输入框为空（可直接打字）
  await expect(page.locator(TITLE)).toHaveValue('')

  // 清理：删掉刚新建的笔记
  await deleteBtn(page).click()
  await page.waitForTimeout(300)
  await deleteBtn(page).click()
  await page.waitForTimeout(1200)
})

test('T7: 删除需两步确认且真删', async ({ page }) => {
  await page.request.post('/api/notes', {
    data: { title: 'T7-temp-delete', body: 'temp', folder: '' },
  })
  await page.goto('/')
  await page.getByRole('button', { name: 'New note' }).waitFor({ timeout: 10000 })
  await page.waitForTimeout(800)

  const before = (await (await page.request.get('/api/notes')).json()).length
  const tempRow = rows(page).filter({ hasText: 'T7-temp-delete' })
  await expect(tempRow).toHaveCount(1)
  await tempRow.first().click()
  await page.waitForTimeout(500)

  // 第一步：只出确认，不删
  await deleteBtn(page).click()
  await page.waitForTimeout(400)
  await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible()
  expect((await (await page.request.get('/api/notes')).json()).length).toBe(before)

  // 第二步：确认删除
  await deleteBtn(page).click()
  await page.waitForTimeout(1500)
  expect((await (await page.request.get('/api/notes')).json()).length).toBe(before - 1)
})
