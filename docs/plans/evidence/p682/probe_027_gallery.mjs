const paths = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
  'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
  'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
  'playwright'
];
(async () => {
  let chromium;
  for (const p of paths) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? 'file:///' + p.replace(/\\/g, '/') : p;
      const m = await import(target);
      chromium = m.chromium;
      break;
    } catch (_) {}
  }
  let browser;
  try {
    browser = await chromium.launch({ headless: true, channel: 'msedge' });
  } catch (_) {
    browser = await chromium.launch({ headless: true });
  }
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, colorScheme: 'dark' });
  const dir = 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/';
  const log = (...a) => console.log('[probe]', ...a);

  await page.goto('http://localhost:3049/', { waitUntil: 'domcontentloaded', timeout: 20000 });
  await page.waitForTimeout(2500);

  // 侧栏搜索 027 → 点击条目
  const search = page.locator('input').first();
  await search.fill('027', { timeout: 8000 }).catch(e => log('search fill fail', e.message));
  await page.waitForTimeout(800);
  const hit = page.locator('text=/027.*文件管理器|027-file-manager/i').first();
  await hit.click({ timeout: 8000 }).catch(async e => {
    log('click by text fail, dumping buttons:', e.message);
    const texts = await page.locator('button').allInnerTexts();
    log(JSON.stringify(texts.slice(0, 60)));
  });
  // lazy session 首请求装载 + 首列目录
  await page.waitForTimeout(6000);

  // 诊断：全部可见文本行（条目区）+ 按钮面
  const bodyText = await page.locator('body').innerText();
  log('BODY-SNIPPET>>>\n' + bodyText.split('\n').filter(l => l.trim()).slice(0, 80).join('\n') + '\n<<<');
  await page.screenshot({ path: dir + 'gallery_027_embed.png' });

  // 条目计数行（"N 个项目"）与地址（面包屑/addr）
  const cnt = bodyText.match(/(\d+\+?)\s*个项目/);
  log('item-count:', cnt && cnt[0]);
  const hasDemo = bodyText.includes('demo://');
  log('has-demo-path:', hasDemo);

  // 双击进入 Documents（真盘目录）
  const doc = page.locator('text=Documents').first();
  if (await doc.count()) {
    await doc.dblclick({ timeout: 5000 }).catch(e => log('dblclick Documents fail', e.message));
    await page.waitForTimeout(2500);
    await page.screenshot({ path: dir + 'gallery_027_nav.png' });
    const t2 = await page.locator('body').innerText();
    const c2 = t2.match(/(\d+\+?)\s*个项目/);
    log('after-nav count:', c2 && c2[0]);
    log('after-nav has-Documents-crumb:', t2.includes('Documents'));
    // 上行一次（GoUp）→ 回主目录
    const up = page.locator('button:has(svg)').filter({ hasText: '' }).first();
    log('(skip precise up-click)');
  } else {
    log('no Documents row found');
  }

  // 视口档位：切 375 移动档（按钮文本探测）
  const btns = await page.locator('button').allInnerTexts();
  const mob = btns.find(t => /375|移动|mobile/i.test(t));
  if (mob) {
    await page.locator('button', { hasText: mob }).first().click();
    await page.waitForTimeout(1200);
    await page.screenshot({ path: dir + 'gallery_027_mobile.png' });
    log('mobile preset clicked:', mob.trim());
  } else {
    log('mobile preset button not found among:', JSON.stringify(btns.slice(0, 40)));
  }

  await browser.close();
})().catch(e => { console.error('[probe] fatal', e); process.exit(1); });
