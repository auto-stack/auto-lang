// PLAN-761: tracked b3-era corpus lives at test/fixtures/plan446/b3check/auto.
// Override AUTO_BIN/P446_CORPUS_ROOT to compare another build/export.
import { spawn, execSync } from 'child_process';
import { setTimeout as sleep } from 'timers/promises';
import { reproContext } from './context.mjs';

const { port: PORT, binary, corpus } = reproContext('9385');
const MCP = `http://127.0.0.1:${PORT}/mcp`;

const proc = spawn(binary, ['run', ['-r', 'vm']].flat(), {
  cwd: corpus,
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

function foreground() {
  try {
    execSync(`powershell -NoProfile -Command "$p = Get-Process auto -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq 'Auto - App' } | Select-Object -First 1; if ($p) { Add-Type 'using System; using System.Runtime.InteropServices; public class FG { [DllImport(\\"user32.dll\\")] public static extern bool SetForegroundWindow(IntPtr h); [DllImport(\\"user32.dll\\")] public static extern bool ShowWindow(IntPtr h, int c); }'; [FG]::ShowWindow($p.MainWindowHandle, 9) | Out-Null; [FG]::SetForegroundWindow($p.MainWindowHandle) | Out-Null }"`, { stdio: 'ignore' });
  } catch {}
}

async function waitFor(pred, tries = 25, ms = 1500) {
  for (let i = 0; i < tries; i++) { if (await pred()) return true; foreground(); await sleep(ms); }
  return false;
}

async function pressLoose(label) {
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
    if (l.includes('button') && l.includes(label)) {
      const b = l.match(/(vnode_\d+)/);
      if (b) { await call('autoui_action', { element_id: b[0], action: 'press' }); return true; }
    }
  }
  return false;
}

await waitFor(async () => (await snap()).includes('Roles'));
console.log('roles:', await pressLoose('Roles')); await sleep(2500); foreground();
console.log('load:', await waitFor(async () => await pressLoose('Load'), 8)); await sleep(3500); foreground();
console.log('assistant:', await waitFor(async () => await pressLoose('assistant'), 8)); await sleep(5000); foreground();

const t = await snap();
const lines = t.split('\n');
const idx = lines.findIndex((l) => l.startsWith('tree:'));
const tree = lines.slice(idx);
const applies = tree.filter((l) => l.includes('button') && l.includes('Apply')).length;
const inputs = tree.filter((l) => l.includes('input #')).length;
console.log(`tree lines: ${tree.length} | inputs: ${inputs} | Apply buttons: ${applies}`);
proc.kill();
