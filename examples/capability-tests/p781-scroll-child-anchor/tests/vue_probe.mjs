/**
 * p781-scroll-child-anchor Vue 腿验证驱动 —— PLAN-781 阶段B T-05。
 *
 * 与 tests/vm_probe.py（MCP 原生腿）对称的全矩阵断言（合同 =
 * docs/plans/reports/781-interface-decision.md；Vue 轨语义镜像 = §7 r5）：
 *   V-01 双 4000 字长段（中/英）顶对齐；V-02 非连续 key 身份；
 *   V-03 Init(onMounted) 先发目标 + 双章换绑（v-if 双 pane）；
 *   V-04 负例族（空key/bad_handle/幽灵 not_bound/重复 last-wins/
 *          越界/跨pane 不串线）；V-05 字号/宽度重排 + 已可见不跳；
 *   V-06 旧 offset 族回归 + applied 后真滚动(mouse.wheel)不回拉 +
 *          managed(data-scroll-managed) unsupported。
 *
 * 几何 = [data-scroll-anchor] 真实 DOM rect ∩ pane 客户区（正面积）；
 * 回执 = R=<receipt> 显示面；用户滚动 = mouse.wheel（真实 wheel 事件）。
 * 证据（几何 JSON/截图/CLI hash）→ P781_T05_OUT（默认 tests/evidence-vue）。
 *
 * 用法：node tests/vue_probe.mjs
 * AUTO_BIN 覆盖 auto 可执行档；VUE_PORT/VUE_BACK_PORT 覆盖端口。
 */
import { execSync, spawn } from 'child_process';
import crypto from 'crypto';
import fs from 'fs';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';

const APP_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const PORT = Number(process.env.VUE_PORT || 14805);
const BACK_PORT = Number(process.env.VUE_BACK_PORT || 14806);
const BASE = `http://localhost:${PORT}`;
const OUT = process.env.P781_T05_OUT ||
  path.join(path.dirname(fileURLToPath(import.meta.url)), 'evidence-vue');

const MAIN = '[data-scroll-ctl="main-pane"]';
const SIDE = '[data-scroll-ctl="side-pane"]';

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
      return (await import(target)).chromium;
    } catch { /* try next */ }
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

function defaultAutoBin() {
  if (process.env.AUTO_BIN) return process.env.AUTO_BIN;
  const wt = 'D:/autostack/.wt/lang-781/auto-lang/target/debug/auto.exe';
  return fs.existsSync(wt) ? wt : 'auto';
}

function startAutoRun(bin) {
  const proc = spawn(bin, ['run', '-F', String(PORT), '-B', String(BACK_PORT)],
    { cwd: APP_DIR, shell: true });
  let tail = '';
  const onD = d => { tail += d.toString(); if (tail.length > 12000) tail = tail.slice(-12000); };
  proc.stdout.on('data', onD);
  proc.stderr.on('data', onD);
  return { proc, getTail: () => tail };
}

async function stopAutoRun(handle) {
  if (!handle || handle.proc.exitCode !== null) return;
  try { handle.proc.kill(); } catch { /* gone */ }
  try { execSync(`taskkill /PID ${handle.proc.pid} /T /F`, { stdio: 'ignore' }); } catch { /* gone */ }
}

async function pollMainTs(timeoutMs) {
  const t0 = Date.now();
  let last = 0;
  while (Date.now() - t0 < timeoutMs) {
    try {
      const res = await fetch(`${BASE}/src/main.ts`);
      last = res.status;
      if (res.status === 200) return { ok: true };
      await res.arrayBuffer().catch(() => {});
    } catch { /* not up */ }
    await new Promise(r => setTimeout(r, 1500));
  }
  return { ok: false, last };
}

const results = [];
const raw = {};
function record(case_id, ok, detail) {
  results.push({ case: case_id, ok: !!ok, detail: String(detail) });
  console.log(`${ok ? 'PASS' : 'FAIL'} ${case_id}: ${detail}`);
}

const chromium = await resolvePlaywright();
fs.mkdirSync(OUT, { recursive: true });
const autoBin = defaultAutoBin();
const srv = startAutoRun(autoBin);
let browser;
try {
  const up = await pollMainTs(300000); // 首跑含后端冷编译，窗口放大
  if (!up.ok) throw new Error(`dev server not ready (last ${up.last})`);
  console.log(`[+] dev server up @ ${BASE}`);

  browser = await chromium.launch({ headless: true, channel: 'msedge' });
  const context = await browser.newContext({ viewport: { width: 1360, height: 900 }, colorScheme: 'dark' });
  const page = await context.newPage();
  const pageErrors = [];
  page.on('pageerror', e => pageErrors.push(String(e)));
  await page.goto(BASE, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForSelector(MAIN, { timeout: 20000 });
  await page.waitForTimeout(900);

  const geom = (paneSel, key) => page.evaluate(([ps, k]) => {
    const pane = document.querySelector(ps);
    if (!pane) return null;
    const t = pane.querySelector(`[data-scroll-anchor="${k}"]`);
    if (!t) return { pane: true, target: null };
    const pr = pane.getBoundingClientRect();
    const tr = t.getBoundingClientRect();
    const ct = pr.top + pane.clientTop;
    const ix = Math.max(0, Math.min(pr.right, tr.right) - Math.max(pr.left, tr.left));
    const iy = Math.max(0, Math.min(pr.bottom, tr.bottom) - Math.max(ct, tr.top));
    return {
      pane: true, target: true,
      paneTop: ct, paneH: pane.clientHeight, scrollTop: pane.scrollTop,
      targetTop: tr.top, targetH: tr.height,
      ix, iy, winY: window.scrollY,
    };
  }, [paneSel, key]);

  const reply = () => page.evaluate(() => {
    const m = document.body.innerText.match(/R=(\S+)/);
    return m ? m[1] : null;
  });

  const clickAndWait = async (label, wantReceipt) => {
    // F-781-R02 勘定：带引号的 text= 是精确匹配——无引号形态是子串匹配
    //（'GoMI' 会命中更早的 'GoMiss'，W-02 mi 检查曾因此假红）。
    await page.click(`text="${label}"`);
    await page.waitForTimeout(300);
    if (!wantReceipt) return true;
    const r = await reply();
    return r === wantReceipt ? r : `${r} (want ${wantReceipt})`;
  };

  // ── V-03/V-01：Init(onMounted) 先发目标 —— c3 可见 ──
  let g = await geom(MAIN, 'c3');
  record('v03_init_target_visible', g && g.target && g.iy > 0 && g.ix > 0,
    g ? `iy=${g.iy.toFixed(1)} ix=${g.ix.toFixed(1)} scrollTop=${g.scrollTop}` : 'no target');
  const rQC3 = await clickAndWait('QC3', 'applied');
  record('v03_init_state_applied', rQC3 === 'applied', rQC3);

  // ── V-01：中文长段 c50（高于视口 → 顶对齐起始区域）──
  const q1 = await clickAndWait('GoC50', 'queued');
  record('v01_c50_queued', q1 === 'queued', q1);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'c50');
  raw.v01_c50 = g;
  record('v01_c50_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)} h=${g.targetH.toFixed(0)}/${g.paneH}` : 'no target');
  record('v01_c50_tall_top_aligned',
    g && g.target && g.targetH > g.paneH && Math.abs(g.targetTop - g.paneTop) <= 2,
    g && g.target ? `targetTop=${g.targetTop.toFixed(1)} paneTop=${g.paneTop.toFixed(1)} tall=${g.targetH.toFixed(0)}>${g.paneH}` : 'no target');

  // 英文长段 a80
  const q2 = await clickAndWait('GoA80', 'queued');
  record('v01_a80_queued', q2 === 'queued', q2);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'a80');
  raw.v01_a80 = g;
  record('v01_a80_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');

  // ── V-02：非连续 key 身份 ──
  const q3 = await clickAndWait('GoP8', 'queued');
  record('v02_p8_queued', q3 === 'queued', q3);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'p8');
  raw.v02_p8 = g;
  record('v02_p8_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');

  // ── V-04：负例族 ──
  const off = () => page.evaluate(ps => document.querySelector(ps).scrollTop, MAIN);
  const off_pre = await off();
  let r = await clickAndWait('GoMiss', 'queued');
  record('v04_missing_queued', r === 'queued', r);
  r = await clickAndWait('QMiss', 'invalid:not_found');
  record('v04_missing_state', r === 'invalid:not_found', r);
  r = await clickAndWait('GoOOB', 'queued');
  record('v04_oob_queued', r === 'queued', r);
  const off_now = await off();
  record('v04_invalid_no_scroll', off_now === off_pre, `${off_pre} -> ${off_now}`);
  r = await clickAndWait('GoEmptyK', 'error:empty_key');
  record('v04_empty_key', r === 'error:empty_key', r);
  r = await clickAndWait('GoBadH', 'error:bad_handle');
  record('v04_bad_handle', r === 'error:bad_handle', r);
  r = await clickAndWait('GoGhost', 'queued');
  record('v04_ghost_queued', r === 'queued', r);
  r = await clickAndWait('QGhost', 'invalid:not_bound');
  record('v04_ghost_not_bound', r === 'invalid:not_bound', r);

  // 同 pane 重复 → 文档序最后一个可见
  r = await clickAndWait('GoDup', 'queued');
  record('v04_dup_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  const dupGeom = await page.evaluate(ps => {
    const pane = document.querySelector(ps);
    const all = pane.querySelectorAll('[data-scroll-anchor="dup"]');
    const last = all[all.length - 1];
    if (!last) return null;
    const pr = pane.getBoundingClientRect();
    const tr = last.getBoundingClientRect();
    const iy = Math.max(0, Math.min(pr.bottom, tr.bottom) - Math.max(pr.top, tr.top));
    return { count: all.length, iy, text: last.innerText.slice(0, 24) };
  }, MAIN);
  raw.v04_dup = dupGeom;
  record('v04_dup_last_wins', dupGeom && dupGeom.count === 2 && dupGeom.iy > 0 &&
    dupGeom.text.includes('dup-B'), JSON.stringify(dupGeom));

  // 跨 pane 同 key：side pane 的 dup 滚动；main 不动
  const main_pre = await off();
  r = await clickAndWait('GoSideDup', 'queued');
  record('v04_side_dup_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geom(SIDE, 'dup');
  raw.v04_side_dup = g;
  record('v04_side_dup_visible', g && g.target && g.iy > 0, g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');
  const main_now2 = await off();
  record('v04_no_cross_pane_scroll', main_now2 === main_pre, `${main_pre} -> ${main_now2}`);
  r = await clickAndWait('GoSideOnly', 'queued');
  record('v04_side_only_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geom(SIDE, 'sideonly');
  record('v04_side_only_visible', g && g.target && g.iy > 0, g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');
  r = await clickAndWait('QUnknown', 'unknown');
  record('v04_unknown_state', r === 'unknown', r);

  // ── 阶段D W-01（F-781-R01）：标准 controller（onMounted 里
  // scroll_controller() → "__scrollctl_N"）+ 含冒号 key 真实定位。
  // pane 发现 = 含该锚点的 [data-scroll-ctl]（句柄值为运行期生成，不硬编码）。
  r = await clickAndWait('GoStd', 'queued');
  record('w01_std_handle_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  const stdGeom = await page.evaluate((key) => {
    const anchors = [...document.querySelectorAll(`[data-scroll-anchor="${key}"]`)];
    const t = anchors[anchors.length - 1];
    if (!t) return { target: false };
    const pane = t.closest('[data-scroll-ctl]');
    if (!pane) return { target: true, pane: false };
    const pr = pane.getBoundingClientRect();
    const tr = t.getBoundingClientRect();
    const ct = pr.top + pane.clientTop;
    const ix = Math.max(0, Math.min(pr.right, tr.right) - Math.max(pr.left, tr.left));
    const iy = Math.max(0, Math.min(pr.bottom, tr.bottom) - Math.max(ct, tr.top));
    return { target: true, pane: true, iy, ix, handle: pane.getAttribute('data-scroll-ctl') };
  }, 'std:colon-key');
  raw.w01_std = stdGeom;
  record('w01_std_colon_key_located',
    stdGeom && stdGeom.target && stdGeom.iy > 0 && stdGeom.ix > 0,
    JSON.stringify(stdGeom).slice(0, 140));
  r = await clickAndWait('QStd', 'applied');
  record('w01_std_state_applied', r === 'applied', r);

  // ── 阶段D W-02（F-781-R02）：嵌套 pane 最近归属——同名 key 零串线、
  // 仅内层有 key（含无 controller 嵌套 pane）→ 外层 not_found、内层
  // managed 不误伤外层。同名行在各自 pane 末尾（旧实现取内层目标则
  // 外层末行不可见——区分性锚定）。
  const NEST_O = '[data-scroll-ctl="nest-outer"]';
  const NEST_I = '[data-scroll-ctl="nest-inner"]';
  // 观测面按最近 pane 归属取目标（同名 key 在嵌套 pane 两侧都有）。
  const geomOwned = (paneSel, key) => page.evaluate(([ps, k]) => {
    const pane = document.querySelector(ps);
    if (!pane) return null;
    let t = null;
    for (const el of pane.querySelectorAll('[data-scroll-anchor]')) {
      if (el.getAttribute('data-scroll-anchor') === k
          && el.closest('[data-scroll-pane]') === pane) t = el;
    }
    if (!t) return { pane: true, target: null };
    const pr = pane.getBoundingClientRect();
    const tr = t.getBoundingClientRect();
    const ct = pr.top + pane.clientTop;
    const ix = Math.max(0, Math.min(pr.right, tr.right) - Math.max(pr.left, tr.left));
    const iy = Math.max(0, Math.min(pr.bottom, tr.bottom) - Math.max(ct, tr.top));
    return { pane: true, target: true, iy, ix, scrollTop: pane.scrollTop };
  }, [paneSel, key]);
  r = await clickAndWait('GoNestO', 'queued');
  record('w02_nest_outer_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geomOwned(NEST_O, 'nest-dup');
  raw.w02_outer = g;
  record('w02_nest_outer_same_key_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');
  const innerOff = await page.evaluate(ps => document.querySelector(ps).scrollTop, NEST_I);
  record('w02_nest_no_cross_scroll', innerOff === 0, `inner scrollTop=${innerOff}`);
  r = await clickAndWait('QNestO', 'applied');
  record('w02_nest_outer_state_applied', r === 'applied', r);
  r = await clickAndWait('GoNestI', 'queued');
  record('w02_nest_inner_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geomOwned(NEST_I, 'nest-dup');
  raw.w02_inner = g;
  record('w02_nest_inner_same_key_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');
  r = await clickAndWait('QNestI', 'applied');
  record('w02_nest_inner_state_applied', r === 'applied', r);
  r = await clickAndWait('GoInnerK', 'queued');
  record('w02_inner_only_queued', r === 'queued', r);
  r = await clickAndWait('QInnerK', 'invalid:not_found');
  record('w02_inner_only_outer_not_found', r === 'invalid:not_found', r);
  r = await clickAndWait('GoPlainK', 'queued');
  record('w02_plain_inner_queued', r === 'queued', r);
  r = await clickAndWait('QPlainK', 'invalid:not_found');
  record('w02_plain_inner_outer_not_found', r === 'invalid:not_found', r);
  r = await clickAndWait('GoMO', 'queued');
  record('w02_mo_outer_not_unsupported', r === 'queued', r);
  r = await clickAndWait('GoMI', 'error:unsupported');
  record('w02_mi_inner_unsupported', r === 'error:unsupported', r);

  // ── V-03：双章换绑（v-if 双 pane 分支）──
  await page.click('text="SwitchCh"');
  await page.waitForTimeout(700);
  const ch2 = await page.evaluate(ps => {
    const pane = document.querySelector(ps);
    return { hasC50: !!pane.querySelector('[data-scroll-anchor="c50"]'),
             hasQ5: !!pane.querySelector('[data-scroll-anchor="q5"]'),
             scrollTop: pane.scrollTop };
  }, MAIN);
  record('v03_ch2_swapped', ch2.hasC50 === false && ch2.hasQ5 === true,
    JSON.stringify(ch2));
  r = await clickAndWait('GoC50', 'queued');
  record('v03_old_key_queued', r === 'queued', r);
  await page.waitForTimeout(300);
  r = await clickAndWait('QC50', 'invalid:not_found');
  record('v03_old_key_not_found_on_ch2', r === 'invalid:not_found', r);
  const ch2_off = await off();
  r = await clickAndWait('GoQ5', 'queued');
  record('v03_new_key_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'q5');
  raw.v03_q5 = g;
  record('v03_new_key_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');
  r = await clickAndWait('QQ5', 'applied');
  record('v03_new_key_state_applied', r === 'applied', r);

  // ── V-05：重排（换回章1 → 字号 large → 窄宽）──
  await page.click('text="SwitchCh"');
  await page.waitForTimeout(700);
  await page.click('text="FontL"');
  await page.waitForTimeout(500);
  r = await clickAndWait('GoA80', 'queued');
  record('v05_large_a80_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'a80');
  raw.v05_a80_large = g;
  record('v05_large_a80_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)} h=${g.targetH.toFixed(0)}` : 'no target');
  await clickAndWait('GoC3', 'queued');
  await page.waitForTimeout(200);
  const off_a = await off();
  await clickAndWait('GoC3', 'queued');
  await page.waitForTimeout(400);
  const off_b = await off();
  record('v05_already_visible_stable', off_a === off_b, `${off_a} -> ${off_b}`);
  await page.click('text="SetNarrow"');
  await page.waitForTimeout(500);
  r = await clickAndWait('GoC50', 'queued');
  record('v05_narrow_c50_queued', r === 'queued', r);
  await page.waitForTimeout(200);
  g = await geom(MAIN, 'c50');
  raw.v05_c50_narrow = g;
  record('v05_narrow_c50_visible', g && g.target && g.ix > 0 && g.iy > 0,
    g && g.target ? `iy=${g.iy.toFixed(1)}` : 'no target');

  // ── V-06：旧 offset 族 + 不回拉（mouse.wheel 真滚动）+ managed ──
  await page.click('text="OldStart"');
  await page.waitForTimeout(300);
  const o0 = await off();
  record('v06_old_start_zero', o0 === 0, `${o0}`);
  await page.click('text="OldBy"');
  await page.waitForTimeout(300);
  const o1 = await off();
  record('v06_old_by_moves', Math.abs(o1 - 320) < 30, `${o1}`);
  await page.click('text="OldEnd"');
  await page.waitForTimeout(400);
  const o2 = await off();
  record('v06_old_end_large', o2 > o1, `${o2}`);
  r = await clickAndWait('GoC50', 'queued');
  await page.waitForTimeout(300);
  g = await geom(MAIN, 'c50');
  const anchor_off = await off();
  record('v06_anchor_applied_before_user', g && g.target && g.iy > 0,
    `off=${anchor_off}`);
  const box = await page.locator(MAIN).boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, -900);
  await page.waitForTimeout(600);
  const u1 = await off();
  await page.waitForTimeout(1000);
  const u2 = await off();
  record('v06_no_pullback_after_user_scroll',
    u1 !== anchor_off && u2 === u1, `anchor=${anchor_off} user=${u1} stable=${u2}`);
  r = await clickAndWait('GoMgd', 'error:unsupported');
  record('v06_managed_unsupported', r === 'error:unsupported', r);

  record('no_page_errors', pageErrors.length === 0, pageErrors.join(' | ') || 'zero');

  await page.screenshot({ path: path.join(OUT, 'vue_final.png'), fullPage: false });
  let cliHash = 'unavailable';
  try { cliHash = crypto.createHash('sha256').update(fs.readFileSync(autoBin)).digest('hex'); } catch { /* not file */ }
  fs.writeFileSync(path.join(OUT, 'vue_report.json'), JSON.stringify({
    leg: 'vue', url: BASE, auto_bin: autoBin, auto_bin_sha256: cliHash,
    results, raw,
  }, null, 2));

  const failed = results.filter(x => !x.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exitCode = failed.length ? 2 : 0;
} catch (e) {
  console.error('[-] driver error:', e);
  console.error(srv.getTail().slice(-3000));
  process.exitCode = 2;
} finally {
  if (browser) await browser.close().catch(() => {});
  await stopAutoRun(srv);
}
