#!/usr/bin/env node
/**
 * 028-launcher vue 端验收（Plan 464 T2）。
 *
 * 键盘流全集 + 网格形态 + recent(storage) 的 Playwright 断言套件：
 *   fill "to" → ↓×2 → Enter 命中 012-stopwatch（计划验收原案）
 *   recent 顶置分组 + localStorage 持久化（reload 后仍在）
 *   Tab → grid 形态；网格方向键 + Enter；Esc 逐层退出
 *
 * 用法：
 *   先 `auto run`（vue dev server，front_port 4028），然后
 *   node tests/vue_verify.mjs [baseURL]      # 默认 http://localhost:4028
 *   node tests/vue_verify.mjs --shots <dir>  # 附带截图输出目录
 *
 * Playwright 解析复用 autoui-verifier 的定位序列（scripts/test_vue_playwright.mjs）。
 */

import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';
import assert from 'node:assert';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of possiblePaths) {
    try {
      return await import(pathToFileURL(p).href);
    } catch (_) {}
  }
  throw new Error('Playwright not found');
}

const args = process.argv.slice(2);
const shotsIdx = args.indexOf('--shots');
const shotsDir = shotsIdx >= 0 ? args[shotsIdx + 1] : null;
const base = args.find((a) => a.startsWith('http')) || 'http://localhost:4028';

const chromium = await resolvePlaywright().then((m) => m.chromium);
const browser = await chromium.launch({ headless: true, channel: 'msedge' })
  .catch(() => chromium.launch({ headless: true }));
const page = await (await browser.newContext({ viewport: { width: 1280, height: 800 } })).newPage();
page.on('pageerror', (e) => console.error('[PAGE ERROR]', e));

const shot = async (name) => {
  if (!shotsDir) return;
  fs.mkdirSync(shotsDir, { recursive: true });
  await page.screenshot({ path: path.join(shotsDir, name), fullPage: true });
  console.log(`  [shot] ${name}`);
};
const open = async () => {
  await page.getByRole('button', { name: /Open launcher/ }).click();
  await page.waitForSelector('input', { state: 'visible' });
};
const input = () => page.locator('input');
const rows = () => page.locator('input').locator('xpath=ancestor::div[2]/following-sibling::div[1] >> div.cursor-pointer');

async function main() {
  await page.goto(base, { waitUntil: 'networkidle' });
  // 清 storage 保证可重入
  await page.evaluate(() => {
    for (let i = 0; i < 5; i++) localStorage.removeItem(`launcher.recent_apps.${i}`);
  });
  await page.reload({ waitUntil: 'networkidle' });

  // ---- 1. 键盘流：fill "to" → ↓×2 → Enter 命中 012-stopwatch ----
  console.log('[1] palette keyboard flow: fill "to" → ↓×2 → Enter');
  await open();
  await input().fill('to');
  await page.waitForTimeout(200);
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  const lastText = await page.getByText(/Last launched:/).textContent();
  assert.match(lastText, /Last launched: 012-stopwatch/, `Enter 应命中 012-stopwatch（ ranked: to → [todo, calculator, stopwatch, auto edit]，↓×2 → stopwatch），实际 "${lastText}"`);
  console.log('  ok: Enter 命中 012-stopwatch');
  await shot('t2-1-enter-launch.png');

  // ---- 2. recent 顶置 + storage 持久化（reload 后仍在） ----
  console.log('[2] recent: reopen → stopwatch 顶置 (recent 分组)；reload 后仍顶置');
  await open();
  await page.waitForTimeout(200);
  const firstSub = await page.getByText('recent').first().textContent();
  assert.equal(firstSub, 'recent', 'reopen 后第一行应为 recent 分组');
  const firstTitle = await page.locator('div.cursor-pointer').first().textContent();
  assert.match(firstTitle, /Stopwatch/, `recent 顶置应为 Stopwatch，实际 "${firstTitle}"`);
  await shot('t2-2-recent-top.png');
  await page.reload({ waitUntil: 'networkidle' });
  await open();
  await page.waitForTimeout(200);
  const firstTitle2 = await page.locator('div.cursor-pointer').first().textContent();
  assert.match(firstTitle2, /Stopwatch/, `reload 后 recent 持久化顶置应为 Stopwatch，实际 "${firstTitle2}"`);
  console.log('  ok: recent 顶置 + localStorage 持久化');
  // 清场，避免影响后续用例与重复运行
  await page.evaluate(() => {
    for (let i = 0; i < 5; i++) localStorage.removeItem(`launcher.recent_apps.${i}`);
  });
  await page.reload({ waitUntil: 'networkidle' });

  // ---- 3. Tab → grid；网格方向键 + Enter ----
  console.log('[3] grid: Tab 切换 → ↓ → Enter 命中 015-notes');
  await open();
  await page.keyboard.press('Tab');
  await page.waitForTimeout(200);
  await page.getByText('All apps').waitFor({ state: 'visible' });
  await shot('t2-3-grid.png');
  await page.keyboard.press('ArrowDown'); // gsel: 0 → 4 (015-notes)
  await page.waitForTimeout(200);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  const lastText3 = await page.getByText(/Last launched:/).textContent();
  assert.match(lastText3, /Last launched: 015-notes/, `grid ↓ 应到 015-notes（gsel 0→4），实际 "${lastText3}"`);
  console.log('  ok: grid 方向键 + Enter 命中 015-notes');

  // ---- 4. Esc 逐层退出：清词 → 关闭 ----
  console.log('[4] esc layering: 清词 → 再 Esc 关闭');
  await open();
  await input().fill('calc');
  await page.waitForTimeout(200);
  assert.equal(await input().inputValue(), 'calc');
  await page.keyboard.press('Escape'); // 清词（palette 保持）
  await page.waitForTimeout(200);
  assert.equal(await input().inputValue(), '', '第一次 Esc 应清词');
  const nres = await page.locator('div.cursor-pointer').count();
  assert.equal(nres, 12, `清词后应回到全量 12 行，实际 ${nres}`);
  await shot('t2-4-esc-cleared.png');
  await page.keyboard.press('Escape'); // 关闭
  await page.waitForTimeout(200);
  const openBtn = page.getByRole('button', { name: /Open launcher/ });
  await openBtn.waitFor({ state: 'visible' });
  console.log('  ok: Esc 清词 → 关闭');

  // ---- 5. Ctrl+Space 开启（独立模式自管开关） ----
  console.log('[5] ctrl+space opens palette');
  await page.keyboard.press('Control+Space');
  await page.waitForTimeout(200);
  await page.waitForSelector('input', { state: 'visible' });
  console.log('  ok: Ctrl+Space 召唤');

  await browser.close();
  console.log('\nALL PASS (028-launcher vue 验收)');
}

main().catch((e) => {
  console.error('\nFAIL:', e.message);
  process.exitCode = 1;
  browser.close();
});
