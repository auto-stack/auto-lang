// Plan 508 T4 —— 浏览器远程渲染 002-counter 点击闭环（Playwright 端到端）。
//
// 编排：宿主 harness（cargo test p508_remote_host_body，真 auto.exe
// outproc 生产链 + WS :port）→ vite demo 页 → Chromium：
//   1. 等待 Welcome + 首帧（"Counter: 0" 文本到达画布）；
//   2. 点击 "+"（第 3 个按钮命中区中心）；
//   3. 断言新帧文本 "Counter: 1"（点击 → InputMsg → handler → 新帧）；
//   4. 前后截图留痕（docs/plans/reports/assets/508/）。
//
// 用法：node e2e/p508-remote-e2e.mjs [--smoke]（--smoke = 仅截图不断言点击）。
// Playwright 解析复用 autoui-verifier 约定（标准 node_modules 位点扫描）。

import { execSync, spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const VIEWER = path.resolve(HERE, '..');
const WORKTREE = path.resolve(VIEWER, '../../..');
const ASSETS = path.join(WORKTREE, 'docs/plans/reports/assets/508');
const SMOKE_ONLY = process.argv.includes('--smoke');

const TOKEN = 'p508-e2e-token';
const READY = path.join(WORKTREE, 'target/p508-host.ready');

async function resolvePlaywright() {
  const candidates = [
    // 顺序：本仓 kanban 测试位点（1.62.1 ↔ 缓存 chromium-1234）优先，
    // forge-ui 位点（1.60.0）的 headless-shell-1223 本机缓存缺。
    'd:/autostack/auto-lang/examples/ui/022-kanban/tests/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of candidates) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      return (await import(target)).chromium;
    } catch (_) {
      /* 下一个位点 */
    }
  }
  throw new Error('Playwright not found (autoui-verifier 标准位点)');
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function killTree(proc) {
  if (!proc || proc.exitCode !== null) return;
  try {
    execSync(`taskkill /T /F /PID ${proc.pid}`, { stdio: 'ignore' });
  } catch (_) {
    /* 已退 */
  }
}

async function main() {
  fs.mkdirSync(ASSETS, { recursive: true });
  fs.rmSync(READY, { force: true });

  // ---- 宿主 harness（后台；ready 文件 = 端口同步点）。
  const host = spawn(
    'cargo',
    [
      'test',
      '-p',
      'auto-lang',
      '--lib',
      '--features',
      'ui-iced',
      'p508_remote_host_body',
      '--',
      '--nocapture',
    ],
    {
      cwd: WORKTREE,
      env: {
        ...process.env,
        P508_HOST_TOKEN: TOKEN,
        P508_HOST_PORT: '0',
        P508_HOST_READY: READY,
        P508_HOST_APPS: '002-counter',
      },
      stdio: ['ignore', 'pipe', 'inherit'],
      shell: true,
    },
  );
  let port = 0;
  for (let i = 0; i < 1200 && !port; i++) {
    if (fs.existsSync(READY)) {
      port = parseInt(fs.readFileSync(READY, 'utf8').trim(), 10);
      break;
    }
    await sleep(200);
  }
  if (!port) {
    killTree(host);
    throw new Error('host ready 超时（120s）');
  }
  console.log(`[e2e] host ws=127.0.0.1:${port}`);

  // ---- vite demo 页（后台）。
  const vite = spawn('pnpm', ['dev'], { cwd: VIEWER, stdio: ['ignore', 'pipe', 'inherit'], shell: true });
  let viteUp = false;
  for (let i = 0; i < 150 && !viteUp; i++) {
    try {
      const res = await fetch('http://127.0.0.1:5180/');
      viteUp = res.ok;
    } catch (_) {
      /* 未起 */
    }
    if (!viteUp) await sleep(200);
  }
  if (!viteUp) {
    killTree(vite);
    killTree(host);
    throw new Error('vite 起服超时（30s）');
  }

  // ---- Chromium 闭环。
  const chromium = await resolvePlaywright();
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 900, height: 720 } });
  try {
    await page.goto(`http://127.0.0.1:5180/?token=${TOKEN}&port=${port}&app=002-counter`);
    const probe = () => page.evaluate(() => (window).__remote);
    // Welcome + 首帧（Counter: 0）。
    let state = null;
    for (let i = 0; i < 300; i++) {
      state = await probe();
      if (state?.welcome && state?.frames > 0 && state.lastTexts.includes('Counter: 0')) break;
      await sleep(100);
    }
    if (!(state?.welcome && state?.frames > 0)) throw new Error('Welcome/首帧超时');
    console.log(`[e2e] welcome ✓ 首帧 ✓ texts=${JSON.stringify(state.lastTexts)}`);
    await page.screenshot({ path: path.join(ASSETS, 'p508-e2e-before-click.png') });

    if (!SMOKE_ONLY) {
      // 点击 "+"（第 3 个按钮命中区）。
      const centers = await page.evaluate(() => (window).__remote.buttonCenters());
      const plus = centers[2];
      if (!plus) throw new Error(`'+' 命中区缺失: ${JSON.stringify(centers)}`);
      await page.mouse.click(plus.x, plus.y);
      let after = null;
      for (let i = 0; i < 200; i++) {
        after = await probe();
        if (after?.lastTexts.includes('Counter: 1')) break;
        await sleep(50);
      }
      if (!after?.lastTexts.includes('Counter: 1')) {
        throw new Error(`点击闭环超时: clicks=${after?.clicks} texts=${JSON.stringify(after?.lastTexts)}`);
      }
      await page.screenshot({ path: path.join(ASSETS, 'p508-e2e-after-click.png') });
      console.log(`[e2e] 点击闭环 ✓ clicks=${after.clicks} texts=${JSON.stringify(after.lastTexts)}`);
    }
    console.log('[e2e] PASS');
  } finally {
    await browser.close().catch(() => {});
    killTree(vite);
    killTree(host);
    fs.rmSync(READY, { force: true });
  }
}

main().catch((e) => {
  console.error(`[e2e] FAIL: ${e.message}`);
  process.exit(1);
});
