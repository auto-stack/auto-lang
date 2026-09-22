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
  const page = await browser.newPage({ viewport: { width: 1280, height: 820 }, colorScheme: 'dark' });
  const dir = 'D:/autostack/.wt/lang-684/auto-lang/docs/plans/evidence/p682/';
  await page.goto('http://localhost:4027/', { waitUntil: 'domcontentloaded', timeout: 20000 });
  await page.waitForTimeout(5000); // boot tick 窗 + fs_home 骨架往返
  const body = await page.locator('body').innerText();
  const cnt = body.match(/(\d+)\s*个项目/);
  console.log('[standalone-vue] item-count:', cnt && cnt[0]);
  console.log('[standalone-vue] demo-entries:', ['Documents', 'Pictures', 'Downloads', 'readme.md', 'report.pdf', 'photo.png'].filter(n => body.includes(n)).join(','));
  console.log('[standalone-vue] demo-home-crumb:', body.includes('demo://home') || body.includes('主目录'));
  await page.screenshot({ path: dir + 'vue_standalone_demo_fallback.png' });
  await browser.close();
})().catch(e => { console.error('fatal', e); process.exit(1); });
