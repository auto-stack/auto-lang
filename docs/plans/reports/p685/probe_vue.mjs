/** Probe: what does the vue app render? console errors + body text. */
import { pathToFileURL } from 'url';
import path from 'path';

async function resolvePlaywright() {
  const paths = [
    'd:/autostack/auto-lang/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of paths) {
    try {
      const t = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      return (await import(t)).chromium;
    } catch (_) {}
  }
  throw new Error('no playwright');
}

const chromium = await resolvePlaywright();
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on('console', (m) => {
  if (m.type() === 'error' || m.type() === 'warning') errors.push(`${m.type()}: ${m.text().slice(0, 300)}`);
});
page.on('pageerror', (e) => errors.push(`pageerror: ${String(e).slice(0, 500)}`));
await page.goto('http://localhost:3000/', { waitUntil: 'networkidle', timeout: 60000 });
await page.waitForTimeout(4000);
const text = await page.evaluate(() => document.body.innerText.slice(0, 800));
const buttons = await page.evaluate(() => Array.from(document.querySelectorAll('button')).length);
console.log('=== BODY TEXT ===');
console.log(text);
console.log('=== button count:', buttons, '===');
console.log('=== CONSOLE ===');
console.log(errors.slice(0, 12).join('\n') || '(none)');
await browser.close();
