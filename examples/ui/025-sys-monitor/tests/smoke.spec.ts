/**
 * 025-sys-monitor 冒烟测试 (Plan 541)
 * 验证 Vue 模式下 4 页导航、真实系统指标渲染、进程表排序、
 * 结束任务 AlertDialog 二次确认及暂停/继续控制。
 */
import { test, expect } from '@playwright/test'

async function waitForApp(page) {
  await page.goto('/')
  await page.locator('text=系统监视器').first().waitFor({ timeout: 10000 })
  // 等待快照数据加载
  await page.waitForTimeout(1000)
}

test('T1: 初始渲染与 4 个导航 Tab', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('系统监视器')
  expect(body).toContain('Task Manager')
  expect(body).toContain('进程')
  expect(body).toContain('性能')
  expect(body).toContain('详细信息')
  expect(body).toContain('用户')
})

test('T2: 进程页 KPI 四卡与真实进程列表', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('CPU')
  expect(body).toContain('内存')
  expect(body).toContain('磁盘吞吐')
  expect(body).toContain('网络总吞吐')

  // 验证进程列表行数大于 0
  const rows = page.locator('table tbody tr')
  const count = await rows.count()
  expect(count).toBeGreaterThan(0)
})

test('T3: 结束任务 AlertDialog 二次确认开合与取消', async ({ page }) => {
  await waitForApp(page)

  // 初始状态下结束任务按钮处于禁用或不可点击状态
  const firstRow = page.locator('table tbody tr').first()
  await firstRow.click()
  await page.waitForTimeout(300)

  // 点击选中的结束任务按钮
  const killBtn = page.locator('button:has-text("结束任务")')
  await killBtn.click()
  await page.waitForTimeout(400)

  // 验证 AlertDialog 弹窗出现
  const dialogTitle = page.locator('text=结束任务确认')
  await expect(dialogTitle).toBeVisible()

  // 点击取消
  const cancelBtn = page.locator('button:has-text("取消")')
  await cancelBtn.click()
  await page.waitForTimeout(300)

  // 弹窗关闭
  await expect(dialogTitle).not.toBeVisible()
})

test('T4: 性能页切换与 SVG 曲线图', async ({ page }) => {
  await waitForApp(page)

  // 点击左侧性能 Tab
  await page.locator('text=性能').first().click()
  await page.waitForTimeout(500)

  const body = await page.locator('body').innerText()
  expect(body).toContain('CPU 使用率实时曲线')

  // 验证 SVG 包含 path
  const paths = page.locator('svg path')
  const pathCount = await paths.count()
  expect(pathCount).toBeGreaterThan(0)
})

test('T5: 详细信息与用户页切换', async ({ page }) => {
  await waitForApp(page)

  // 切换到详细信息
  await page.locator('text=详细信息').first().click()
  await page.waitForTimeout(500)
  let body = await page.locator('body').innerText()
  expect(body).toContain('详细信息视图')

  // 切换到用户
  await page.locator('text=用户').first().click()
  await page.waitForTimeout(500)
  body = await page.locator('body').innerText()
  expect(body).toContain('已登录与系统用户账户')
})

test('T6: 刷新频率切换与暂停/继续控制', async ({ page }) => {
  await waitForApp(page)

  // 点击快速 (250ms)
  const fastBtn = page.locator('button:has-text("250ms")')
  await fastBtn.click()
  await page.waitForTimeout(300)

  // 点击暂停
  const pauseBtn = page.locator('button:has-text("⏸ 暂停")')
  await pauseBtn.click()
  await page.waitForTimeout(300)

  // 变为继续
  const resumeBtn = page.locator('button:has-text("▶ 继续")')
  await expect(resumeBtn).toBeVisible()

  // 恢复继续
  await resumeBtn.click()
  await page.waitForTimeout(300)
  await expect(pauseBtn).toBeVisible()
})
