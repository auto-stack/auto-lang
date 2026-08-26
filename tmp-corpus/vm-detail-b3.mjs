// tmp/vm-detail-dump.mjs — dump the full detail-region tree after selecting
// assistant (J1 flattening iteration: which subtrees build, which vanish).
import { spawn, execSync } from 'child_process';
import { setTimeout as sleep } from 'timers/promises';

const PORT = process.argv[2] || '9378';
const MCP = `http://127.0.0.1:${PORT}/mcp`;

const proc = spawn('D:/autostack/auto-lang/.worktree/plan-446/target/debug/auto.exe', ['run', '-r', 'vm'], {
  cwd: new URL('./b3check/auto/', import.meta.url),
  env: { ...process.env, AUTOUI_MCP_PORT: PORT },
  stdio: 'ignore',
});
process.on('exit', () => { try { proc.kill(); } catch {} });

const call = async (n, a) => {
  try {
    const r = await fetch(MCP, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: n, arguments: a } }),
      signal: AbortSignal.timeout(8000),
    });
    return (await r.json()).result?.content?.[0]?.text ?? '';
  } catch { return null; }
};
const snap = async () => (await call('autoui_snapshot', { include_state: false })) ?? '';

async function waitFor(pred, tries = 20, ms = 1500) {
  for (let i = 0; i < tries; i++) { if (await pred()) return true; await sleep(ms); }
  return false;
}

try {
  execSync(`powershell -NoProfile -Command "$p = Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq 'Auto - App' } | Select-Object -First 1; if ($p) { Add-Type 'using System; using System.Runtime.InteropServices; public class FG { [DllImport(\\"user32.dll\\")] public static extern bool SetForegroundWindow(IntPtr h); [DllImport(\\"user32.dll\\")] public static extern bool SetForegroundWindow(IntPtr h); [DllImport(\\"user32.dll\\")] public static extern bool ShowWindow(IntPtr h, int c); }'; [FG]::ShowWindow($p.MainWindowHandle, 9) | Out-Null; [FG]::SetForegroundWindow($p.MainWindowHandle) | Out-Null }"`, { stdio: 'ignore' });
} catch {}

async function pressText(label) {
  const lines = (await snap()).split('\n');
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/text #(vnode_\d+) "(.+)"$/);
    if (m && m[2] === label) {
      for (let j = i; j >= 0; j--) {
        const b = lines[j].match(/button #(vnode_\d+)/);
        if (b) { await call('autoui_action', { element_id: b[1], action: 'press' }); return true; }
      }
    }
  }
  for (const l of lines) {
    if (l.includes('button') && l.includes(`"${label}"`)) {
      const b = l.match(/(vnode_\d+)/);
      if (b) { await call('autoui_action', { element_id: b[0], action: 'press' }); return true; }
    }
  }
  return false;
}

await waitFor(async () => (await snap()).includes('AutoOS Settings'));
await waitFor(async () => await pressText('Roles')); await sleep(2500);
await waitFor(async () => await pressText('Load')); await sleep(3500);
await waitFor(async () => await pressText('assistant')); await sleep(5000);

const t = await snap();
const lines = t.split('\n');
const idx = lines.findIndex((l) => l.startsWith('tree:'));
const tree = lines.slice(idx);
console.log(`tree lines: ${tree.length}`);
// print everything from the first 'input #' onward, truncated per line
const firstInput = tree.findIndex((l) => l.includes('input #'));
const start = Math.max(0, firstInput - 12);
console.log(`first input at tree line ${firstInput}; dumping ${start}..${tree.length}`);
console.log(tree.slice(start).map((l) => l.slice(0, 118)).join('\n'));
proc.kill();
