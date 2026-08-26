/**
 * AutoUI Vue Mode Playwright Test Runner.
 * Drives a Vue/Vite frontend and captures high-fidelity dark-mode screenshots.
 */

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
  const url = process.argv[2] || 'http://localhost:5173';
  const outPath = process.argv[3] || './vue_initial.png';

  const dir = path.dirname(path.resolve(outPath));
  fs.mkdirSync(dir, { recursive: true });

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
  page.on('console', msg => console.log('[PAGE CONSOLE]', msg.type(), msg.text()));
  page.on('pageerror', err => console.error('[PAGE ERROR]', err));
  console.log(`[*] Navigating to ${url}...`);
  await page.goto(url, { waitUntil: 'networkidle' });

  await page.screenshot({ path: outPath });
  console.log(`[+] Saved Vue screenshot to ${outPath}`);

  await browser.close();
}

main().catch(err => {
  console.error('[-] Error in Playwright runner:', err);
  process.exit(1);
});
