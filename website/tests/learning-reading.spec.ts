import { test, expect, type Page } from '@playwright/test'

// PLAN-718 T-06：学习入口与阅读导航行为测试（AC-01/02/04/05/06/07/08）。
// 测用户可观察行为，不镜像实现。前置：npm run build；服务=playwright.config.ts
// 的 webServer（AUTO_WEBSITE_TEST_PORT 独占端口 + CI=1 禁止复用未知旧服务）。

const READ_VIEWPORTS = [
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

async function openSearch(page: Page): Promise<ReturnType<Page['locator']>> {
  await page
    .getByRole('button', { name: /Search \(Ctrl\+K\)|搜索（Ctrl\+K）/ })
    .first()
    .click()
  const input = page.locator('.VPLocalSearchBox input')
  await expect(input).toBeVisible()
  return input
}

// ---------- AC-01 学习入口 ----------

test.describe('learning hubs', () => {
  for (const [hub, path] of [
    ['docs EN', '/docs/'],
    ['docs ZH', '/zh/docs/'],
    ['books EN', '/books/'],
    ['books ZH', '/zh/books/'],
  ] as const) {
    test(`${hub} hub renders SSR learning cards with real links`, async ({ page }) => {
      await page.goto(path, { waitUntil: 'networkidle' })
      const hubRoot = page.locator('.learning-hub')
      await expect(hubRoot).toBeVisible()
      // 静态 HTML（SSR）即含卡片文本——无需等待水合即可见
      if (hub.startsWith('docs')) {
        await expect(hubRoot.locator('a', { hasText: /Auto Tour|Auto 巡礼/ }).first()).toBeVisible()
        await expect(hubRoot.locator('a', { hasText: /syntax|语法/ }).first()).toBeVisible()
        const href = await hubRoot.locator('a', { hasText: /Auto Tour|Auto 巡礼/ }).first().getAttribute('href')
        expect(href).toMatch(/\/(zh\/)?docs\/tour\/ch01-hello/)
      } else {
        // 八本书全部展示且有真实章节入口
        const bookCards = hubRoot.locator('.lh-book-card')
        await expect(bookCards).toHaveCount(8)
        const entryHrefs = await hubRoot.locator('.lh-cta').evaluateAll((els) =>
          els.map((e) => (e as HTMLAnchorElement).getAttribute('href')),
        )
        expect(entryHrefs.length).toBe(8)
        for (const h of entryHrefs) {
          expect(h).toMatch(/^\/(zh\/)?books\/[a-z-]+\/ch/)
        }
        const featured = hubRoot.locator('.lh-book-featured')
        await expect(featured.locator('.lh-card-title')).toHaveText(/The Auto Programming Language|Auto 编程语言（主书）/)
      }
    })
  }

  test('local search finds docs hub entry text (EN + ZH)', async ({ page }) => {
    await page.goto('/docs/', { waitUntil: 'networkidle' })
    const input = await openSearch(page)
    await input.fill('guided tour')
    const results = page.locator('.VPLocalSearchBox .results')
    await expect(results.locator('a').first()).toBeVisible({ timeout: 15_000 })
    const hrefs = await results.locator('a').evaluateAll((els) => els.map((e) => e.getAttribute('href') ?? ''))
    expect(hrefs.some((h) => h.includes('/docs/tour'))).toBeTruthy()
    await page.keyboard.press('Escape')

    await page.goto('/zh/docs/', { waitUntil: 'networkidle' })
    const inputZh = await openSearch(page)
    await inputZh.fill('巡礼')
    const resultsZh = page.locator('.VPLocalSearchBox .results')
    await expect(resultsZh.locator('a').first()).toBeVisible({ timeout: 15_000 })
  })
})

// ---------- AC-02 导航完整 ----------

test.describe('docs navigation completeness', () => {
  test('docs sidebar starts with Start here -> tour, EN and ZH', async ({ page }) => {
    for (const [path, label, href] of [
      ['/docs/', 'Start here', '/docs/tour/ch01-hello'],
      ['/zh/docs/', '开始使用', '/zh/docs/tour/ch01-hello'],
    ] as const) {
      await page.goto(path, { waitUntil: 'networkidle' })
      const first = page.locator('.VPSidebar .group > .item > .link, .VPSidebar a').first()
      const sidebarText = await page.locator('.VPSidebar').textContent()
      expect(sidebarText).toContain(label)
      const link = page.locator(`.VPSidebar a[href="${href}"]`).first()
      await expect(link).toBeVisible()
      void first
    }
  })

  test('legacy quick-link exits exist as real pages', async ({ page }) => {
    for (const p of ['/docs/syntax', '/docs/roadmap', '/docs/migration-guide', '/docs/language/specification']) {
      const resp = await page.goto(p, { waitUntil: 'domcontentloaded' })
      expect(resp?.status(), p).toBe(200)
      const notFound = await page.locator('.NotFound').count()
      expect(notFound, p).toBe(0)
    }
  })

  test('every book has its own sidebar: chapters visible, other books absent', async ({ page }) => {
    await page.goto('/books/rust/ch03-common-concepts', { waitUntil: 'networkidle' })
    const sidebar = page.locator('.VPSidebar')
    await expect(sidebar.locator('a[href="/books/rust/ch04-ownership"]')).toBeVisible()
    await expect(sidebar.locator('a[href^="/books/tapl/"]')).toHaveCount(0)
  })
})

// ---------- AC-04 移动阅读几何 ----------

test.describe('reading geometry across widths', () => {
  for (const vp of READ_VIEWPORTS) {
    test(`no page-level horizontal overflow at ${vp.width}px (article + chapter)`, async ({ page }) => {
      for (const p of ['/docs/features/actor-concurrency', '/zh/books/tapl/ch01-getting-started']) {
        await page.setViewportSize(vp)
        await page.goto(p, { waitUntil: 'networkidle' })
        const overflow = await page.evaluate(() => ({
          scroll: document.documentElement.scrollWidth,
          client: document.documentElement.clientWidth,
        }))
        expect(overflow.scroll, `${p} @${vp.width}`).toBeLessThanOrEqual(overflow.client + 1)
      }
    })
  }

  test('mobile: local nav sits below navbar when scrolled; anchor clears both bars', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 })
    await page.goto('/docs/features/actor-concurrency', { waitUntil: 'networkidle' })
    await page.evaluate(() => window.scrollTo(0, 800))
    await page.waitForTimeout(200)
    const geo = await page.evaluate(() => {
      const nav = document.querySelector('.site-navbar')
      const local = document.querySelector('.VPLocalNav')
      return {
        navbarBottom: Math.round(nav!.getBoundingClientRect().bottom),
        localTop: Math.round(local!.getBoundingClientRect().top),
      }
    })
    expect(geo.localTop).toBeGreaterThanOrEqual(geo.navbarBottom)

    // 锚点：目录弹层点击后标题位于导航条（不含已收起弹层）之下
    await page.evaluate(() => window.scrollTo(0, 0))
    await page.locator('.VPLocalNavOutlineDropdown button').first().click()
    await page.waitForTimeout(300)
    await page.locator('.VPLocalNavOutlineDropdown .items a').nth(1).click()
    await page.waitForTimeout(800)
    const landed = await page.evaluate(() => {
      const hash = location.hash
      const el = hash ? document.getElementById(decodeURIComponent(hash.slice(1))) : null
      // .container = 目录条本身；弹层展开态会抬高 .VPLocalNav 的整体盒
      const bar = document.querySelector('.VPLocalNav .container')!
      return { top: el ? Math.round(el.getBoundingClientRect().top) : -1, barBottom: Math.round(bar.getBoundingClientRect().bottom) }
    })
    expect(landed.top).toBeGreaterThanOrEqual(landed.barBottom)
  })

  test('390px: mobile reading controls are reachable touch targets (>=44px)', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 })
    await page.goto('/docs/features/actor-concurrency', { waitUntil: 'networkidle' })
    const menu = page.locator('.VPLocalNav button.menu')
    await expect(menu).toBeVisible()
    const box = await menu.boundingBox()
    expect(box!.height).toBeGreaterThanOrEqual(44)
    const outlineBtn = page.locator('.VPLocalNavOutlineDropdown button').first()
    const ob = await outlineBtn.boundingBox()
    expect(ob!.height).toBeGreaterThanOrEqual(44)
  })
})

// ---------- AC-05 章节衔接 ----------

test.describe('chapter boundaries', () => {
  test('first chapter has no prev; middle prev/next stay in book; last has no next', async ({ page }) => {
    const pagers = async (p: string) => {
      await page.goto(p, { waitUntil: 'domcontentloaded' })
      // 缺席的 pager 不能用 getAttribute 等（会悬挂到超时）——先 count 再取
      const href = async (sel: string) => {
        const loc = page.locator(sel)
        return (await loc.count()) ? loc.getAttribute('href') : null
      }
      return { prev: await href('a.pager-link.prev'), next: await href('a.pager-link.next') }
    }
    expect(await pagers('/books/tapl/ch00-introduction')).toEqual({ prev: null, next: '/books/tapl/ch01-getting-started' })
    expect(await pagers('/books/tapl/ch02-variables-operators')).toEqual({
      prev: '/books/tapl/ch01-getting-started',
      next: '/books/tapl/ch03-functions',
    })
    expect((await pagers('/books/tapl/appendix-d-stdlib-index')).next).toBeNull()
    // 不串书
    const rust = await pagers('/books/rust/ch03-common-concepts')
    expect(rust.prev).toMatch(/^\/books\/rust\//)
    expect(rust.next).toMatch(/^\/books\/rust\//)
  })

  test('docs hub has no meaningless next page (Autocache gone)', async ({ page }) => {
    await page.goto('/docs/', { waitUntil: 'networkidle' })
    // hub 不是章节序列：无"下一页"pager 即可（侧栏含 Autocache 指南属正常目录）
    await expect(page.locator('.vp-doc a.pager-link.next')).toHaveCount(0)
    await expect(page.locator('a.pager-link.next')).toHaveCount(0)
  })
})

// ---------- AC-06 语言与来源 ----------

test.describe('locale controls and author sources', () => {
  test('reader controls follow locale', async ({ page }) => {
    await page.goto('/docs/features/actor-concurrency', { waitUntil: 'domcontentloaded' })
    let html = await page.content()
    expect(html).toContain('Previous page')
    expect(html).toContain('Next page')
    await page.goto('/zh/docs/features/actor-concurrency', { waitUntil: 'domcontentloaded' })
    html = await page.content()
    expect(html).toContain('上一页')
    expect(html).toContain('下一页')
    expect(html).toContain('此页内容')
  })

  test('edit links point to verified real sources per locale and repo', async ({ page }) => {
    await page.goto('/docs/features/actor-concurrency', { waitUntil: 'domcontentloaded' })
    let href = await page.locator('a.edit-link-button').getAttribute('href')
    expect(href).toBe('https://github.com/auto-stack/auto-lang/edit/master/docs/features/actor-concurrency.md')

    // ZH 页有 .cn.md 译本——作者源是 .cn.md
    await page.goto('/zh/docs/features/actor-concurrency', { waitUntil: 'domcontentloaded' })
    href = await page.locator('a.edit-link-button').getAttribute('href')
    expect(href).toBe('https://github.com/auto-stack/auto-lang/edit/master/docs/features/actor-concurrency.cn.md')

    // 书页 → gitee book 仓
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'domcontentloaded' })
    href = await page.locator('a.edit-link-button').getAttribute('href')
    expect(href).toBe('https://gitee.com/auto-stack/book/edit/master/tapl/ch01-getting-started.md')

    // 生成入口不显示虚假 editLink
    await page.goto('/books/', { waitUntil: 'domcontentloaded' })
    expect(await page.locator('a.edit-link-button').count()).toBe(0)
  })
})

// ---------- AC-07/08 代码工具栏与交互 ----------

test.describe('auto fence toolbar', () => {
  test('bilingual labels, locked note and locale-correct playground link', async ({ page }) => {
    await page.setViewportSize({ width: 360, height: 800 })
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'networkidle' })
    const fence = page.locator('.auto-fence').first()
    await expect(fence.locator('button.af-run')).toHaveText(/Run/)
    // 工具栏不与代码块首行重叠（流内行）
    const geo = await fence.evaluate((el) => {
      const bar = el.querySelector('.af-toolbar')!.getBoundingClientRect()
      const orig = el.querySelector('.af-original')!.getBoundingClientRect()
      return { gap: Math.round(orig.top - bar.bottom) }
    })
    expect(geo.gap).toBeGreaterThanOrEqual(0)

    await page.goto('/zh/books/tapl/ch01-getting-started', { waitUntil: 'networkidle' })
    await expect(page.locator('.auto-fence button.af-run').first()).toHaveText(/运行/)

    // 锁定示例：说明区双语 + locale 正确的 Playground 链接
    await page.goto('/books/byte-of-python/ch08-modules', { waitUntil: 'networkidle' })
    let lockNote = page.locator('.af-lock-note').first()
    await expect(lockNote).toContainText('cannot run inline')
    await expect(lockNote.locator('a')).toHaveAttribute('href', '/playground')
    await page.goto('/zh/books/byte-of-python/ch08-modules', { waitUntil: 'networkidle' })
    lockNote = page.locator('.af-lock-note').first()
    await expect(lockNote).toContainText('此示例依赖多文件模块')
    await expect(lockNote.locator('a')).toHaveAttribute('href', '/zh/playground')
  })

  test('copy button copies raw source and follows locale title', async ({ page, context }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write'])
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'networkidle' })
    const codeBlock = page.locator('.auto-fence .af-original [class*="language-"]').first()
    await expect(codeBlock.locator('button.copy')).toHaveAttribute('title', 'Copy Code')
    await codeBlock.locator('button.copy').click()
    const clip = await page.evaluate(() => navigator.clipboard.readText())
    const source = await codeBlock.locator('pre code').innerText()
    // innerText 带行尾空白（shiki span 布局），按行归一后比较原始源码
    const norm = (s: string) => s.split('\n').map((l) => l.trimEnd()).join('\n').trim()
    expect(norm(clip)).toBe(norm(source))

    await page.goto('/zh/books/tapl/ch01-getting-started', { waitUntil: 'networkidle' })
    await expect(
      page.locator('.auto-fence .af-original [class*="language-"]').first().locator('button.copy'),
    ).toHaveAttribute('title', '复制代码')
  })

  test('aria wiring is unique and keyboard operable', async ({ page }) => {
    await page.goto('/books/tapl/ch01-getting-started', { waitUntil: 'networkidle' })
    const buttons = page.locator('.auto-fence button.af-run[aria-controls]')
    const count = await buttons.count()
    expect(count).toBeGreaterThan(0)
    // 唯一 aria-controls 关联 ID
    const ids = await buttons.evaluateAll((els) => els.map((e) => e.getAttribute('aria-controls')))
    expect(new Set(ids).size).toBe(count)
    for (const id of ids) {
      expect(id, 'panel element exists').toBeTruthy()
    }
    // 键盘开合 + aria-expanded 同步
    const btn = buttons.first()
    await btn.focus()
    await page.keyboard.press('Enter')
    await expect(btn).toHaveAttribute('aria-expanded', 'true')
    await expect(page.locator(`#${ids[0]}`)).toBeVisible()
    await page.keyboard.press('Enter')
    await expect(btn).toHaveAttribute('aria-expanded', 'false')
    // 焦点在开关上（未丢进不可见元素）
    await expect(btn).toBeFocused()
  })

  test('no new console errors on reading pages', async ({ page }) => {
    for (const p of ['/docs/', '/books/', '/docs/features/actor-concurrency', '/books/tapl/ch01-getting-started']) {
      const errors = trackConsole(page)
      await page.goto(p, { waitUntil: 'networkidle' })
      expect(errors, p).toEqual([])
    }
  })
})
