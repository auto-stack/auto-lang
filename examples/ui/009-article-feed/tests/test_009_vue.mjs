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
  const url = process.argv[2] || 'http://localhost:5199';
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

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };

  // 1. Initial Dark
  console.log('[*] Capturing 009_vue_dark_initial...');
  await page.screenshot({ path: path.join(outDir, '009_vue_dark_initial.png'), fullPage: true });

  // Plan 506: in-app title bar / Settings removed (ExampleHeader retired —
  // title/theme/accent moved to pac.at + os-config, host chrome carries them).
  const settingsBtn = await page.$('button:has-text("Settings")');
  check('no in-app Settings header (Plan 506)', settingsBtn === null);

  // Content sanity: article cards still render.
  const blogHeading = await page.$('text=Blog Posts');
  check('content marker present: Blog Posts', blogHeading !== null);
  const readMoreBtn = await page.$('button:has-text("Read more")');
  check('content marker present: Read more', readMoreBtn !== null);

  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    await browser.close();
    process.exit(1);
  }
  console.log('[+] All Vue assertions passed!');
  await browser.close();
}

main().catch(err => {
  console.error('[-] Error in Vue Playwright runner:', err);
  process.exit(1);
});
