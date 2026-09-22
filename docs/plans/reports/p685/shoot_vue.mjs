/**
 * PLAN-685 T-06: bps-gallery VUE-arm walkthrough grid (3 bp x 3 section).
 * Usage: node shoot_vue.mjs <url> <out_dir>
 * Mirrors docs/plans/reports/p685/shoot_grid.py's VM grid; screenshots named
 * vue_*.png. Clicks resolve by button text (sidebar card = bp id prefix,
 * tabs = exact Spec/Reference/Gotchas).
 */
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of possiblePaths) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      return (await import(target)).chromium;
    } catch (_) {}
  }
  throw new Error('playwright not resolvable');
}

const [, , url, outDir] = process.argv;
const bps = [
  ['dashboard/overview', 'overview'],
  ['data-display/row-list', 'row-list'],
  ['data-display/master-detail', 'master-detail'],
];
const sections = ['Spec', 'Reference', 'Gotchas'];

fs.mkdirSync(outDir, { recursive: true });
const chromium = await resolvePlaywright();
const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: 1280, height: 800 },
  colorScheme: 'dark',
});
await page.goto(url, { waitUntil: 'networkidle', timeout: 60000 });

async function shot(name) {
  const p = path.join(outDir, `${name}.png`);
  await page.screenshot({ path: p, fullPage: false });
  const size = fs.statSync(p).size;
  console.log(`  shot ${name}: ${size}B`);
}

async function clickButton(label, { exact = false } = {}) {
  const loc = exact
    ? page.getByRole('button', { name: label, exact: true }).first()
    : page.getByRole('button', { name: label }).first();
  await loc.waitFor({ state: 'visible', timeout: 10000 });
  await loc.click();
}

await page.waitForTimeout(2500); // hydration settle
await shot('vue_00_initial');

for (const [bpId, short] of bps) {
  await clickButton(bpId);
  await page.waitForTimeout(1200);
  console.log(`== ${bpId} ==`);
  for (const s of sections) {
    await clickButton(s, { exact: true });
    await page.waitForTimeout(900);
    await shot(`vue_${short}_${s.toLowerCase()}`);
  }
}

await browser.close();
console.log('vue grid done');
