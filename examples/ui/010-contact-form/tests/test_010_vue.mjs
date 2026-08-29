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
  const outDir = path.resolve('examples/ui/010-contact-form/src/front/tests/screenshots');
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

  await page.waitForTimeout(300);
  console.log('[*] Capturing 010_vue_typed...');
  await page.screenshot({ path: path.join(outDir, '010_vue_typed.png'), fullPage: true });

  // 3. Click Send Message
  console.log('[*] Clicking Send Message button...');
  await page.click('button:has-text("Send Message")');
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_submitted...');
  await page.screenshot({ path: path.join(outDir, '010_vue_submitted.png'), fullPage: true });

  // 4. Open Settings
  console.log('[*] Clicking Settings button...');
  await page.click('button:has-text("Settings")');
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_settings_open...');
  await page.screenshot({ path: path.join(outDir, '010_vue_settings_open.png'), fullPage: true });

  // 5. Click Coral Accent (2nd accent circle)
  console.log('[*] Clicking Coral accent button...');
  const accentButtons = await page.$$('button.bg-rose-500, button[class*="rose"]');
  if (accentButtons.length > 0) {
    await accentButtons[0].click();
  } else {
    const buttons = await page.$$('button');
    if (buttons.length >= 5) {
      await buttons[4].click();
    }
  }
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_coral_accent...');
  await page.screenshot({ path: path.join(outDir, '010_vue_coral_accent.png'), fullPage: true });

  // 6. Click Light mode
  console.log('[*] Clicking Light mode button...');
  await page.click('button:has-text("Light")');
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_light_mode...');
  await page.screenshot({ path: path.join(outDir, '010_vue_light_mode.png'), fullPage: true });

  // 7. Back to Dark mode
  console.log('[*] Clicking Dark mode button...');
  await page.click('button:has-text("Dark")');
  await page.waitForTimeout(400);
  console.log('[*] Capturing 010_vue_back_to_dark...');
  await page.screenshot({ path: path.join(outDir, '010_vue_back_to_dark.png'), fullPage: true });

  console.log('[+] All Vue screenshots captured successfully!');
  await browser.close();
}

main().catch(err => {
  console.error('[-] Error in Vue Playwright runner:', err);
  process.exit(1);
});
