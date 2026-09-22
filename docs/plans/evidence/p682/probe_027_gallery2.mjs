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
  const browser = await chromium.launch({ headless: true, channel: 'msedge' });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, colorScheme: 'dark' });
  const dir = 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/';
  const log = (...a) => console.log('[probe2]', ...a);

  await page.goto('http://localhost:3049/', { waitUntil: 'domcontentloaded', timeout: 20000 });
  await page.waitForTimeout(2500);
  await page.locator('input').first().fill('027');
  await page.waitForTimeout(800);
  await page.locator('text=/027.*文件管理器/i').first().click({ timeout: 8000 });
  await page.waitForTimeout(6000);

  // 排序：点「名称」表头两次（asc→desc），验证 vue 轨共享 sort_files 生效
  const firstNames = async () => {
    const rows = await page.locator('tr, [class*="row"], .mouse-area').allInnerTexts().catch(() => []);
    if (rows.length) return rows.slice(0, 5).map(r => r.split('\n')[0].trim()).filter(Boolean);
    return [];
  };
  const before = await firstNames();
  await page.locator('text=名称').first().click();
  await page.waitForTimeout(1200);
  const afterAsc = await firstNames();
  await page.locator('text=名称').first().click();
  await page.waitForTimeout(1200);
  const afterDesc = await firstNames();
  log('first5 before :', JSON.stringify(before));
  log('first5 asc    :', JSON.stringify(afterAsc));
  log('first5 desc   :', JSON.stringify(afterDesc));

  // 平板档 768×1024
  await page.locator('button', { hasText: '平板 768×1024' }).first().click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: dir + 'gallery_027_tablet.png', fullPage: false });
  log('tablet screenshot saved');

  // 演示后备对照：断网后端不可达场景由 standalone 验证，略
  await browser.close();
})().catch(e => { console.error('[probe2] fatal', e); process.exit(1); });
