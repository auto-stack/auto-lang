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
    try {
      browser = await chromium.launch({ headless: true, channel: 'chrome' });
    } catch (_) {
      browser = await chromium.launch({ headless: true });
    }
  }
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, colorScheme: 'dark' });
  const dir = 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/';
  const btnTexts = async () => {
    const texts = await page.locator('button').allInnerTexts();
    return texts.map((t) => t.replace(/\\s+/g, ' ').trim()).filter((t) => /Play|Pause|Reset/.test(t));
  };
  const axisTexts = async () => {
    const texts = await page.locator('svg text').allInnerTexts();
    return texts.map((t) => t.trim()).filter(Boolean);
  };

  await page.goto('http://localhost:3052/', { waitUntil: 'domcontentloaded', timeout: 15000 });
  await page.waitForTimeout(2500);
  const input = page.locator('input').first();
  await input.fill('024');
  await page.waitForTimeout(1000);
  await page.locator('text=024-charts').first().click();
  await page.waitForTimeout(3500);

  console.log('BEFORE_BTNS=' + JSON.stringify(await btnTexts()));
  console.log('BEFORE_AXIS=' + JSON.stringify(await axisTexts()));
  await page.screenshot({ path: dir + 'g2_before.png', fullPage: true });

  const play = page.locator('button', { hasText: 'Play' }).first();
  await play.click();
  await page.waitForTimeout(400);
  console.log('AFTER_CLICK_BTNS=' + JSON.stringify(await btnTexts()));

  await page.waitForTimeout(4500);
  console.log('LATER_BTNS=' + JSON.stringify(await btnTexts()));
  console.log('LATER_AXIS=' + JSON.stringify(await axisTexts()));
  await page.screenshot({ path: dir + 'g2_after.png', fullPage: true });

  await browser.close();
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
