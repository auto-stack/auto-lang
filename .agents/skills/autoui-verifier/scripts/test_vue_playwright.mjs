/**
 * AutoUI Vue Mode Playwright Test Runner.
 * Drives a Vue/Vite frontend and captures high-fidelity dark-mode screenshots.
 * 
 * Usage:
 *   1. Single screenshot:
 *      node test_vue_playwright.mjs <url> <outPath>
 *   2. Interactive steps via JSON:
 *      node test_vue_playwright.mjs <url> --actions '[{"action":"screenshot","path":"shot1.png"},{"action":"click","selector":"button"},{"action":"screenshot","path":"shot2.png"}]'
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
  const args = process.argv.slice(2);
  let url = 'http://localhost:3000';
  let outPath = './vue_initial.png';
  let actions = null;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--actions' && i + 1 < args.length) {
      actions = JSON.parse(args[i + 1]);
      i++;
    } else if (args[i] === '--actions-file' && i + 1 < args.length) {
      actions = JSON.parse(fs.readFileSync(args[i + 1], 'utf-8'));
      i++;
    } else if (args[i].startsWith('http://') || args[i].startsWith('https://')) {
      url = args[i];
    } else if (!args[i].startsWith('--')) {
      outPath = args[i];
    }
  }

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

  if (actions && Array.isArray(actions)) {
    console.log(`[*] Executing ${actions.length} interaction actions...`);
    for (const step of actions) {
      const { action } = step;
      if (action === 'screenshot') {
        const p = path.resolve(step.path);
        fs.mkdirSync(path.dirname(p), { recursive: true });
        await page.screenshot({ path: p, fullPage: true });
        console.log(`[+] Captured screenshot: ${step.path}`);
      } else if (action === 'click') {
        await page.click(step.selector);
        console.log(`[+] Clicked: ${step.selector}`);
      } else if (action === 'fill' || action === 'type') {
        await page.fill(step.selector, step.text || '');
        console.log(`[+] Filled ${step.selector} with '${step.text || ''}'`);
      } else if (action === 'focus') {
        await page.focus(step.selector);
        console.log(`[+] Focused: ${step.selector}`);
      } else if (action === 'press') {
        await page.keyboard.press(step.key);
        console.log(`[+] Pressed key: ${step.key}`);
      } else if (action === 'wait') {
        await page.waitForTimeout(step.ms || 500);
      } else if (action === 'drag') {
        // Plan 481: raw mouse drag in page coordinates (text-selection
        // gestures). {action:'drag', from:[x,y], to:[x,y], steps:N}
        const [fx, fy] = step.from;
        const [tx, ty] = step.to;
        await page.mouse.move(fx, fy);
        await page.mouse.down();
        const steps = step.steps || 8;
        for (let i = 1; i <= steps; i++) {
          await page.mouse.move(fx + (tx - fx) * i / steps, fy + (ty - fy) * i / steps);
        }
        await page.mouse.up();
        console.log(`[+] Dragged (${fx},${fy}) -> (${tx},${ty})`);
      } else if (action === 'selectionText') {
        const sel = await page.evaluate(() => window.getSelection().toString());
        console.log(`[SELECTION] ${JSON.stringify(sel)}`);
      } else if (action === 'dblclick') {
        await page.dblclick(step.selector, { position: step.position });
        console.log(`[+] Double-clicked: ${step.selector}`);
      }
    }
  } else {
    const dir = path.dirname(path.resolve(outPath));
    fs.mkdirSync(dir, { recursive: true });
    await page.screenshot({ path: outPath, fullPage: true });
    console.log(`[+] Saved Vue screenshot to ${outPath}`);
  }

  await browser.close();
}

main().catch(err => {
  console.error('[-] Error in Playwright runner:', err);
  process.exit(1);
});
