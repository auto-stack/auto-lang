const paths = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
  'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
  'playwright'
];
(async () => {
  let chromium;
  for (const p of paths) {
    try {
      const m = await import(p.includes(':') ? 'file:///' + p.replace(/\\/g, '/') : p);
      chromium = m.chromium;
      break;
    } catch (_) {}
  }
  const browser = await chromium.launch({ headless: true, channel: 'msedge' });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, colorScheme: 'dark' });
  const names = async () =>
    (await page.locator('[class*="h-11"]').allInnerTexts()).map(t => t.split('\n')[0].trim()).filter(Boolean);
  await page.goto('http://localhost:3049/', { waitUntil: 'domcontentloaded', timeout: 20000 });
  await page.waitForTimeout(2500);
  await page.locator('input').first().fill('027');
  await page.waitForTimeout(800);
  await page.locator('text=/027.*文件管理器/i').first().click({ timeout: 8000 });
  await page.waitForTimeout(6000);
  const n0 = await names();
  await page.locator('text=名称').first().click();
  await page.waitForTimeout(1500);
  const n1 = await names();
  await page.locator('text=名称').first().click();
  await page.waitForTimeout(1500);
  const n2 = await names();
  console.log('[sort] default  :', JSON.stringify(n0.slice(0, 6)));
  console.log('[sort] name-asc :', JSON.stringify(n1.slice(0, 6)));
  console.log('[sort] name-desc:', JSON.stringify(n2.slice(0, 6)));
  await browser.close();
})().catch(e => { console.error('fatal', e); process.exit(1); });
