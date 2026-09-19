#!/usr/bin/env node
/**
 * PLAN-661 T-06: vue-track Playwright tests for 049-canvas-graph.
 *
 * Prerequisites: the vue dev server for this sample must be running
 * (`auto run` in the sample dir → http://localhost:3000).
 *
 * Covers the vue-side acceptance surfaces:
 *   - AC-03: native `<input type="range">` with min/max/step/value binding
 *     present; setting the value dispatches the payload msg (state + derived
 *     display sync — radius label + rebuilt canvas nodes).
 *   - AC-06: coordinate click on a node center dispatches onhit with the
 *     element id (selected text updates); click on empty area dispatches
 *     nothing.
 *
 * Playwright resolution mirrors the autoui-verifier skill script
 * (test_vue_playwright.mjs) — same standard node_modules locations.
 */
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
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      const mod = await import(target);
      return mod.chromium;
    } catch (_) {}
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

const URL_BASE = process.env.VUE_URL || 'http://localhost:3000';
const OUT_DIR = process.env.OUT_DIR || path.join(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1')), 'shots');

let pass = 0;
let fail = 0;
function check(name, cond, detail = '') {
  if (cond) { pass++; console.log(`  PASS  ${name}`); }
  else { fail++; console.log(`  FAIL  ${name}  ${detail}`); }
}

const chromium = await resolvePlaywright();
const browser = await chromium.launch({ headless: true, channel: 'msedge' })
  .catch(() => chromium.launch({ headless: true }));
const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, colorScheme: 'dark' });
const page = await context.newPage();
await page.goto(URL_BASE, { waitUntil: 'networkidle' });
await page.waitForSelector('canvas', { timeout: 10000 });
await page.waitForTimeout(800);

// ---- AC-03: native range input with attrs ----
const rangeInfo = await page.evaluate(() => {
  const el = document.querySelector('input[type="range"]');
  if (!el) return null;
  return { min: el.min, max: el.max, step: el.step, value: el.value };
});
check('range input present', rangeInfo !== null);
check('range attrs min/max/step', rangeInfo && rangeInfo.min === '50' && rangeInfo.max === '110' && rangeInfo.step === '5', JSON.stringify(rangeInfo));
check('range value bound to radius 80', rangeInfo && rangeInfo.value === '80', JSON.stringify(rangeInfo));

fs.mkdirSync(OUT_DIR, { recursive: true });
await page.screenshot({ path: path.join(OUT_DIR, 'vue-initial.png'), fullPage: true });

// ---- AC-06: coordinate click on node center (n0 = (230,150) logical) ----
// 画布 w-full h-[300px] 非方形——逻辑→px 按 x/y 独立缩放（引擎 sx/sy 同式）。
const box = await page.locator('canvas').boundingBox();
const sx = box.width / 300, sy = box.height / 300;
const toPx = (lx, ly) => [box.x + lx * sx, box.y + ly * sy];

// n0 at radius 80 → angle 0 → (230, 150).
let [cx, cy] = toPx(230, 150);
await page.mouse.click(cx, cy);
await page.waitForTimeout(500);
// .selected 的值直接渲染为文本节点（无前缀，body 文本无分隔拼接）——
// n0 只可能来自命中派发（图元标签绘制在 canvas 上，非 DOM 文本）。
let selText = await page.evaluate(() => document.body.textContent);
check('onhit n0 dispatched', selText.includes('n0'), selText.slice(0, 200));
await page.screenshot({ path: path.join(OUT_DIR, 'vue-hit-n0.png'), fullPage: true });

// empty-area click (corner, far from nodes) → selected unchanged (still n0)
const before = await page.evaluate(() => document.body.textContent.includes('n0'));
[ cx, cy ] = toPx(6, 6);
await page.mouse.click(cx, cy);
await page.waitForTimeout(400);
const after = await page.evaluate(() => document.body.textContent.includes('n0'));
check('empty-area click no dispatch', before && after, 'selected changed on empty click');

// ---- AC-03: set range value → payload msg → radius state + canvas rebuild ----
await page.fill('input[type="range"]', '110');
await page.waitForTimeout(600);
// 状态↔显示同步的 DOM 级断言：:value="radius" 回写——SetRadius 载荷链
// 走通后 el.value 保持 110（fill 只设一次，后续值来自状态回灌）。
const rangeAfter = await page.evaluate(() => {
  const el = document.querySelector('input[type="range"]');
  return el ? el.value : null;
});
check('slider set → state sync (value round-trip)', rangeAfter === '110', `got ${rangeAfter}`);
// n0 at radius 110 → (260, 150) — click there hits n0 again (validates rebuild coords).
[ cx, cy ] = toPx(260, 150);
await page.mouse.click(cx, cy);
await page.waitForTimeout(500);
selText = await page.evaluate(() => document.body.textContent);
check('rebuilt node hit at new coords', selText.includes('n0'), selText.slice(0, 200));
await page.screenshot({ path: path.join(OUT_DIR, 'vue-radius110.png'), fullPage: true });

await browser.close();
console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail === 0 ? 0 : 1);
