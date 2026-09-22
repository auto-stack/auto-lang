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
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, colorScheme: 'dark' });
  const shot = (n) =>
    page.screenshot({ path: 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/' + n, fullPage: true });

  await page.goto('http://localhost:3052/', { waitUntil: 'domcontentloaded', timeout: 15000 });
  await page.waitForTimeout(2000);
  console.log('TITLE=' + (await page.title()));

  // Search and open 024-charts
  const input = page.locator('input').first();
  await input.fill('024');
  await page.waitForTimeout(800);
  await page.locator('text=024-charts').first().click();
  await page.waitForTimeout(3000);
  await shot('gallery_024_stream_before.png');

  // Click Play inside the embed (not gallery chrome)
  const playBtn = page.locator('button', { hasText: 'Play' });
  const playCount = await playBtn.count();
  console.log('PLAY_BTNS=' + playCount);
  if (playCount) {
    await playBtn.first().click();
    await page.waitForTimeout(3000);
    const pauseCount = await page.locator('button', { hasText: 'Pause' }).count();
    const body = await page.locator('body').innerText();
    console.log('AFTER_PAUSE_BTNS=' + pauseCount);
    console.log('HAS_T_LABELS=' + /\\bt[0-9]+\\b/.test(body) || body.includes('t1') || body.includes('t2'));
    console.log('HAS_JAN=' + body.includes('Jan'));
    console.log('HAS_FEB=' + body.includes('Feb'));
    await shot('gallery_024_stream_after.png');
  } else {
    console.log('NO_PLAY_BUTTON');
    await shot('gallery_024_stream_noplay.png');
  }
  await browser.close();
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
