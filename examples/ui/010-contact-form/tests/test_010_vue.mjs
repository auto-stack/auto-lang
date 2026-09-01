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

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };

  // Plan 506: in-app title bar / Settings removed (ExampleHeader retired —
  // title/theme/accent moved to pac.at + os-config, host chrome carries them).
  const settingsBtn0 = await page.$('button:has-text("Settings")');
  check('no in-app Settings header (Plan 506)', settingsBtn0 === null);

  // 1. Initial Dark
  console.log('[*] Capturing 010_vue_dark_initial...');
  await page.screenshot({ path: path.join(outDir, '010_vue_dark_initial.png'), fullPage: true });

  // 2. Type into Name, Email, Message
  console.log('[*] Typing into contact form...');
  const nameInput = await page.$('input[placeholder*="name" i]');
  if (nameInput) await nameInput.fill('Alice Smith');

  const emailInput = await page.$('input[placeholder*="example" i], input[placeholder*="email" i]');
  if (emailInput) await emailInput.fill('alice@example.com');

  const msgInput = await page.$('textarea');
  if (msgInput) await msgInput.fill('I would like to inquire about enterprise support options.');

  check('3 form inputs present', nameInput !== null && emailInput !== null && msgInput !== null);

  await page.waitForTimeout(300);
  console.log('[*] Capturing 010_vue_typed...');
  await page.screenshot({ path: path.join(outDir, '010_vue_typed.png'), fullPage: true });

  // 3. Click Send Message
  console.log('[*] Clicking Send Message button...');
  await page.click('button:has-text("Send Message")');
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_submitted...');
  await page.screenshot({ path: path.join(outDir, '010_vue_submitted.png'), fullPage: true });

  const thankYou = await page.$('text=Thank you! Your message has been sent.');
  check('submitted confirmation visible', thankYou !== null);

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
