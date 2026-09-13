/**
 * 030-video-player 真实播放冒烟测试（PLAN-617 T-03/T-04/T-07/T-08 重写）。
 *
 * 取代 Plan 542 的「外壳真实、内核全假」版本：那一版断言的
 * `00:45 / 03:45`、`BigBuckBunny.mp4`、`1080P`、`H.264` 全部是被否定的常量，
 * 本版一律断言**元素与后端的实测值**。
 *
 * 前置：`AUTO_MEDIA_ROOT='E:\Video' auto run`（真后端 + Vite），
 * baseURL 由 VIDEO_PLAYER_URL 给出（本地用 playwright.p617.config.ts，
 * 因为 dev server 可能落在非 3030 端口）。
 *
 * 纪律：
 * - e2e **不加载 GB 级样本播放**（T2 只点 7.7MB 的 caelestia.mp4）；
 * - 队列断言与 `/api/media/scan` 自洽（不写死 14 这类会随目录变化的数字，
 *   但断言「嵌套项存在」——平铺实现只能得到 1 条，这是递归与否的判别式）。
 */
import { test, expect, type Page } from '@playwright/test'
import path from 'node:path'

async function waitForApp(page: Page) {
  await page.goto('/')
  await page.getByText('Video Player').first().waitFor({ timeout: 15000 })
  // 队列来自 /api/media/scan，等它落地
  await page.locator('.queue-item').first().waitFor({ timeout: 15000 })
}

function videoOf(page: Page) {
  return page.locator('video').first()
}

async function selectByTitle(page: Page, fragment: string) {
  await page.locator('.queue-item', { hasText: fragment }).first().click()
}

async function videoState(page: Page) {
  return videoOf(page).evaluate((v) => {
    const el = v as HTMLVideoElement
    return {
      src: el.currentSrc || el.src,
      paused: el.paused,
      currentTime: el.currentTime,
      duration: Number.isFinite(el.duration) ? el.duration : -1,
      volume: el.volume,
      muted: el.muted,
      playbackRate: el.playbackRate,
      videoWidth: el.videoWidth,
      videoHeight: el.videoHeight,
      readyState: el.readyState,
      errorCode: el.error ? el.error.code : null,
      audioTracks: (el as unknown as { audioTracks?: { length: number } }).audioTracks?.length ?? null,
      webkitAudioBytes: (el as unknown as { webkitAudioDecodedByteCount?: number })
        .webkitAudioDecodedByteCount ?? null,
    }
  })
}

/** 等元素真正开始走时间（最多 25s）。 */
async function waitUntilPlaying(page: Page) {
  await expect.poll(async () => (await videoState(page)).paused, { timeout: 25000 }).toBe(false)
  await expect
    .poll(async () => (await videoState(page)).currentTime, { timeout: 25000 })
    .toBeGreaterThan(0)
}

test('T1 队列来自真实目录且递归（按 rel_dir 分组）', async ({ page }) => {
  await waitForApp(page)

  // 与后端扫描自洽（这条断言同时证明前端读的是真后端，不是烘死的字面量）
  const res = await page.request.get('/api/media/scan')
  expect(res.ok()).toBe(true)
  const data = await res.json()
  const entries: Array<{ title: string; rel_dir: string }> = data.entries
  expect(entries.length).toBeGreaterThan(0)

  // 递归判据：存在嵌套目录条目（平铺实现只能得到根目录那 1 条）
  const nested = entries.filter((e) => e.rel_dir !== '')
  expect(nested.length, '存在嵌套条目').toBeGreaterThan(0)
  const nestedDirs = [...new Set(nested.map((e) => e.rel_dir))]
  expect(nestedDirs.length, '存在嵌套分组').toBeGreaterThan(0)

  // DOM：条目数 == 后端条数；分组标题逐一出现
  await expect(page.locator('.queue-item')).toHaveCount(entries.length)
  for (const dir of nestedDirs) {
    await expect(page.locator('.queue-group', { hasText: dir }).first()).toBeVisible()
  }
  // 根目录下的散文件也在列（"根目录" 是空 rel_dir 的可见文案）
  await expect(page.locator('.queue-group', { hasText: '根目录' }).first()).toBeVisible()
})

test('T1b 组内自然排序（S01E02 在 S01E10 之前）', async ({ page }) => {
  await waitForApp(page)
  const joined = (await page.locator('.queue-item').allInnerTexts()).join('\n')
  const e02 = joined.indexOf('S01E02')
  const e10 = joined.indexOf('S01E10')
  if (e02 >= 0 && e10 >= 0) {
    expect(e02, '自然序：E02 排在 E10 之前').toBeLessThan(e10)
  }
})

test('T2 真实起播：videoWidth>0 且 currentTime 递增', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  const s = await videoState(page)
  expect(s.videoWidth, '分辨率由元素实测').toBeGreaterThan(0)
  expect(s.videoHeight).toBeGreaterThan(0)
  expect(s.duration, '时长由 loadedmetadata 回灌').toBeGreaterThan(0)

  const t1 = (await videoState(page)).currentTime
  await page.waitForTimeout(700)
  const t2 = (await videoState(page)).currentTime
  expect(t2, 'currentTime 严格递增').toBeGreaterThan(t1)

  const body = await page.locator('body').innerText()
  expect(body, '旧版的烘死常量不再出现').not.toContain('03:45')
  // AC-16(b) 反面：可解码音轨的文件**不出现**「音轨不受支持」标注
  // （探针在 currentTime>1s 时回灌 true；无声不标，有声也不标）。
  await expect(page.locator('.audio-note')).toHaveCount(0)
  console.log('[T2 实测]', JSON.stringify(s))
})

test('T3 暂停真实作用于元素', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  await page.locator('.transport-playpause').click()
  await expect.poll(async () => (await videoState(page)).paused, { timeout: 10000 }).toBe(true)

  const a = (await videoState(page)).currentTime
  await page.waitForTimeout(1000)
  const b = (await videoState(page)).currentTime
  expect(Math.abs(b - a), '暂停后 1s 内时间不动').toBeLessThan(0.25)

  await page.locator('.transport-playpause').click()
  await expect.poll(async () => (await videoState(page)).paused, { timeout: 10000 }).toBe(false)
})

test('T4 seek 真实：点进度条中点 → currentTime 落在 duration*0.5 ±5%', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  // 进度条 = AutoUI `progress` + onseek（本轮新加的双端能力）：
  // 按下即 seek，按住拖动持续 seek。
  const bar = page.locator('div:has(> [role="progressbar"])').first()
  const box = await bar.boundingBox()
  expect(box, '进度条可见').not.toBeNull()
  await page.mouse.click(box!.x + box!.width * 0.5, box!.y + box!.height / 2)

  const s = await videoState(page)
  const expected = s.duration * 0.5
  await expect
    .poll(async () => Math.abs((await videoState(page)).currentTime - expected), { timeout: 10000 })
    .toBeLessThan(s.duration * 0.05)
})

test('T4b 拖拽 seek：从 20% 拖到 80%', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  const bar = page.locator('div:has(> [role="progressbar"])').first()
  const box = (await bar.boundingBox())!
  await page.mouse.move(box.x + box.width * 0.2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width * 0.5, box.y + box.height / 2, { steps: 5 })
  await page.mouse.move(box.x + box.width * 0.8, box.y + box.height / 2, { steps: 5 })
  await page.mouse.up()

  const s = await videoState(page)
  const expected = s.duration * 0.8
  await expect
    .poll(async () => Math.abs((await videoState(page)).currentTime - expected), { timeout: 10000 })
    .toBeLessThan(s.duration * 0.05)
})

test('T5 音量 / 静音 / 倍速真实作用于元素', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  // 音量：初始 80 → 元素 0.8（作者面 0..100 的换算）
  expect(Math.abs((await videoState(page)).volume - 0.8)).toBeLessThan(0.02)
  await page.locator('.vol-up').click()
  await expect
    .poll(async () => (await videoState(page)).volume, { timeout: 10000 })
    .toBeGreaterThan(0.8)

  // 静音
  await page.locator('.vol-mute').click()
  await expect.poll(async () => (await videoState(page)).muted, { timeout: 10000 }).toBe(true)
  await page.locator('.vol-mute').click()
  await expect.poll(async () => (await videoState(page)).muted, { timeout: 10000 }).toBe(false)

  // 倍速：1.0 → 1.25
  await page.locator('.rate-btn').click()
  await expect
    .poll(async () => (await videoState(page)).playbackRate, { timeout: 10000 })
    .toBeCloseTo(1.25, 2)
})

test('T7 上下曲真实：换源且为下一条目的流地址', async ({ page }) => {
  await waitForApp(page)
  const res = await page.request.get('/api/media/scan')
  const entries: Array<{ id: string }> = (await res.json()).entries

  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)
  const first = await videoState(page)
  expect(first.src).toContain(`/api/media/stream/${entries[0].id}`)

  // 传输键
  await page.locator('.transport-next').click()
  await expect
    .poll(async () => (await videoState(page)).src, { timeout: 15000 })
    .toContain(`/api/media/stream/${entries[1].id}`)
})

test('T9 不可播放源 → 视口出现错误文案（非静默黑屏）', async ({ page }) => {
  await waitForApp(page)
  await selectByTitle(page, 'caelestia')
  await waitUntilPlaying(page)

  // 直接在元素上指向一个不存在的令牌：走的是**真实**的 error 监听链路
  await videoOf(page).evaluate((v) => {
    ;(v as HTMLVideoElement).src = '/api/media/stream/deadbeefdeadbeef'
  })
  await expect(page.getByText('无法播放该文件')).toBeVisible({ timeout: 20000 })
})

test('T11 逐项真实状态：4K MKV 只断言实测元数据，不谎报', async ({ page }) => {
  await waitForApp(page)
  const res = await page.request.get('/api/media/scan')
  const entries: Array<{ title: string; name: string; rel_dir: string }> = (await res.json()).entries
  const mkv = entries.find((e) => e.name.toLowerCase().endsWith('.mkv'))
  test.skip(!mkv, '本机媒体根没有 .mkv 样本')

  await selectByTitle(page, mkv!.title)
  // 只等元数据（header），不长时间播放 GB 级样本
  await expect
    .poll(async () => (await videoState(page)).readyState, { timeout: 40000 })
    .toBeGreaterThanOrEqual(1)
  const s = await videoState(page)

  // (a) 画面维度由元素实测
  expect(s.videoWidth, 'MKV 画面可解').toBeGreaterThan(0)
  // (c) 界面**不出现**任何编造的音频/编码声称
  const body = await page.locator('body').innerText()
  expect(body, '不出现凭文件名编造的编码').not.toContain('HEVC')
  expect(body, '不出现凭文件名编造的码率').not.toContain('Mbps')
  expect(body, '不出现凭文件名编造的音轨信息').not.toContain('AAC Stereo')
  console.log('[T11 实测]', JSON.stringify(s), 'rel_dir=', mkv!.rel_dir)

  // (b) AC-16b 正向标注：让文件真实播放超过 1s（探针的判定条件），Chromium
  // 的 FFmpeg 构建解不出 Dolby Digital Plus —— 实测 webkitAudioDecodedByteCount
  // 恒 0（§4.1/§9.20），界面必须**主动**标注「音轨不受支持」，不假装有声。
  await waitUntilPlaying(page)
  await expect(page.locator('.audio-note')).toBeVisible({ timeout: 30000 })
  const body2 = await page.locator('body').innerText()
  expect(body2).toContain('音轨不受支持')

  await videoOf(page).evaluate((v) => (v as HTMLVideoElement).pause())
})

test('T8 本地文件浏览真实：File API → object URL → 元素可播（AC-09）', async ({ page }) => {
  await waitForApp(page)

  // 点「打开本地视频文件」→ 真文件对话框被打开（Playwright 拦截 chooser，
  // 等价于用户在其中选中文件）→ change → URL.createObjectURL → 换源。
  const mediaRoot = process.env.AUTO_MEDIA_ROOT || 'E:\\Video'
  const [chooser] = await Promise.all([
    page.waitForEvent('filechooser', { timeout: 15000 }),
    page.locator('.local-pick').click(),
  ])
  await chooser.setFiles(path.join(mediaRoot, 'caelestia.mp4'))

  // video 源变为 blob: 地址（不是任何 /api 流），且元素真实起播
  await expect
    .poll(async () => (await videoState(page)).src.startsWith('blob:'), { timeout: 15000 })
    .toBe(true)
  await waitUntilPlaying(page)
  const s = await videoState(page)
  expect(s.videoWidth, '本地文件真实解码').toBeGreaterThan(0)

  // 界面如实显示本地文件名与「本地文件」副标题
  const body = await page.locator('body').innerText()
  expect(body).toContain('caelestia.mp4')
  expect(body).toContain('本地文件')
  console.log('[T8 实测]', JSON.stringify(s))
})

test('T10 重新扫描真实生效（Rescan）', async ({ page }) => {
  await waitForApp(page)
  const res = await page.request.get('/api/media/scan')
  const entries: Array<unknown> = (await res.json()).entries

  await page.locator('.rescan-btn').click()
  await expect(page.getByText('媒体库已就绪').first()).toBeVisible({ timeout: 20000 })
  await expect(page.locator('.queue-item')).toHaveCount(entries.length)
})

test('T8b 静态文案：队列管理入口可见', async ({ page }) => {
  await waitForApp(page)
  const body = await page.locator('body').innerText()
  expect(body).toContain('播放队列')
  expect(body).toContain('重新扫描媒体库')
})
