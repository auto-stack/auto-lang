// PLAN-575 T2/T3 复现驱动：任务栏铃铛二次开合通知中心 → 桌面进程静默退出
// （KNOWN-DEBT 526）归因取证。desktop MCP JSON-RPC（vm-smoke 同款协议）。
//
// 每轮：启动异名副本 ui_desktop_p575.exe（--fullscreen，避开并行会话
// taskkill 按名清扫，049 定因对策）→ 等 MCP initialize → 找铃铛 →
// press ×2（二次开合；铃铛不可寻则走 autoui_desktop bus notes_toggle——
// 同一 records→DesktopCommand::NotesToggle 路径）→ 轮询进程存活 3s →
// 记一轮台账（JSONL）。进程死亡即抓审计文件尾 + 退出码。
//
// 用法：
//   node notes_toggle_repro.mjs --rounds 20 --exe <ui_desktop_p575.exe路径>
//       [--ledger <jsonl路径>] [--audit <审计log路径>] [--press-gap-ms 800]
// MCP 不可达（30s 内 initialize 不通）：记 mcp_unreachable 轮并杀进程；
// 连续 2 轮不可达则提示回退 t5_smoke 原生驱动并退出码 2。

import http from 'node:http';
import net from 'node:net';
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';

// ── CLI ────────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
function argOf(flag, dflt) {
  const i = argv.indexOf(flag);
  return i >= 0 && argv[i + 1] !== undefined ? argv[i + 1] : dflt;
}
const ROUNDS = parseInt(argOf('--rounds', '20'), 10);
const EXE = argOf('--exe', '');
const LEDGER = path.resolve(argOf('--ledger', 'ledger.jsonl'));
const AUDIT = path.resolve(argOf('--audit', 'exit-audit.log'));
const PRESS_GAP_MS = parseInt(argOf('--press-gap-ms', '800'), 10);
const MCP_WAIT_MS = 30_000;
const LIVENESS_POLL_MS = 3_000;

if (!EXE || !fs.existsSync(EXE)) {
  console.error(`[p575] --exe 必须指向异名副本（ui_desktop_p575.exe），未找到: ${EXE}`);
  process.exit(2);
}

// ── MCP JSON-RPC 客户端（HTTP POST /mcp，vm-smoke 同款）─────────────────────
function pickFreePort() {
  return new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.listen(0, '127.0.0.1', () => {
      const { port } = srv.address();
      srv.close(() => resolve(port));
    });
    srv.on('error', reject);
  });
}

function mcpCall(port, method, params, timeoutMs = 10_000) {
  const body = JSON.stringify({ jsonrpc: '2.0', id: 1, method, params });
  return new Promise((resolve, reject) => {
    const req = http.request(
      { host: '127.0.0.1', port, path: '/mcp', method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(body) } },
      (res) => {
        let data = '';
        res.on('data', (c) => (data += c));
        res.on('end', () => {
          try {
            const json = JSON.parse(data);
            if (json.error) reject(new Error(`MCP error: ${JSON.stringify(json.error)}`));
            else resolve(json.result ?? {});
          } catch (e) { reject(e); }
        });
      });
    req.on('error', reject);
    req.setTimeout(timeoutMs, () => req.destroy(new Error('MCP timeout')));
    req.end(body);
  });
}

async function mcpTool(port, name, args) {
  const res = await mcpCall(port, 'tools/call', { name, arguments: args ?? {} });
  return res?.content?.[0]?.text ?? JSON.stringify(res);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ── 审计 ────────────────────────────────────────────────────────────────────
function auditTail(nLines = 8) {
  try {
    const lines = fs.readFileSync(AUDIT, 'utf8').trimEnd().split('\n');
    return lines.slice(-nLines).join(' | ');
  } catch { return '(audit file unreadable)'; }
}

// ── 台账 ────────────────────────────────────────────────────────────────────
function appendLedger(entry) {
  fs.appendFileSync(LEDGER, JSON.stringify(entry) + '\n');
  console.log(`[p575] R${entry.round}: ${entry.verdict} alive=${entry.alive_after_3s} exit=${entry.exit_code ?? '-'} mech=${entry.mechanism} bell=${entry.bell_found}`);
}

// ── 铃铛定位 + 二次开合 ─────────────────────────────────────────────────────
async function findBell(port) {
  try {
    const text = await mcpTool(port, 'autoui_find', { kind: 'button', label: 'bell', limit: 5 });
    const m = text.match(/Found (\d+) node\(s\)/);
    if (m && parseInt(m[1], 10) > 0) {
      const ids = [...text.matchAll(/vnode_(\d+)/g)].map((x) => x[1]);
      return { id: ids[ids.length - 1], text: text.slice(0, 400) };
    }
  } catch { /* fallthrough */ }
  return null;
}

async function toggleTwice(port, bell) {
  if (bell) {
    for (const el of [bell.id, bell.id]) {
      const t = await mcpTool(port, 'autoui_action', { element_id: el, action: 'press' });
      if (/error|fail|No live|not found/i.test(t)) return { ok: false, why: `press#${t}` };
      await sleep(PRESS_GAP_MS);
    }
    return { ok: true, why: 'bell press x2' };
  }
  // 回退：bus 直注 notes_toggle 记录——真实铃铛点击的同一条
  // records → DesktopCommand::NotesToggle 路径（acceptance 通道）。
  for (let i = 0; i < 2; i++) {
    const t = await mcpTool(port, 'autoui_desktop', { action: 'bus', verb: 'notes_toggle' });
    if (/error/i.test(t)) return { ok: false, why: `bus#${t}` };
    await sleep(PRESS_GAP_MS);
  }
  return { ok: true, why: 'bus notes_toggle x2' };
}

// ── 单轮 ────────────────────────────────────────────────────────────────────
let consecutiveUnreachable = 0;
async function runRound(round) {
  const port = await pickFreePort();
  const stderrPath = path.join(path.dirname(LEDGER), `round-${round}-stderr.log`);
  const stderrFd = fs.openSync(stderrPath, 'a');
  const child = spawn(EXE, ['--fullscreen'], {
    cwd: path.dirname(LEDGER), // 截图等运行期产物落台账同目录
    env: {
      ...process.env,
      AUTOUI_MCP_PORT: String(port),
      AUTO_DESKTOP_EXIT_LOG: AUDIT,
      AUTOUI_ACCEPTANCE: '1',
      RUST_BACKTRACE: 'full',
    },
    stdio: ['ignore', 'ignore', stderrFd],
  });
  const pid = child.pid;
  const base = { round, ts: Date.now(), pid, port };

  // 等 MCP 就绪
  const deadline = Date.now() + MCP_WAIT_MS;
  let ready = false;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) break;
    try { await mcpCall(port, 'initialize', {}); ready = true; break; }
    catch { await sleep(500); }
  }
  if (!ready) {
    const wasAlive = child.exitCode === null;
    if (wasAlive) child.kill();
    await sleep(500);
    consecutiveUnreachable++;
    appendLedger({ ...base, verdict: 'mcp_unreachable', died_before_mcp: !wasAlive,
      exit_code: child.exitCode, audit_tail: auditTail() });
    return consecutiveUnreachable >= 2;
  }
  consecutiveUnreachable = 0;

  // 找铃铛（记账用——宿主 dock 不在组件 VTree，预期 NOT FOUND）→
  // bus 直注 notes_toggle ×2（真实铃铛点击的同一条
  // records → DesktopCommand::NotesToggle → toggle_notification_center 路径）。
  let bell = null;
  try { bell = await findBell(port); } catch { /* fallthrough */ }
  const mechanism = bell ? 'bell_press' : 'bus_inject';
  let pressed;
  try { pressed = await toggleTwice(port, bell); }
  catch (e) { pressed = { ok: false, why: String(e) }; }
  fs.closeSync(stderrFd);

  // toggle 执行证据：stderr 中 NotificationCenter Init/RebuildNotes handler 行
  let toggleEvidence = '(none)';
  try {
    const err = fs.readFileSync(stderrPath, 'utf8');
    const hits = err.split('\n').filter((l) => l.includes('NotificationCenter'));
    toggleEvidence = hits.length ? `${hits.length} handler lines (last: ${hits[hits.length - 1].trim()})` : '(no NotificationCenter lines)';
  } catch { /* keep default */ }

  // 存活轮询 3s
  const t0 = Date.now();
  while (Date.now() - t0 < LIVENESS_POLL_MS && child.exitCode === null) await sleep(200);
  const alive = child.exitCode === null;
  const exitCode = child.exitCode;
  let verdict;
  if (!pressed.ok) verdict = 'action_failed';
  else if (!alive) verdict = 'EXITED'; // 归因关键轮：抓审计
  else verdict = 'alive';

  appendLedger({ ...base, verdict, mechanism, bell_found: !!bell,
    press_detail: pressed.why, toggle_evidence: toggleEvidence,
    alive_after_3s: alive, exit_code: exitCode,
    audit_tail: verdict === 'EXITED' ? auditTail() : undefined });

  if (alive) { child.kill(); await sleep(800); }
  return false;
}

// ── 主循环 ──────────────────────────────────────────────────────────────────
(async () => {
  fs.writeFileSync(LEDGER, '');
  console.log(`[p575] rounds=${ROUNDS} exe=${EXE}\n[p575] ledger=${LEDGER}\n[p575] audit=${AUDIT}`);
  let exited = 0;
  for (let r = 1; r <= ROUNDS; r++) {
    const stop = await runRound(r);
    if (stop) {
      console.error(`[p575] 连续 2 轮 MCP 不可达——回退路径：实机手跑 t5_smoke 原生驱动` +
        `（cargo test -p auto-lang --features test-native-dock --test t5_smoke -- --ignored --nocapture）后人工核对。`);
      process.exit(2);
    }
    await sleep(1000);
  }
  const lines = fs.readFileSync(LEDGER, 'utf8').trim().split('\n').map((l) => JSON.parse(l));
  exited = lines.filter((l) => l.verdict === 'EXITED').length;
  console.log(`\n[p575] 完成 ${ROUNDS} 轮：EXITED=${exited} alive=${lines.filter((l) => l.verdict === 'alive').length}` +
    ` other=${lines.filter((l) => !['EXITED', 'alive'].includes(l.verdict)).length}`);
})();
