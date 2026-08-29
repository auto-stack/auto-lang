import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'd:/autostack/auto-lang/examples/ui/022-kanban/tests/node_modules/playwright/index.mjs',
    'playwright'
  ];
  for (const p of possiblePaths) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      const mod = await import(target);
      return mod.chromium;
    } catch (_) {}
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

async function main() {
  const url = process.argv[2] || 'http://localhost:5198';
  const outDir = path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([a-zA-Z]:)/, '$1')), '../src/front/tests/screenshots');
  fs.mkdirSync(outDir, { recursive: true });

  const chromium = await resolvePlaywright();
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

  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    colorScheme: 'dark'
  });

  const page = await context.newPage();
  console.log(`[*] Navigating to ${url}...`);
  await page.goto(url, { waitUntil: 'networkidle' });

  // 1. Initial Dark
  console.log('[*] Capturing 008_vue_dark_initial...');
  await page.screenshot({ path: path.join(outDir, '008_vue_dark_initial.png'), fullPage: true });

  // 2. Open Settings
  console.log('[*] Clicking Settings button...');
  const settingsBtn = await page.$('button:has-text("Settings")');
  if (settingsBtn) {
    await settingsBtn.click();
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(outDir, '008_vue_settings_open.png'), fullPage: true });

    // 3. Coral Accent
    console.log('[*] Clicking Coral accent button...');
    const accentBtns = await page.$$('.z-50 button[class*="rounded-full"], button.rounded-full');
    if (accentBtns.length >= 2) {
      await accentBtns[1].click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: path.join(outDir, '008_vue_coral_accent.png'), fullPage: true });
    }

    // 4. Light Mode
    console.log('[*] Clicking Light mode button...');
    const lightBtn = await page.$('button:has-text("Light")');
    if (lightBtn) {
      await lightBtn.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: path.join(outDir, '008_vue_light_mode.png'), fullPage: true });
    }

    // 5. Dark Mode
    console.log('[*] Clicking Dark mode button...');
    const darkBtn = await page.$('button:has-text("Dark")');
    if (darkBtn) {
      await darkBtn.click();
      await page.waitForTimeout(500);
      await page.screenshot({ path: path.join(outDir, '008_vue_back_to_dark.png'), fullPage: true });
    }
  }

  console.log('[+] All Vue screenshots captured successfully!');
  await browser.close();
}

main().catch(err => {
  console.error('[-] Test failed:', err);
  process.exit(1);
});
