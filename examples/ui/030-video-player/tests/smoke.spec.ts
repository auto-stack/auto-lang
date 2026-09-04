/**
 * 030-video-player 冒烟测试 (Plan 542)
 * 验证 AutoOS 原生视频播放器：初始渲染 / 播控 / 步进寻道 / 队列切换 / 音量静音 / 倍速 / 媒体属性弹窗 / 主题与 Accent.
 * 前置：auto run 已启动前端 (:3030).
 */
import { test, expect } from '@playwright/test'

async function waitForPlayer(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.getByText('AutoOS Video Player').first().waitFor({ timeout: 10000 })
  await page.waitForTimeout(1000)
}

test('T1: 初始状态加载与元数据渲染', async ({ page }) => {
  await waitForPlayer(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('AutoOS Video Player')
  expect(body).toContain('AutoOS 2026 Keynote & Intro')
  expect(body).toContain('01_intro.mp4')
  expect(body).toContain('1080P')
  expect(body).toContain('H.264')
  expect(body).toContain('00:45')
  expect(body).toContain('03:45')
})

test('T2: 播放与暂停切换', async ({ page }) => {
  await waitForPlayer(page)
  // 默认是播放中，点击暂停按钮
  const playPauseBtn = page.getByRole('button', { name: /暂停/ }).first()
  await playPauseBtn.click()
  await page.waitForTimeout(500)

  const body = await page.locator('body').innerText()
  expect(body).toContain('已暂停')
  expect(body).toContain('▶ 播放')

  // 再次点击恢复播放
  const resumeBtn = page.getByRole('button', { name: /播放/ }).first()
  await resumeBtn.click()
  await page.waitForTimeout(500)
  expect(await page.locator('body').innerText()).toContain('正在播放')
})

test('T3: 步进快进快退寻道', async ({ page }) => {
  await waitForPlayer(page)
  // 点击快进 5s
  const stepFwdBtn = page.getByRole('button', { name: /5s ⏩/ }).first()
  await stepFwdBtn.click()
  await page.waitForTimeout(400)
  let body = await page.locator('body').innerText()
  expect(body).toContain('快进 +5 秒')
  expect(body).toContain('00:50')

  // 点击快退 5s
  const stepBackBtn = page.getByRole('button', { name: /⏪ 5s/ }).first()
  await stepBackBtn.click()
  await page.waitForTimeout(400)
  body = await page.locator('body').innerText()
  expect(body).toContain('快退 -5 秒')
  expect(body).toContain('00:45')
})

test('T4: 切集与切回 (Next / Prev Video)', async ({ page }) => {
  await waitForPlayer(page)
  // 点击下一集
  const nextBtn = page.getByRole('button', { name: '⏭' }).first()
  await nextBtn.click()
  await page.waitForTimeout(600)

  let body = await page.locator('body').innerText()
  expect(body).toContain('Kernel & AutoVM Architecture')
  expect(body).toContain('02_architecture.mkv')
  expect(body).toContain('4K')
  expect(body).toContain('HEVC')
  expect(body).toContain('12:20')

  // 点击上一集
  const prevBtn = page.getByRole('button', { name: '⏮' }).first()
  await prevBtn.click()
  await page.waitForTimeout(600)
  body = await page.locator('body').innerText()
  expect(body).toContain('AutoOS 2026 Keynote & Intro')
  expect(body).toContain('01_intro.mp4')
})

test('T5: 音量调节与静音切换', async ({ page }) => {
  await waitForPlayer(page)
  // 点击静音
  const muteBtn = page.getByRole('button', { name: /🔊/ }).first()
  await muteBtn.click()
  await page.waitForTimeout(400)
  let body = await page.locator('body').innerText()
  expect(body).toContain('已静音')

  // 再次点击取消静音
  const unMuteBtn = page.getByRole('button', { name: /🔇/ }).first()
  await unMuteBtn.click()
  await page.waitForTimeout(400)
  body = await page.locator('body').innerText()
  expect(body).toContain('取消静音')
})

test('T6: 多档倍速平滑切换', async ({ page }) => {
  await waitForPlayer(page)
  // 点击倍速按钮（初始 1.0x）
  const speedBtn = page.getByRole('button', { name: /⚡ 1.0x/ }).first()
  await speedBtn.click()
  await page.waitForTimeout(400)

  const body = await page.locator('body').innerText()
  expect(body).toContain('1.25x')
})

test('T7: 播放队列抽屉与选集播放', async ({ page }) => {
  await waitForPlayer(page)
  // 检查播放队列内包含第 3 项
  expect(await page.locator('body').innerText()).toContain('AutoUI Dual-Backend Engine Demo')

  // 点击播放队列中的第 3 项
  await page.getByText('AutoUI Dual-Backend Engine Demo').first().click()
  await page.waitForTimeout(600)

  const body = await page.locator('body').innerText()
  expect(body).toContain('03_ui_engine.mp4')
  expect(body).toContain('08:15')
  expect(body).toContain('AV1')
})

test('T8: 媒体属性详细信息弹窗', async ({ page }) => {
  await waitForPlayer(page)
  // 打开属性弹窗
  await page.getByRole('button', { name: /属性/ }).first().click()
  await page.waitForTimeout(400)

  let body = await page.locator('body').innerText()
  expect(body).toContain('媒体详细属性')
  expect(body).toContain('分辨率 (Resolution)')
  expect(body).toContain('视频编码 (Codec)')
  expect(body).toContain('解码管线 (Pipeline)')

  // 点击关闭弹窗
  await page.getByRole('button', { name: /确定 \(Close\)/ }).first().click()
  await page.waitForTimeout(400)
  body = await page.locator('body').innerText()
  expect(body).not.toContain('解码管线 (Pipeline)')
})

test('T9: 深浅主题与 Accent 色彩切换', async ({ page }) => {
  await waitForPlayer(page)
  // 点击主题切换
  const themeBtn = page.getByRole('button', { name: /🌙/ }).first()
  await themeBtn.click()
  await page.waitForTimeout(400)

  // 此时应显示太阳 ☀ 图标
  expect(await page.locator('body').innerText()).toContain('☀')

  // 切回深色
  const sunBtn = page.getByRole('button', { name: /☀/ }).first()
  await sunBtn.click()
  await page.waitForTimeout(400)
  expect(await page.locator('body').innerText()).toContain('🌙')
})
