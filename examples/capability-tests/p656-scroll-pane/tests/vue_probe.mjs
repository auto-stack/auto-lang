/**
 * p656-scroll-pane Vue 轨验证驱动 —— PLAN-656 review F-R1 落地。
 *
 * 与 tests/vm_probe.py（VM/MCP 腿）对称的 Vue 腿实机断言：
 * 1. 布局面：y-row-01 / x-cell-01 / obs-1 / managed 按钮在场。
 * 2. ordinary controller 全链：click to-end → click probe → probe > 0
 *    （data-scroll-ctl 锚 + JS helper 族 scrollTo → scroll_state 投影）。
 * 3. managed controller 全链：m-to-end → m-probe ≥ 9,000,000
 *    （10M 逻辑 extent 的 logical spacer 驱动 range；收敛轮询——
 *    物化同步中间值会短暂回写，同 vm_probe 的 1/8 抢读实证）。
 * 4. on-scroll 观察：obs pane 滚动后 oy > 0 且 py > 0（8 位置实参派发；
 *    程序化 scrollTop 赋值后显式 dispatchEvent——IAB/嵌入式 webview 不为
 *    程序化赋值派发 scroll 事件，review F-R4 方法论）。
 * 5. managed spacer 形态：巨大 scrollHeight 节点恰 2 个（wrapper+spacer），
 *    spacer 子节点数 0、逻辑高度 1e7px（AC-11 DOM 节点不随 extent 膨胀）。
 * 6. 归档：截图 tests/vue_review.png + 状态落盘 tests/vue_state_dump.json
 *    （与 vm_review.png/vm_snapshot.txt 对称，§14.8 双端 state dump 素材）。
 *
 * 环境前置（KNOWN-DEBT 656）：脚手架 prismjs ^1.29.0 浮动解析到 1.30.0 会
 * 使 vite transformCjsImport 崩（main.ts 500、#app 空挂载）——本脚本自愈：
 * 检测到该失败形态时钉回 1.29.0 + pnpm install + 清 .vite 缓存后重启。
 * 根修归 auto-man 脚手架面。
 *
 * 用法：node tests/vue_probe.mjs （cwd 任意；AUTO_BIN 覆盖 auto 可执行档）。
 */
import { execSync, spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { pathToFileURL } from 'url';

const APP_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const PORT = 3000;
const BASE = `http://localhost:${PORT}`;

// ── Playwright 解析（autoui-verifier 技能脚本同款多路径约定） ──────────
async function resolvePlaywright() {
  const candidates = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'd:/autostack/auto-lang/examples/ui/022-kanban/tests/node_modules/playwright/index.mjs',
    'playwright',
  ];
  for (const p of candidates) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      const mod = await import(target);
      return mod.chromium;
    } catch (_) { /* try next */ }
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

// ── dev server 生命周期 ────────────────────────────────────────────────
function startAutoRun() {
  const bin = process.env.AUTO_BIN || 'auto';
  const proc = spawn(bin, ['run'], { cwd: APP_DIR, shell: true });
  let tail = '';
  proc.stdout.on('data', d => { tail += d.toString(); if (tail.length > 8000) tail = tail.slice(-8000); });
  proc.stderr.on('data', d => { tail += d.toString(); if (tail.length > 8000) tail = tail.slice(-8000); });
  return { proc, getTail: () => tail };
}

async function stopAutoRun(handle) {
  if (!handle || handle.proc.exitCode !== null) return;
  const pid = handle.proc.pid;
  try { handle.proc.kill(); } catch (_) { /* already gone */ }
  try {
    // Windows：vite 是 auto run 的子进程，须连树终止（实测残留锁端口）。
    execSync(`taskkill /PID ${pid} /T /F`, { stdio: 'ignore' });
  } catch (_) { /* non-Windows 或已退出 */ }
}

async function pollMainTs(timeoutMs) {
  const t0 = Date.now();
  let lastCode = 0;
  while (Date.now() - t0 < timeoutMs) {
    try {
      const res = await fetch(`${BASE}/src/main.ts`);
      lastCode = res.status;
      if (res.status === 200) return { ok: true };
      await res.arrayBuffer().catch(() => {});
    } catch (_) { /* server not up yet */ }
    await new Promise(r => setTimeout(r, 1500));
  }
  return { ok: false, lastCode };
}

// ── prismjs 1.30.0 上游漂移自愈（KNOWN-DEBT 656，根修在 auto-man） ────
function pinPrismjs() {
  const pkgPath = path.join(APP_DIR, 'gen/front/vue/package.json');
  if (!fs.existsSync(pkgPath)) return false;
  const src = fs.readFileSync(pkgPath, 'utf8');
  if (!src.includes('"prismjs": "^1.29.0"')) return false;
  fs.writeFileSync(pkgPath, src.replace('"prismjs": "^1.29.0"', '"prismjs": "1.29.0"'));
  execSync('pnpm install --silent', { cwd: path.join(APP_DIR, 'gen/front/vue'), stdio: 'ignore' });
  const viteCache = path.join(APP_DIR, 'gen/front/vue/node_modules/.vite');
  fs.rmSync(viteCache, { recursive: true, force: true });
  console.log('[*] prismjs pinned 1.29.0 (KNOWN-DEBT 656 workaround), vite cache cleared');
  return true;
}

// ── 断言面 ─────────────────────────────────────────────────────────────
const fail = [];
function check(ok, label, detail) {
  console.log((ok ? '[+] ' : '[-] ') + label + (detail ? `: ${detail}` : ''));
  if (!ok) fail.push(label);
}

const readStatus = `(() => {
  const el = [...document.querySelectorAll('*')].find(x => x.children.length === 0 && /oy=/.test(x.textContent || ''));
  return el ? el.textContent : '';
})()`;

// mprobe 独立文本节点（`mprobe=${.mprobe}`），不在 oy 状态行内。
const readMprobe = `(() => {
  const el = [...document.querySelectorAll('*')].find(x => x.children.length === 0 && /mprobe=/.test(x.textContent || ''));
  return el ? el.textContent : '';
})()`;

function parseField(status, name) {
  const m = status.match(new RegExp(`${name}=([-\\d.eE+]+)`));
  return m ? parseFloat(m[1]) : null;
}

async function main() {
  console.log(`[*] auto run (vue) @ ${APP_DIR}`);
  let handle = startAutoRun();
  try {
    let boot = await pollMainTs(90_000);
    if (!boot.ok) {
      // 失败形态甄别：500 = prismjs interop 崩 → 钉版自愈重启；其它 = 报错退出。
      if (boot.lastCode === 500 && pinPrismjs()) {
        await stopAutoRun(handle);
        console.log('[*] restarting after prismjs pin...');
        await new Promise(r => setTimeout(r, 3000));
        handle = startAutoRun();
        boot = await pollMainTs(90_000);
      }
      if (!boot.ok) {
        console.log(`[-] vite dev not ready (main.ts HTTP ${boot.lastCode})`);
        console.log('--- auto run tail ---\n' + handle.getTail().slice(-2000));
        return 2;
      }
    }
    console.log('[+] vite dev ready');

    const chromium = await resolvePlaywright();
    let browser;
    for (const channel of ['msedge', 'chrome', undefined]) {
      try { browser = await chromium.launch({ headless: true, ...(channel ? { channel } : {}) }); break; } catch (_) {}
    }
    if (!browser) throw new Error('chromium launch failed');
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, colorScheme: 'dark' });
    const page = await context.newPage();

    // goto 重试环：auto run 的 vite 可能在首 200 后短暂重启（auto-sources
    // 重写/依赖安装窗口），单发 goto 会撞连接间隙。
    let mounted = false, lastErr = null;
    for (let i = 0; i < 4 && !mounted; i++) {
      try {
        await page.goto(BASE, { waitUntil: 'domcontentloaded', timeout: 10_000 });
        await page.waitForSelector(`text=PLAN-656 scroll-pane capability`, { timeout: 15_000 });
        mounted = true;
      } catch (e) {
        lastErr = e;
        console.log(`[*] goto/mount attempt ${i + 1} failed (${String(e.message || e).split('\n')[0]}), retrying...`);
        await pollMainTs(30_000);
        await new Promise(r => setTimeout(r, 2000));
      }
    }
    if (!mounted) throw new Error('app mount failed after retries: ' + (lastErr && lastErr.message));
    console.log('[+] app mounted');

    const clickByName = async (name) => {
      const n = await page.evaluate((x) => {
        const b = [...document.querySelectorAll('button')].find(e => e.textContent.trim() === x);
        if (b) { b.click(); return true; }
        return false;
      }, name);
      if (!n) throw new Error(`button not found: ${name}`);
    };
    const status = () => page.evaluate(readStatus);

    // 1. 布局面
    for (const [needle, why] of [['y-row-01', 'ordinary y pane'], ['x-cell-01', 'ordinary x pane'], ['obs-1', 'observe pane'], ['m-to-end', 'managed controls']]) {
      const ok = await page.evaluate((t) => document.body.textContent.includes(t), needle);
      check(ok, `layout: ${why}`);
    }

    // 2. ordinary controller 全链
    await clickByName('to-end');
    await page.waitForTimeout(600);
    await clickByName('probe');
    await page.waitForTimeout(300);
    const st1 = await status();
    const probe = parseField(st1, 'probe');
    check(probe !== null && probe > 0, 'ordinary controller chain', `probe=${probe}`);

    // 3. managed controller 全链（收敛轮询）
    let mprobe = null;
    await clickByName('m-to-end');
    for (let i = 0; i < 6; i++) {
      await page.waitForTimeout(1200);
      await clickByName('m-probe');
      const st = await page.evaluate(readMprobe);
      mprobe = parseField(st, 'mprobe');
      if (mprobe !== null && mprobe >= 9_000_000) break;
    }
    check(mprobe !== null && mprobe >= 9_000_000, 'managed controller chain', `mprobe=${mprobe} (>=9M, logical extent drives range)`);

    // 4. on-scroll 观察（scrollTop 赋值 + 显式 dispatchEvent，F-R4 方法论）
    const obs = await page.evaluate(async () => {
      const el = [...document.querySelectorAll('div')].filter(x =>
        x.textContent.includes('obs-1') && x.clientHeight > 0 && x.clientHeight < 300 && x.scrollHeight > x.clientHeight + 10
      ).sort((a, b) => a.clientHeight - b.clientHeight)[0];
      if (!el) return null;
      el.scrollTop = 60;
      el.dispatchEvent(new Event('scroll'));
      await new Promise(r => setTimeout(r, 400));
      return { cls: el.className.slice(0, 40), scrollTop: el.scrollTop };
    });
    check(!!obs, 'obs pane located', obs ? `${obs.cls} scrollTop=${obs.scrollTop}` : 'not found');
    const st4 = await status();
    const oy = parseField(st4, 'oy'), py = parseField(st4, 'py');
    check(oy !== null && oy > 0, 'on-scroll observe: oy>0', `oy=${oy}`);
    check(py !== null && py > 0, 'on-scroll observe: py>0 (8 positional args)', `py=${py}`);

    // 5. managed spacer 形态（AC-11：节点数不随 logical extent 膨胀）
    const spacer = await page.evaluate(() => {
      const huge = [...document.querySelectorAll('*')].filter(x => x.scrollHeight > 1_000_000);
      const inner = huge.filter(x => (parseInt(x.style?.height, 10) || 0) > 1_000_000 || x.style?.height === '1e+07px');
      return {
        hugeCount: huge.length,
        wrapperChildCount: huge.length ? huge[0].children.length : -1,
        spacerCount: inner.length,
        spacerChildCount: inner.length ? inner[0].children.length : -1,
        spacerHeight: inner.length ? inner[0].style.height : null,
      };
    });
    check(spacer.hugeCount === 2 && spacer.spacerCount === 1 && spacer.spacerChildCount === 0 && spacer.wrapperChildCount === 1,
      'managed spacer: single logical node, no DOM bloat',
      JSON.stringify(spacer));

    // 6. 归档（截图 + 状态 dump，§14.8 双端素材）
    const shotPath = path.join(APP_DIR, 'tests/vue_review.png');
    await page.screenshot({ path: shotPath, fullPage: true });
    console.log(`[*] screenshot: ${path.relative(APP_DIR, shotPath)}`);
    const dump = {
      backend: 'vue', captured_at: new Date().toISOString(),
      probe, mprobe, oy, py, vy: parseField(await status(), 'vy'), spacer,
    };
    fs.writeFileSync(path.join(APP_DIR, 'tests/vue_state_dump.json'), JSON.stringify(dump, null, 2) + '\n');

    await browser.close();
    console.log('='.repeat(50));
    if (fail.length) { console.log('[-] FAILURES: ' + fail.join('; ')); return 2; }
    console.log('[+] ALL P656 VUE CHECKS PASSED');
    return 0;
  } finally {
    await stopAutoRun(handle);
  }
}

main().then(code => process.exit(code)).catch(e => { console.error('[!] ' + (e.stack || e)); process.exit(3); });
