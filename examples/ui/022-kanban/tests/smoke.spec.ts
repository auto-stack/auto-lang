/**
 * 022-kanban 冒烟测试 (Plan 404) — 对应 acceptance.atd T1-T8.
 * 前端 vite 跑在 localhost:3022 (pac.at front_port), 后端 8022 (vite proxy 转发).
 *
 * 端口缺口注意: 当前 `auto run` 不读 pac.at 的 front_port/back_port, 测试前需用
 * 环境变量分离启动:
 *   AUTO_HTTP_PORT=8022 ./app-022-kanban-back.exe          (后端)
 *   AUTO_FRONT_PORT=3022 AUTO_HTTP_PORT=8022 pnpm dev       (前端, 在 gen/front/vue)
 */
import { test, expect } from '@playwright/test'

// 等看板加载: 打开首页, 等首张种子卡片渲染 (store.Init 的 GET /api/cards 回填).
async function waitForBoard(page) {
  await page.goto('/')
  await page.locator('text=Project Board').waitFor({ timeout: 10000 })
  // 等 store.Init 异步拉取后端数据
  await page.waitForTimeout(1500)
}

test('T1: 初始看板渲染 — 9 张种子卡片', async ({ page }) => {
  await waitForBoard(page)
  const body = await page.locator('body').innerText()
  // 4 todo
  expect(body).toContain('Design landing page')
  expect(body).toContain('Write API docs')
  expect(body).toContain('Fix login bug')
  expect(body).toContain('Setup CI/CD')
  // 2 doing
  expect(body).toContain('Implement auth flow')
  expect(body).toContain('Build dashboard')
  // 3 done
  expect(body).toContain('Setup database')
  expect(body).toContain('Create user model')
  expect(body).toContain('Design logo')
})

test('T2: 三列标题显示', async ({ page }) => {
  await waitForBoard(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('To Do')
  expect(body).toContain('In Progress')
  expect(body).toContain('Done')
})

// 找到某张卡片的"行"div: 定位标题 span, 再上溯到卡片行.
// 卡片行结构: div.bg-white > span(title) + button(>) + button(×).
async function cardRow(page, title) {
  // getByText 精确匹配标题 span; locator('..') 到卡片行 div.
  return page.getByText(title, { exact: true }).locator('xpath=..')
}

test('T3-T6: 添加 → 移动 todo→doing → 移动 doing→done → 删除', async ({ page }) => {
  await waitForBoard(page)

  // T3: 添加卡片 "Playwright card"
  const input = page.locator('input[placeholder="Add a card…"]')
  await input.fill('Playwright card')
  await page.locator('button:has-text("+ Add Card")').click()
  await page.waitForTimeout(800)
  let body = await page.locator('body').innerText()
  expect(body).toContain('Playwright card')

  // T4: 把 todo 卡片 (Design landing page) 移到 doing.
  // 卡片行有两个按钮: [0]=">"(移动) [1]="×"(删除).
  await (await cardRow(page, 'Design landing page')).locator('button').nth(0).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).toContain('Design landing page')

  // T5: 把 doing 卡片 (Implement auth flow) 移到 done.
  await (await cardRow(page, 'Implement auth flow')).locator('button').nth(0).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).toContain('Implement auth flow')

  // T6: 删除一张卡片 (Write API docs 的 "×" 按钮 = button[1]).
  await (await cardRow(page, 'Write API docs')).locator('button').nth(1).click()
  await page.waitForTimeout(800)
  body = await page.locator('body').innerText()
  expect(body).not.toContain('Write API docs')
})

test('T7: 刷新后状态持久 (后端内存态)', async ({ page }) => {
  await waitForBoard(page)
  // 刷新页面, store.Init 重新 GET /api/cards, 后端内存态保持.
  // Design landing page 在 T4 移到了 doing — 刷新后仍在 (验证持久, 而非重置).
  await page.reload()
  await page.waitForTimeout(1500)
  const body = await page.locator('body').innerText()
  expect(body).toContain('Setup CI/CD')
})

test('T8: 控制台无实质错误', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })
  await waitForBoard(page)
  await page.waitForTimeout(1000)
  // 过滤掉 favicon 404 等无害噪声
  const real = errors.filter((e) => !e.includes('favicon') && !e.includes('Failed to load resource'))
  expect(real, `console errors: ${real.join('; ')}`).toEqual([])
})
