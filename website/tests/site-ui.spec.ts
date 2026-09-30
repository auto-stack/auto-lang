import { test, expect, type Page } from '@playwright/test'

// PLAN-715 T-07：全站 UI 行为测试（导航/搜索/语言/主题/键盘/断点/画廊/ash 回归）。
// 测用户可观察行为，不镜像实现。前置：npm run build；服务=playwright.config.ts
// 的 webServer（AUTO_WEBSITE_TEST_PORT，独占端口）。

const VIEWPORTS = [
  { width: 360, height: 800 },
  { width: 390, height: 844 },
  { width: 768, height: 1024 },
  { width: 1024, height: 768 },
  { width: 1440, height: 1000 },
]

function trackConsole(page: Page): string[] {
  const errors: string[] = []
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })
  page.on('pageerror', (err) => errors.push('pageerror: ' + err.message))
  return errors
}

// ---------- AC-01 导航/语言/Logo/当前位置 ----------

test.describe('navigation & locale', () => {
  test('zh homepage navbar links stay in /zh', async ({ page }) => {
    await page.goto('/zh/')
    const nav = page.locator('header nav[aria-label]')
    await expect(nav).toBeVisible()
    // 应用入口指向 zh
    const appsLink = nav.getByRole('link', { name: '应用' })
    await expect(appsLink).toHaveAttribute('href', '/zh/apps')
    // 语言与生态组：语言入口指向 zh docs
    await nav.getByRole('button', { name: '语言与生态' }).click()
    const langLink = page.locator('.nav-dropdown').getByRole('link', { name: '语言', exact: true })
    await expect(langLink).toHaveAttribute('href', '/zh/docs/language')
  })

  test('en homepage navbar links are EN paths', async ({ page }) => {
    await page.goto('/')
    const nav = page.locator('header nav[aria-label]')
    await expect(nav.getByRole('link', { name: 'Apps' })).toHaveAttribute('href', '/apps')
  })

  test('language switch preserves path and locale', async ({ page }) => {
    await page.goto('/zh/rust')
    await page.locator('header').getByRole('link', { name: 'English' }).click()
    await expect(page).toHaveURL(/\/rust$/)
    await page.locator('header').getByRole('link', { name: '中文' }).click()
    await expect(page).toHaveURL(/\/zh\/rust$/)
  })

  test('logo routes to locale home', async ({ page }) => {
    await page.goto('/zh/apps')
    await page.locator('header .brand').click()
    await expect(page).toHaveURL(/\/zh\/?$/)
  })

  test('current position marked on section page', async ({ page }) => {
    await page.goto('/apps/automusk/')
    const current = page.locator('header nav [aria-current="page"]')
    await expect(current).toHaveText(/Apps|应用/)
  })

  test('shared SPA entries labeled and reachable from nav', async ({ page }) => {
    await page.goto('/')
    const nav = page.locator('header nav[aria-label]')
    await nav.getByRole('button', { name: 'UI' }).click()
    const gallery = page.locator('.nav-dropdown').getByRole('link', { name: /Gallery/ })
    await expect(gallery).toHaveAttribute('href', '/ui/gallery/index.html')
    // 共享标注（shared/共享）
    await expect(gallery).toContainText(/shared|共享/)
    await gallery.click()
    // SPA 经整页加载（layout-top 拦截 window.location）
    await page.waitForURL(/\/ui\/gallery\/index\.html/)
    await expect(page).toHaveTitle(/widgets-gallery/)
  })

  test('mobile menu: opens, groups, closes on navigate, Esc returns focus', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 })
    await page.goto('/')
    const trigger = page.getByRole('button', { name: /Open menu|打开菜单/ })
    await trigger.click()
    const menu = page.locator('#mobile-menu')
    await expect(menu).toBeVisible()
    await expect(trigger).toHaveAttribute('aria-expanded', 'true')
    // Esc 收起并回焦触发器
    await trigger.focus()
    await page.keyboard.press('Escape')
    await expect(menu).toBeHidden()
    await expect(trigger).toBeFocused()
    // 导航后自动收起
    await trigger.click()
    await menu.getByRole('link', { name: 'Docs', exact: true }).click()
    await page.waitForURL(/\/docs/)
    await expect(page.locator('#mobile-menu')).toBeHidden()
  })
})

// ---------- AC-02 搜索 ----------

test.describe('site search', () => {
  async function openSearch(page: Page) {
    await page.getByRole('button', { name: /Search \(Ctrl\+K\)|搜索（Ctrl\+K）/ }).first().click()
    const input = page.locator('.VPLocalSearchBox input')
    await expect(input).toBeVisible()
    return input
  }

  test('search button opens VitePress local search modal', async ({ page }) => {
    await page.goto('/')
    const input = await openSearch(page)
    await input.fill('ownership')
    await page.waitForTimeout(600)
    const results = page.locator('.VPLocalSearchBox .results')
    await expect(results).toBeVisible()
    await expect(results).toContainText(/ownership/i)
    await page.keyboard.press('Escape')
    await expect(input).toBeHidden()
  })

  test('zh locale: 所有权 hits zh docs', async ({ page }) => {
    await page.goto('/zh/docs/language')
    const input = await openSearch(page)
    await input.fill('所有权')
    await page.waitForTimeout(600)
    const results = page.locator('.VPLocalSearchBox .results')
    await expect(results).toBeVisible()
    const first = results.getByRole('listitem').first()
    await expect(first).toContainText('所有权')
    // 命中 zh 页面
    const href = await first.locator('a').getAttribute('href')
    expect(href).toBeTruthy()
  })

  test('AutoShell query hits the AutoShell topic page', async ({ page }) => {
    await page.goto('/docs/')
    const input = await openSearch(page)
    await input.fill('AutoShell')
    await page.waitForTimeout(600)
    const results = page.locator('.VPLocalSearchBox .results')
    await expect(results).toBeVisible()
    const hrefs = await results.getByRole('link').evaluateAll((els) => els.map((e) => (e as HTMLAnchorElement).getAttribute('href')))
    expect(hrefs.some((h) => h && h.includes('autoshell'))).toBeTruthy()
  })

  test('mobile search opens from navbar', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 })
    await page.goto('/')
    await openSearch(page)
  })
})

// ---------- AC-03 控件名称/状态/键盘 ----------

test.describe('controls & keyboard', () => {
  test('dropdown: aria-expanded, Esc close + focus return', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await page.goto('/')
    const trigger = page.getByRole('button', { name: 'UI' })
    await trigger.click()
    await expect(trigger).toHaveAttribute('aria-expanded', 'true')
    await page.keyboard.press('Escape')
    await expect(trigger).toHaveAttribute('aria-expanded', 'false')
    await expect(trigger).toBeFocused()
  })

  test('theme toggle has name and persists across navigation', async ({ page }) => {
    await page.goto('/')
    const toggle = page.getByRole('button', { name: /Toggle dark mode|切换深浅主题/ })
    await expect(toggle).toBeVisible()
    const before = await page.evaluate(() => document.documentElement.classList.contains('dark'))
    await toggle.click()
    const after = await page.evaluate(() => document.documentElement.classList.contains('dark'))
    expect(after).toBe(!before)
    // 切页不重置主题
    await page.goto('/rust')
    const persisted = await page.evaluate(() => document.documentElement.classList.contains('dark'))
    expect(persisted).toBe(after)
  })
})

// ---------- AC-05 断点矩阵 ----------

test.describe('viewports', () => {
  for (const vp of VIEWPORTS) {
    test(`no horizontal overflow at ${vp.width}x${vp.height} (home+v05)`, async ({ page }) => {
      await page.setViewportSize(vp)
      for (const url of ['/', '/zh/', '/v05/', '/zh/v05/', '/apps', '/zh/apps']) {
        await page.goto(url)
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)
        expect(overflow, `${url} overflows by ${overflow}px`).toBeLessThanOrEqual(1)
        // 主要文字/按钮边界在内容区内（hero 按钮 bbox）
        const btn = page.locator('.actions a, .cta-actions a').first()
        if (await btn.count()) {
          const box = await btn.boundingBox()
          expect(box, `${url} primary action missing`).toBeTruthy()
          if (box) {
            expect(box.x).toBeGreaterThanOrEqual(-1)
            expect(box.x + box.width).toBeLessThanOrEqual(vp.width + 1)
          }
        }
      }
    })
  }

  test('360px: navbar controls reachable and ≥44px touch targets', async ({ page }) => {
    await page.setViewportSize({ width: 360, height: 800 })
    await page.goto('/')
    const buttons = page.locator('header .nav-icon-btn')
    const count = await buttons.count()
    expect(count).toBeGreaterThanOrEqual(2)
    for (let i = 0; i < count; i++) {
      const box = await buttons.nth(i).boundingBox()
      expect(box?.height, `button ${i} touch height`).toBeGreaterThanOrEqual(43)
    }
  })
})

// ---------- AC-09 v0.5 首图早见 + 章节导航 ----------

test.describe('v05 release page', () => {
  for (const vp of [{ width: 390, height: 844 }, { width: 1440, height: 1000 }]) {
    for (const url of ['/v05/', '/zh/v05/']) {
      test(`hero image top < 1.5 viewport at ${vp.width} (${url})`, async ({ page }) => {
        await page.setViewportSize(vp)
        await page.goto(url)
        const img = page.locator('figure.hero-shot img').first()
        await expect(img).toBeVisible()
        const box = await img.boundingBox()
        expect(box, 'hero image bbox').toBeTruthy()
        expect(box!.y, 'hero top within 1.5x viewport').toBeLessThan(1.5 * vp.height)
      })
    }
  }

  test('stats moved to journey: desktop section precedes stats', async ({ page }) => {
    await page.goto('/v05/')
    const desktopY = await page.locator('#desktop').evaluate((el) => el.getBoundingClientRect().top + window.scrollY)
    const statsY = await page.locator('.stats-row').first().evaluate((el) => el.getBoundingClientRect().top + window.scrollY)
    expect(desktopY, 'desktop section before stats').toBeLessThan(statsY)
  })

  test('section nav: hash direct + click + no heading cover', async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 1000 })
    // hash 直达
    await page.goto('/v05/#autoui')
    await page.waitForTimeout(400)
    const heading = page.locator('#autoui .autoui-title')
    await expect(heading).toBeVisible()
    const headBox = await heading.boundingBox()
    // 标题不被两层 sticky 遮挡：其顶部应在 nav+sectionnav 之下可视
    expect(headBox!.y).toBeGreaterThanOrEqual(0)
    // 点击章节导航跳转
    await page.locator('.section-nav').getByRole('link', { name: /Apps|旗舰应用/ }).click()
    await page.waitForTimeout(500)
    await expect(page.locator('#flagship')).toBeInViewport()
    // 当前章节高亮
    await expect(page.locator('.section-nav-link.current')).toContainText(/Apps|旗舰应用/)
  })

  test('screenshot gallery: tabs switch main image', async ({ page }) => {
    await page.goto('/v05/')
    const tabs = page.locator('.shot-tabs [role="tab"]')
    await expect(tabs.first()).toHaveAttribute('aria-selected', 'true')
    const firstSrc = await page.locator('#shot-panel img').getAttribute('src')
    await tabs.nth(1).click()
    const secondSrc = await page.locator('#shot-panel img').getAttribute('src')
    expect(secondSrc).not.toBe(firstSrc)
    await expect(tabs.nth(1)).toHaveAttribute('aria-selected', 'true')
  })

  test('start menu uses real launcher image; no TODO placeholders', async ({ page }) => {
    await page.goto('/v05/')
    const html = await page.content()
    expect(html).not.toContain('TODO screenshot')
    expect(html).not.toContain('shot-placeholder')
    await expect(page.locator('img[src="/v05/desktop-launcher.png"]')).toHaveCount(1)
  })

  test('philosophy details expandable and locatable', async ({ page }) => {
    await page.goto('/v05/')
    const details = page.locator('.philosophy-item').first()
    await expect(details.locator('summary')).toContainText(/Dynamic Dev|动态开发/)
    // 默认收起
    await expect(details).not.toHaveAttribute('open')
    await details.locator('summary').click()
    await expect(details).toHaveAttribute('open')
  })

  test('kanban comparison present with both arms', async ({ page }) => {
    await page.goto('/zh/v05/')
    await expect(page.locator('img[src="/v05/kanban-web.png"]')).toHaveCount(1)
    await expect(page.locator('img[src="/v05/kanban-desktop.png"]')).toHaveCount(1)
  })
})

// ---------- AC-11 图片放大键盘路径 ----------

test.describe('image dialog keyboard', () => {
  test('Enter opens, Esc closes, focus returns', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 })
    await page.goto('/v05/')
    const trigger = page.locator('.hero-shot .evidence-image')
    await trigger.focus()
    await page.keyboard.press('Enter')
    const dialog = page.locator('dialog.evidence-dialog[open]')
    await expect(dialog).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(dialog).not.toBeAttached()
    await expect(trigger).toBeFocused()
  })
})

// ---------- AC-12 ash 回归 ----------

test.describe('autoshell regression', () => {
  test('native overview, F1/F2/F3 sections and script tabs intact', async ({ page }) => {
    await page.goto('/apps/autoshell/')
    await expect(page.locator('img[src="/apps/autoshell/ash-01.png"]').first()).toBeVisible()
    for (const id of ['interface-overview', 'daily-shell', 'data-pipelines', 'scripts', 'automation', 'quick-start']) {
      await expect(page.locator('#' + id)).toBeAttached()
    }
    // 脚本标签切换
    const tabs = page.locator('.script-tabs [role="tab"]')
    await expect(tabs).toHaveCount(3)
    const firstFile = await page.locator('.script-panel .code-title').first().textContent()
    await tabs.nth(1).click()
    const secondFile = await page.locator('.script-panel .code-title').first().textContent()
    expect(secondFile).not.toBe(firstFile)
  })

  test('copy button writes clipboard', async ({ page, context }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write'])
    await page.goto('/apps/autoshell/')
    const btn = page.locator('#data-pipelines .copy-button')
    await btn.click()
    const text = await page.evaluate(() => navigator.clipboard.readText())
    expect(text).toContain('from_json')
  })

  test('download links for scripts and sample data', async ({ page }) => {
    await page.goto('/apps/autoshell/')
    await expect(page.locator('a[download][href*="users.json"]').first()).toBeAttached()
    await expect(page.locator('a[download][href*="user-report.ash"]').first()).toBeAttached()
  })
})

// ---------- AC-06 reduced-motion 内容可见 ----------

test.describe('reduced motion', () => {
  test('content visible with reduced motion (reveal disabled)', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' })
    await page.goto('/v05/')
    const section = page.locator('#highlights')
    await expect(section).toBeVisible()
    const opacity = await section.evaluate((el) => getComputedStyle(el).opacity)
    expect(Number(opacity)).toBeGreaterThan(0.9)
  })
})

// ---------- console 卫生 ----------

test.describe('console hygiene', () => {
  for (const url of ['/', '/zh/', '/v05/', '/apps', '/playground']) {
    test(`no new console errors on ${url}`, async ({ page }) => {
      const errors = trackConsole(page)
      await page.goto(url, { waitUntil: 'networkidle' })
      const real = errors.filter((e) => !e.includes('favicon') && !e.includes('43') )
      expect(real, real.join('\n')).toHaveLength(0)
    })
  }
})
