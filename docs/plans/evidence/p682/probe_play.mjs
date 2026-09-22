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
  if (!chromium) throw new Error('playwright not found');
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
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.goto('http://localhost:4024/', { waitUntil: 'domcontentloaded', timeout: 10000 });
  await page.waitForTimeout(1500);
  console.log('TITLE=' + (await page.title()));
  console.log('GALLERY_NODES=' + (await page.locator('text=AutoUI Gallery').count()));
  const playBtns = page.locator('button', { hasText: 'Play' });
  console.log('PLAY_BTNS=' + (await playBtns.count()));
  if (await playBtns.count()) {
    await playBtns.first().click();
    await page.waitForTimeout(2800);
    const pauseCount = await page.locator('button', { hasText: 'Pause' }).count();
    const body = await page.locator('body').innerText();
    console.log('AFTER_PAUSE_BTNS=' + pauseCount);
    console.log('HAS_T_LABELS=' + (body.includes('t1') || body.includes('t2') || body.includes('t3')));
    console.log('HAS_JAN=' + body.includes('Jan'));
    await page.screenshot({ path: 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/probe_024_after.png', fullPage: true });
  }
  await browser.close();
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
