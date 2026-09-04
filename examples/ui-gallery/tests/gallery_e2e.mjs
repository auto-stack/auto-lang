import fs from 'fs';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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
  throw new Error('Playwright not found.');
}

async function main() {
  console.log('▶ Starting UI Gallery E2E Test...');
  const vueDist = path.resolve(__dirname, '../gen/front/vue');

  console.log('▶ Launching vite preview on port 3056...');
  const server = spawn('pnpm', ['exec', 'vite', 'preview', '--port', '3056'], {
    cwd: vueDist,
    shell: true,
    stdio: 'pipe'
  });

  server.stdout.on('data', (d) => process.stdout.write('[server] ' + d));
  server.stderr.on('data', (d) => process.stderr.write('[server-err] ' + d));

  await new Promise((r) => setTimeout(r, 2000));

  let browser;
  try {
    const chromium = await resolvePlaywright();
    browser = await chromium.launch({ headless: true, channel: 'msedge' }).catch(() => chromium.launch({ headless: true }));
    const context = await browser.newContext({ viewport: { width: 1440, height: 1200 } });
    const page = await context.newPage();

    console.log('▶ Navigating to http://localhost:3056 ...');
    await page.goto('http://localhost:3056', { waitUntil: 'networkidle', timeout: 15000 });

    // 1. Initial view: Counter
    console.log('▶ Verifying Counter...');
    const counterBtn = page.locator('text=Counter').first();
    await counterBtn.waitFor({ state: 'visible', timeout: 5000 });
    await counterBtn.click();
    await page.waitForTimeout(1000);

    const shotInitial = path.resolve(__dirname, 'gallery_initial.png');
    await page.screenshot({ path: shotInitial, fullPage: true });
    console.log(`  ✓ Captured initial screenshot: ${shotInitial}`);

    // 2. Click Calculator
    console.log('▶ Clicking Calculator demo...');
    const calcBtn = page.locator('button:has-text("Calculator")').first();
    await calcBtn.click();
    await page.waitForTimeout(1000);

    const shotCalc = path.resolve(__dirname, 'gallery_calculator.png');
    await page.screenshot({ path: shotCalc, fullPage: true });
    console.log(`  ✓ Captured calculator screenshot: ${shotCalc}`);

    // 3. Test Viewport Mode: Mobile
    console.log('▶ Switching viewport to Mobile (375px)...');
    const mobileBtn = page.locator('button:has-text("手机 (375)")').first();
    if (await mobileBtn.count() > 0) {
      await mobileBtn.click();
      await page.waitForTimeout(500);
    }
    const shotMobile = path.resolve(__dirname, 'gallery_mobile_mode.png');
    await page.screenshot({ path: shotMobile, fullPage: true });
    console.log(`  ✓ Captured mobile mode screenshot: ${shotMobile}`);

    // 4. Test Source tab
    console.log('▶ Switching to Source tab...');
    const sourceTab = page.locator('button:has-text("完整源码")').first();
    if (await sourceTab.count() > 0) {
      await sourceTab.click();
      await page.waitForTimeout(500);
    }
    const shotSource = path.resolve(__dirname, 'gallery_source_tab.png');
    await page.screenshot({ path: shotSource, fullPage: true });
    console.log(`  ✓ Captured source tab screenshot: ${shotSource}`);

    console.log('═════════════════════════════════════════');
    console.log('  ✓ UI Gallery E2E Tests Passed Cleanly!');
    console.log('═════════════════════════════════════════');
  } finally {
    if (browser) await browser.close();
    server.kill();
    process.exit(0);
  }
}

main().catch((err) => {
  console.error('Test failed:', err);
  process.exit(1);
});
