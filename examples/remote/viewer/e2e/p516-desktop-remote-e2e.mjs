// Plan 516 T4 —— vue 桌面远程窗全链端到端（Playwright）。
//
// 编排（复用 508 e2e 基建约定）：
//   1. WS 宿主 harness（cargo test p508_remote_host_body，真 auto.exe
//      outproc 生产链 + WS :port，ready 文件同步端口）；
//   2. vue 虚拟桌面宿主（auto run --render vue --desktop --apps examples/ui，
//      vite :5190）；
//   3. Chromium 打开桌面 + URL 注入远程会话（?remote=…&app=002-counter）：
//      a. Welcome + 首帧（"Counter: 0" 落画布，远程窗受 WM 管理）；
//      b. 点击 "+"（命中区中心）→ "Counter: 1"（输入回发闭环）；
//      c. 拖动标题栏 → rect 变化（WM 布局同权）；
//      d. 任务栏：launcher 开本地 App → 点远程任务栏条目 → 焦点切回
//         （与本地窗同权混排）；
//      e. kill 宿主 harness → 状态面 dead + 窗口保留（断线不消失）。
//   前后截图留痕 docs/plans/reports/assets/516/。
//
// 用法：node e2e/p516-desktop-remote-e2e.mjs
// Playwright 解析复用 autoui-verifier 约定（标准 node_modules 位点扫描）。

import { execSync, spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const WORKTREE = path.resolve(HERE, '../../../..');
const APPS = path.join(WORKTREE, 'examples/ui');
const ASSETS = path.join(WORKTREE, 'docs/plans/reports/assets/516');
const DESKTOP_PORT = 5190;

const TOKEN = 'p516-e2e-token';
const READY = path.join(WORKTREE, 'target/p516-host.ready');

async function resolvePlaywright() {
  const candidates = [
    // 顺序沿用 508 e2e（本仓 kanban 位点优先）+ 主检出绝对位点（worktree
    // 内 node_modules 不入库）。
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
  throw new Error('Playwright not found（autoui-verifier 标准位点）');
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

/** auto.exe 在位（WS harness 的 outproc 子进程与桌面宿主共用）。 */
function ensureAutoExe() {
  const exe = path.join(WORKTREE, 'target/debug/auto.exe');
  if (fs.existsSync(exe)) return exe;
  console.log('[e2e] building auto.exe (first run in this worktree)...');
  execSync('cargo build -p auto --bin auto', { cwd: WORKTREE, stdio: 'inherit' });
  return exe;
}

/** 页面内远程窗探针（RemoteWindow 挂 window.__p516[wid]）。 */
async function remoteProbe(page) {
  return page.evaluate(() => {
    const all = window.__p516 ?? {};
    const wid = Object.keys(all)[0];
    return wid ? { wid: Number(wid), ...all[wid] } : null;
  });
}

async function main() {
  fs.mkdirSync(ASSETS, { recursive: true });
  fs.rmSync(READY, { force: true });
  const autoExe = ensureAutoExe();

  // ---- WS 宿主 harness（后台；ready 文件 = 端口同步点）。
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
  for (let i = 0; i < 1500 && !port; i++) {
    if (fs.existsSync(READY)) {
      port = parseInt(fs.readFileSync(READY, 'utf8').trim(), 10);
      break;
    }
    await sleep(200);
  }
  if (!port) {
    killTree(host);
    throw new Error('host ready 超时（300s）');
  }
  console.log(`[e2e] host ws=127.0.0.1:${port}`);

  // ---- vue 虚拟桌面宿主（auto run --desktop；vite :5190）。
  const desktop = spawn(
    autoExe,
    [
      'run',
      '--render',
      'vue',
      '--desktop',
      '--apps',
      APPS,
      '-F',
      String(DESKTOP_PORT),
    ],
    {
      cwd: path.join(APPS, '002-counter'),
      env: {
        ...process.env,
        TAURI_ENV: '1', // vite open 抑制（Playwright 自管浏览器）
      },
      stdio: ['ignore', 'pipe', 'inherit'],
    },
  );
  let viteUp = false;
  for (let i = 0; i < 900 && !viteUp; i++) {
    try {
      const res = await fetch(`http://localhost:${DESKTOP_PORT}/`);
      viteUp = res.ok;
    } catch (_) {
      /* 未起（首跑含 scaffold+install，可到 3 分钟） */
    }
    if (!viteUp) await sleep(200);
  }
  if (!viteUp) {
    killTree(desktop);
    killTree(host);
    throw new Error('vue 桌面起服超时（180s）');
  }
  console.log(`[e2e] desktop vite=127.0.0.1:${DESKTOP_PORT}`);

  // ---- Chromium 全链。
  const chromium = await resolvePlaywright();
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  try {
    // ---- 阶段 0（T5）：无远程配置的裸桌面——零远程窗 + launcher 可用。
    await page.goto(`http://localhost:${DESKTOP_PORT}/`);
    await page.waitForSelector('footer', { timeout: 10_000 });
    const bareRemote = await page.locator('canvas.remote-canvas').count();
    if (bareRemote !== 0) throw new Error(`无配置不应有远程窗（count=${bareRemote}）`);
    await page.click('footer button[title="Summon launcher (Ctrl+Space)"]');
    await page.waitForSelector('input[placeholder^="search apps"]', { timeout: 5000 });
    const appRows = await page.locator('.absolute button:has-text("Counter")').count();
    if (appRows === 0) throw new Error('launcher 未列出本地 App');
    console.log('[e2e] 无配置零变化 ✓（零远程窗，launcher 正常）');
    await page.screenshot({ path: path.join(ASSETS, '00-bare-desktop-no-config.png') });

    // ---- 全链注入远程会话。
    const wsUrl = `ws://127.0.0.1:${port}/?token=${encodeURIComponent(TOKEN)}`;
    const url =
      `http://localhost:${DESKTOP_PORT}/` +
      `?remote=${encodeURIComponent(wsUrl)}&app=002-counter` +
      `&title=${encodeURIComponent('Counter (remote)')}&rid=counter&rbudget=6000`;
    await page.goto(url);

    // a. Welcome + 首帧（Counter: 0 落画布）。
    let probe = null;
    for (let i = 0; i < 300; i++) {
      probe = await remoteProbe(page);
      if (probe?.welcome && probe.frames > 0 && probe.lastTexts.includes('Counter: 0')) break;
      await sleep(100);
    }
    if (!(probe?.welcome && probe?.frames > 0)) {
      throw new Error(`Welcome/首帧超时: ${JSON.stringify(probe)}`);
    }
    console.log(`[e2e] 远程窗在线 ✓ wid=${probe.wid} texts=${JSON.stringify(probe.lastTexts)}`);
    await page.screenshot({ path: path.join(ASSETS, '01-remote-window-online.png') });

    // b. 点击 "+"（第 3 个按钮命中区中心）→ Counter: 1。
    const centers = await page.evaluate((wid) => window.__p516[wid].buttonCenters(), probe.wid);
    const plus = centers[2];
    if (!plus) throw new Error(`'+' 命中区缺失: ${JSON.stringify(centers)}`);
    await page.mouse.click(plus.x, plus.y);
    let after = null;
    for (let i = 0; i < 200; i++) {
      after = await remoteProbe(page);
      if (after?.lastTexts.includes('Counter: 1')) break;
      await sleep(50);
    }
    if (!after?.lastTexts.includes('Counter: 1')) {
      throw new Error(`点击闭环超时: ${JSON.stringify(after)}`);
    }
    console.log(`[e2e] 点击闭环 ✓ revision=${after.revision} texts=${JSON.stringify(after.lastTexts)}`);
    await page.screenshot({ path: path.join(ASSETS, '02-click-closed-loop.png') });

    // c. 拖动远程窗标题栏 → rect 变化（WM 布局同权）。
    const remoteWin = page.locator('.virtual-window:has(canvas.remote-canvas)');
    const before = await remoteWin.evaluate((el) => el.style.left + ',' + el.style.top);
    const bar = await remoteWin.locator('.title-bar').boundingBox();
    await page.mouse.move(bar.x + bar.width / 2, bar.y + 6);
    await page.mouse.down();
    await page.mouse.move(bar.x + bar.width / 2 + 140, bar.y + 66, { steps: 8 });
    await page.mouse.up();
    const moved = await remoteWin.evaluate((el) => el.style.left + ',' + el.style.top);
    if (moved === before) throw new Error(`拖动未生效: ${before}`);
    console.log(`[e2e] 拖动 ✓ ${before} → ${moved}`);
    await page.screenshot({ path: path.join(ASSETS, '03-drag-rect-change.png') });

    // d. 任务栏同权：launcher 开本地 App → 远程失焦 → 点远程条目复焦。
    await page.click('footer button[title="Summon launcher (Ctrl+Space)"]');
    await page.waitForSelector('input[placeholder^="search apps"]', { timeout: 5000 });
    await page.click('.absolute button:has-text("Counter")');
    await page.waitForSelector('.virtual-window:not(:has(canvas.remote-canvas))', { timeout: 5000 });
    let remoteFocused = await remoteWin.evaluate((el) => el.classList.contains('border-primary/60'));
    if (remoteFocused) throw new Error('本地窗启动后远程窗应失焦');
    await page.click('footer button:has-text("Counter (remote)")');
    remoteFocused = await remoteWin.evaluate((el) => el.classList.contains('border-primary/60'));
    if (!remoteFocused) throw new Error('任务栏点击后远程窗应复焦');
    console.log('[e2e] 任务栏聚焦切换 ✓（本地+远程混排同权）');
    await page.screenshot({ path: path.join(ASSETS, '04-taskbar-focus-toggle.png') });

    // e. 断线：kill 宿主 → dead 状态面 + 窗口保留。
    killTree(host);
    let dead = null;
    for (let i = 0; i < 100; i++) {
      dead = await page.evaluate(() => {
        const el = document.querySelector('[data-remote-status]')
        return el ? el.getAttribute('data-remote-status') : null
      });
      if (dead === 'dead' || dead === 'reconnecting') break;
      await sleep(100);
    }
    if (dead !== 'dead' && dead !== 'reconnecting') {
      throw new Error(`断线状态面超时: status=${dead}`);
    }
    const retained = await remoteWin.count();
    if (retained !== 1) throw new Error(`断线后远程窗应保留（count=${retained}）`);
    await sleep(dead === 'reconnecting' ? 6500 : 200); // 预算 6s 耗尽 → dead
    const finalStatus = await page.evaluate(() => document.querySelector('[data-remote-status]')?.getAttribute('data-remote-status'));
    if (finalStatus !== 'dead') throw new Error(`预算耗尽应 dead: ${finalStatus}`);
    console.log('[e2e] 断线保留 ✓（窗口不消失，状态面可见）');
    await page.screenshot({ path: path.join(ASSETS, '05-disconnected-overlay-retained.png') });

    console.log('[e2e] PASS');
  } finally {
    await browser.close().catch(() => {});
    killTree(desktop);
    killTree(host);
    fs.rmSync(READY, { force: true });
  }
}

main().catch((e) => {
  console.error(`[e2e] FAIL: ${e.message}`);
  process.exit(1);
});
