// Plan 508 T6 —— 实机演示：真桌面窗（ui_desktop 验收宿主，outproc 模式
// 跑示例批）× 浏览器真窗（headed Chromium 远程镜像）双向闭环。
//
// 编排（505 验收通道 + 本计划 e2e 复合）：
//   1. ui_desktop --apps-dir examples/ui（storage: process_model=outproc +
//      shell.remote.token）→ MCP 就绪；
//   2. bus 注入 `launch␟002-counter` → outproc 子进程（真 auto.exe）落地
//      → 桌面虚拟窗渲染 queue 臂 DrawList；
//   3. vite viewer + headed Chromium → 远程镜像渲染 Counter: 0；
//   4. 浏览器点 "+" → Counter: 1（新帧推送）+ 桌面侧截图（同一会话状态）；
//   5. 证据 → docs/plans/reports/assets/508/。
//
// 用法：node e2e/p508-t6-real-machine.mjs

import { execSync, spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const VIEWER = path.resolve(HERE, '..');
const WORKTREE = path.resolve(VIEWER, '../../..');
const ASSETS = path.join(WORKTREE, 'docs/plans/reports/assets/508');
const DESKTOP_EXE = path.join(WORKTREE, 'target/debug/examples/ui_desktop.exe');
const STORAGE = path.join(WORKTREE, 'target/p508-t6-storage.json');
const MCP_PORT = 18099;
const TOKEN = 't6-real-token';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function mcpCall(name, args) {
  const res = await fetch(`http://127.0.0.1:${MCP_PORT}/mcp`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: Date.now(), method: 'tools/call', params: { name, arguments: args ?? {} } }),
  });
  const body = await res.json();
  if (body.error) throw new Error(`${name}: ${JSON.stringify(body.error)}`);
  return (body.result?.content ?? [])
    .filter((p) => p.type === 'text')
    .map((p) => p.text)
    .join('\n');
}

function killTree(proc) {
  if (!proc || proc.exitCode !== null) return;
  try {
    execSync(`taskkill /T /F /PID ${proc.pid}`, { stdio: 'ignore' });
  } catch (_) {
    /* 已退 */
  }
}

async function resolvePlaywright() {
  const candidates = [
    'd:/autostack/auto-lang/examples/ui/022-kanban/tests/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of candidates) {
    try {
      const target = p.includes(':') ? pathToFileURL(p).href : p;
      return (await import(target)).chromium;
    } catch (_) {
      /* 下一个位点 */
    }
  }
  throw new Error('Playwright not found');
}

async function main() {
  fs.mkdirSync(ASSETS, { recursive: true });
  if (!fs.existsSync(DESKTOP_EXE)) throw new Error(`ui_desktop 缺失: ${DESKTOP_EXE}`);
  fs.writeFileSync(
    STORAGE,
    JSON.stringify({
      'shell.apps.process_model': 'outproc',
      'shell.remote.token': TOKEN,
    }, null, 2),
  );

  // ---- 桌面宿主（outproc + 远程 token）。
  const desktop = spawn(
    DESKTOP_EXE,
    ['--apps-dir', path.join(WORKTREE, 'examples/ui')],
    {
      cwd: WORKTREE,
      env: {
        ...process.env,
        AUTOUI_ACCEPTANCE: '1',
        AUTOUI_MCP_PORT: String(MCP_PORT),
        AUTO_VM_STORAGE_FILE: STORAGE,
      },
      stdio: ['ignore', 'ignore', 'inherit'],
    },
  );
  try {
    let ready = false;
    for (let i = 0; i < 100 && !ready; i++) {
      try {
        await mcpCall('autoui_check');
        ready = true;
      } catch (_) {
        await sleep(300);
      }
    }
    if (!ready) throw new Error('desktop MCP 未就绪');
    console.log('[t6] desktop MCP ready');
    // MCP 监听先于桌面 boot 完成（registry/shell 装配在后）——等 boot
    // 落定再注入 bus（过早的记录会随 shell 挂载丢失，实测教训）。
    await sleep(6000);

    // ---- outproc 示例批：launch 002-counter（真 auto.exe 子进程）。
    // 实机纪律：`__desktop_cmd` 单槽位——真用户交互（图标按压等）可
    // 覆写排队记录；改为页面先连 + launch 周期重发直至落地（下方循环）。
    const REC_SEP = '';
    const sendLaunch = () =>
      mcpCall('autoui_desktop', { action: 'bus', verb: `launch${REC_SEP}002-counter` });

    // ---- vite + headed browser。
    const vite = spawn('pnpm', ['dev'], { cwd: VIEWER, stdio: ['ignore', 'ignore', 'inherit'], shell: true });
    globalThis.__vite = vite;
    let viteUp = false;
    for (let i = 0; i < 150 && !viteUp; i++) {
      try {
        viteUp = (await fetch('http://127.0.0.1:5180/')).ok;
      } catch (_) {
        /* 未起 */
      }
      if (!viteUp) await sleep(200);
    }
    if (!viteUp) throw new Error('vite 起服超时');

    const chromium = await resolvePlaywright();
    const browser = await chromium.launch({ headless: false });
    const page = await browser.newPage({ viewport: { width: 900, height: 720 } });
    const pageLogs = [];
    page.on('console', (m) => pageLogs.push(`[page:${m.type()}] ${m.text().slice(0, 160)}`));
    page.on('pageerror', (e) => pageLogs.push(`[pageerror] ${String(e).slice(0, 200)}`));
    await page.goto(`http://127.0.0.1:5180/?token=${TOKEN}&port=17800&app=002-counter`);
    const probe = () => page.evaluate(() => (window).__remote);
    let state = null;
    let launched = 0;
    for (let i = 0; i < 200; i++) {
      if (i % 25 === 0 && launched < 8) {
        await sendLaunch();
        launched += 1;
      }
      state = await probe();
      if (state?.welcome && state?.frames > 0 && state.lastTexts.includes('Counter: 0')) break;
      await sleep(150);
    }
    console.log(`[t6] launch 共发 ${launched} 轮，texts=${JSON.stringify(state?.lastTexts)}`);
    if (!state?.welcome) {
      console.log('[t6] page logs: ' + pageLogs.slice(-20).join(' | '));
      try {
        await mcpCall('autoui_screenshot', { name: 'p508-t6-failstate', baseline: true });
        const shot = path.join(WORKTREE, 'tests/screenshots/p508-t6-failstate.png');
        if (fs.existsSync(shot)) fs.copyFileSync(shot, path.join(ASSETS, 'p508-t6-failstate.png'));
        console.log('[t6] failstate 桌面截图已存');
      } catch (_) { /* 尽力 */ }
      throw new Error(`远程镜像 Welcome/首帧超时 state=${JSON.stringify(state)}`);
    }
    console.log(`[t6] browser mirror ✓ texts=${JSON.stringify(state.lastTexts)}`);
    await page.screenshot({ path: path.join(ASSETS, 'p508-t6-browser-before.png') });

    // 浏览器点击 "+" → 同一会话状态推进 → 桌面窗同步新帧。
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
      throw new Error(`浏览器点击闭环超时: ${JSON.stringify(after?.lastTexts)}`);
    }
    console.log(`[t6] browser click ✓ texts=${JSON.stringify(after.lastTexts)}`);
    await page.screenshot({ path: path.join(ASSETS, 'p508-t6-browser-after.png') });
    await sleep(1500);

    // 桌面侧截图（queue 臂虚拟窗——outproc DrawList 直挂）。
    const shotOut = await mcpCall('autoui_screenshot', { name: 'p508-t6-desktop', baseline: true });
    const produced = path.join(WORKTREE, 'tests/screenshots/p508-t6-desktop.png');
    for (let i = 0; i < 15 && !fs.existsSync(produced); i++) await sleep(200);
    if (fs.existsSync(produced)) {
      fs.copyFileSync(produced, path.join(ASSETS, 'p508-t6-desktop.png'));
      console.log('[t6] desktop shot ✓');
    } else {
      console.log(`[t6] desktop shot 缺失: ${shotOut.slice(0, 120)}`);
    }
    await browser.close().catch(() => {});
    console.log('[t6] PASS（outproc 桌面示例批 + 浏览器真窗双向闭环）');
  } finally {
    killTree(globalThis.__vite);
    killTree(desktop);
  }
}

main()
  .catch((e) => {
    console.error(`[t6] FAIL: ${e.message}`);
    process.exitCode = 1;
  })
  .finally(() => {
    fs.rmSync(STORAGE, { force: true });
  });
