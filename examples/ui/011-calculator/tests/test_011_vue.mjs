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
  page.on('console', msg => console.log('[BROWSER CONSOLE]', msg.text()));
  page.on('pageerror', err => console.log('[BROWSER ERROR]', err.message));
  console.log(`[*] Navigating to ${url}...`);
  await page.goto(url, { waitUntil: 'networkidle' });

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };

  // 1. Initial Dark
  console.log('[*] Capturing 011_vue_dark_initial...');
  await page.screenshot({ path: path.join(outDir, '011_vue_dark_initial.png'), fullPage: true });

  // Plan 504: in-app title bar / Settings removed (moved to pac.at + os-config).
  const settingsBtn = await page.$('button:has-text("Settings")');
  check('no in-app Settings header (Plan 504)', settingsBtn === null);

  // Helper to click keypad button by exact text
  async function clickKey(label) {
    const btn = page.getByRole('button', { name: label, exact: true });
    await btn.click();
    await page.waitForTimeout(50);
  }

  // 2. Decimal Evaluation: 3.5 + 1 = 4.5
  console.log('[*] Evaluating 3.5 + 1 = ...');
  await clickKey('C');
  await clickKey('3');
  await clickKey('.');
  await clickKey('5');
  await clickKey('+');
  await clickKey('1');
  await clickKey('=');
  await page.waitForTimeout(300);
  console.log('[*] Capturing 011_vue_calc_eval...');
  await page.screenshot({ path: path.join(outDir, '011_vue_calc_eval.png'), fullPage: true });

  // 3. Scientific Mode: 2 * ( 3 + 4 ) = 14
  console.log('[*] Switching to Scientific mode and evaluating 2 * ( 3 + 4 ) = ...');
  await clickKey('Scientific');
  await page.waitForTimeout(200);
  await clickKey('C');
  await clickKey('2');
  await clickKey('*');
  await clickKey('(');
  await clickKey('3');
  await clickKey('+');
  await clickKey('4');
  await clickKey(')');
  await clickKey('=');
  await page.waitForTimeout(300);
  console.log('[*] Capturing 011_vue_scientific_mode...');
  await page.screenshot({ path: path.join(outDir, '011_vue_scientific_mode.png'), fullPage: true });

  // 4. Plan 504: math.pow static dispatch — 2 ^ 10 = 1024 (was in-app loop).
  console.log('[*] Evaluating 2 ^ 10 = 1024 (math.pow static dispatch)...');
  await clickKey('C');
  await clickKey('2');
  await clickKey('^');
  await clickKey('1');
  await clickKey('0');
  await clickKey('=');
  await page.waitForTimeout(300);
  const powResult = await page.getByText('1024', { exact: true }).count();
  check('2 ^ 10 = 1024 (math.pow)', powResult > 0);
  await page.screenshot({ path: path.join(outDir, '011_vue_pow.png'), fullPage: true });

  // Switch back to Basic
  await clickKey('Basic');
  await clickKey('C');
  await page.waitForTimeout(200);

  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    await browser.close();
    process.exit(1);
  }
  console.log('[+] All Vue screenshots captured successfully!');
  await browser.close();
}

main().catch(err => {
  console.error('[-] Error in Vue Playwright runner:', err);
  process.exit(1);
});
