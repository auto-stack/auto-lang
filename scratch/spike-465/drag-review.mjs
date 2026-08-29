// Plan 465 review-only: drag/resize verification via Playwright mouse API.
// The standard verifier script (autoui-verifier) supports click/fill/press
// only; drag needs page.mouse. Scratch tooling — not part of the shipped
// skill scripts.
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of possiblePaths) {
    try {
      return (await import(pathToFileURL(p).href)).chromium;
    } catch {}
  }
  throw new Error('playwright not found');
}

const chromium = await resolvePlaywright();
const browser = await chromium.launch({ headless: true, channel: 'msedge' }).catch(() => chromium.launch({ headless: true }));
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.goto('http://localhost:3130', { waitUntil: 'networkidle' });

// Launch counter (smallest app).
await page.click('footer button');
await page.waitForTimeout(300);
await page.click("button:has-text('counter')");
await page.waitForTimeout(900);

const win = page.locator('section.virtual-window');
const before = await win.boundingBox();
console.log('before drag:', JSON.stringify(before));

// Drag the title bar by (+120, +60).
await page.mouse.move(before.x + 100, before.y + 14);
await page.mouse.down();
await page.mouse.move(before.x + 220, before.y + 74, { steps: 12 });
await page.mouse.up();
await page.waitForTimeout(300);
const afterDrag = await win.boundingBox();
console.log('after drag :', JSON.stringify(afterDrag));
const dragOk = Math.abs(afterDrag.x - before.x - 120) < 2 && Math.abs(afterDrag.y - before.y - 60) < 2;
console.log(dragOk ? 'DRAG OK' : 'DRAG FAIL');

// Resize via the south-east handle by (+80, +50).
const sizeBefore = { w: afterDrag.width, h: afterDrag.height };
const handle = page.locator('.wm-resize[data-dir="se"]');
const hb = await handle.boundingBox();
await page.mouse.move(hb.x + hb.width / 2, hb.y + hb.height / 2);
await page.mouse.down();
await page.mouse.move(hb.x + hb.width / 2 + 80, hb.y + hb.height / 2 + 50, { steps: 12 });
await page.mouse.up();
await page.waitForTimeout(300);
const afterResize = await win.boundingBox();
console.log('after size :', JSON.stringify(afterResize));
const resizeOk =
  Math.abs(afterResize.width - sizeBefore.w - 80) < 2 &&
  Math.abs(afterResize.height - sizeBefore.h - 50) < 2;
console.log(resizeOk ? 'RESIZE OK' : 'RESIZE FAIL');

// Grid re-layout keeps state consistent after manual geometry.
await page.click("button[title='layout: grid']");
await page.waitForTimeout(400);
const gridRect = await win.boundingBox();
console.log('after grid :', JSON.stringify(gridRect));
const gridOk = Math.abs(gridRect.x - 0) < 2 && Math.abs(gridRect.y - 0) < 2;
console.log(gridOk ? 'GRID RELAYOUT OK' : 'GRID RELAYOUT FAIL');

await page.screenshot({ path: 'D:/autostack/auto-lang/.worktrees/plan-465-dev/scratch/spike-465/shots/21-review-drag-resize.png', fullPage: true });
await browser.close();
if (dragOk && resizeOk && gridOk) {
  console.log('ALL PASS');
} else {
  process.exit(1);
}
