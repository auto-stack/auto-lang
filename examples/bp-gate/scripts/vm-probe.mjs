#!/usr/bin/env node
// vm-probe.mjs — bp-gate VM 臂 gate（PLAN-075 T-03；jade gallery vm-probe.mjs
// 模式裁剪：宿主无切换单按钮，单 boot 直断）。
//
// 协议沿 jade-garden vm-smoke（AutoUI MCP over Streamable HTTP；
// autoui_state / autoui_snapshot）。断言面来自 scripts/units.mjs：
//   state —— root 投影字段等值（F-1：bp 子件子树对快照不可见，内容断言
//            不依赖子树；R-C 裁定的投影形态）。
//   snapshot —— root 单元标记行 needle（"unit: <id>"）。
// 宿主目录由 gate.mjs 经 BP_GATE_SANDBOX_HOST 注入（沙箱内工程）。
// 卫生（vm-smoke 纪律）：只杀本脚本 spawn 的 PID；退出路径保证 kill。

import { spawn } from 'node:child_process'
import net from 'node:net'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { UNITS } from './units.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const HOST_DIR = process.env.BP_GATE_SANDBOX_HOST
const AUTO_EXE = process.env.AUTO_EXE ?? 'D:/autostack/auto-lang/target/debug/auto.exe'

if (!HOST_DIR) {
  console.error('BP_GATE_SANDBOX_HOST not set — run via scripts/gate.mjs')
  process.exit(1)
}

const args = process.argv.slice(2)
const argOf = (name) => {
  const i = args.indexOf(name)
  return i >= 0 ? args[i + 1] : undefined
}
const BASE_PORT = Number(argOf('--port') ?? process.env.AUTOUI_MCP_PORT ?? 9331)
const ONLY = argOf('--unit')?.split(',').map((s) => s.trim()).filter(Boolean)
const UNITS_RUN = ONLY ? UNITS.filter((u) => ONLY.includes(u.id)) : UNITS

const sleep = (ms) => new Promise((r) => setTimeout(r, ms))
const freePort = (port) =>
  new Promise((resolve) => {
    const srv = net.createServer()
    srv.once('error', () => resolve(false))
    srv.once('listening', () => srv.close(() => resolve(true)))
    srv.listen(port, '127.0.0.1')
  })

let port = BASE_PORT
for (let i = 0; i < 10 && !(await freePort(port)); i++) port++

// ---------------- MCP client ----------------
let nextId = 1
const mcpBase = () => `http://127.0.0.1:${port}/mcp`

async function rpc(method, params) {
  const res = await fetch(mcpBase(), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: nextId++, method, params }),
  })
  if (!res.ok) throw new Error(`MCP ${method} -> HTTP ${res.status}`)
  const body = await res.json()
  if (body.error) throw new Error(`MCP ${method} error: ${JSON.stringify(body.error)}`)
  return body.result
}

async function notify(method) {
  const res = await fetch(mcpBase(), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', method }),
  })
  if (!res.ok) throw new Error(`MCP ${method} notify -> HTTP ${res.status}`)
}

async function callTool(name, toolArgs) {
  const result = await rpc('tools/call', { name, arguments: toolArgs })
  if (result.isError) throw new Error(`tool ${name} failed: ${JSON.stringify(result.content)}`)
  return result.content.map((c) => c.text ?? '').join('\n')
}

async function waitForServer(timeoutMs) {
  const deadline = Date.now() + timeoutMs
  for (;;) {
    try {
      await rpc('initialize', {
        protocolVersion: '2025-03-26',
        capabilities: {},
        clientInfo: { name: 'bp-gate-vm', version: '0.1.0' },
      })
      await notify('notifications/initialized')
      return
    } catch (err) {
      if (Date.now() > deadline) throw new Error(`AutoUI MCP not reachable on ${mcpBase()}: ${err.message}`)
      await sleep(500)
    }
  }
}

// ---------------- snapshot helpers（vm-smoke 先例）----------------
function parseAura(text) {
  const root = { head: '<root>', children: [] }
  const stack = [{ node: root, depth: -1 }]
  for (const raw of text.split('\n')) {
    const trimmed = raw.trim()
    if (!trimmed || trimmed === '}' || raw.startsWith('AURA') || raw.startsWith('widget:') || raw.startsWith('tree:')) continue
    const depth = Math.floor((raw.length - raw.replace(/^ */, '').length) / 2)
    const line = raw.trim().replace(/\{$/, '')
    while (stack.length > 1 && stack[stack.length - 1].depth >= depth) stack.pop()
    const node = { head: line, children: [] }
    stack[stack.length - 1].node.children.push(node)
    stack.push({ node, depth })
  }
  return root
}

function findFirst(node, pred) {
  if (pred(node)) return node
  for (const child of node.children) {
    const hit = findFirst(child, pred)
    if (hit) return hit
  }
  return null
}

function ownText(node) {
  const m = node.head.match(/"((?:[^"\\]|\\.)*)"/)
  return m ? m[1] : ''
}

async function snapshot() {
  return parseAura(await callTool('autoui_snapshot', {}))
}

async function stateIs(field, want, timeoutMs = 8000) {
  for (const deadline = Date.now() + timeoutMs; ;) {
    const st = await callTool('autoui_state', { fields: [field] })
    const raw = st.match(new RegExp(`${field}:\\s*(.*)`))?.[1]?.trim() ?? ''
    const got = raw
      .replace(/\s*\((?:str|int|bool|float|map|list)\)\s*$/, '')
      .replace(/^"((?:[^"\\]|\\.)*)"$/, '$1')
      .trim()
    if (got === want) return st.trim()
    if (Date.now() > deadline) throw new Error(`state.${field} never became "${want}" (got: ${raw || st.trim()})`)
    await sleep(120)
  }
}

// ---------------- run ----------------
const env = { ...process.env, AUTOUI_MCP_PORT: String(port) }
const child = spawn(AUTO_EXE, ['run', '-r', 'vm'], { cwd: HOST_DIR, env, stdio: ['ignore', 'pipe', 'pipe'] })
const kill = () => {
  try {
    if (child.pid) spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'])
  } catch {}
}
process.on('exit', kill)
process.on('SIGINT', () => {
  kill()
  process.exit(1)
})
child.stderr.on('data', (d) => process.stderr.write(`[vm] ${d}`))

const checks = []
try {
  await waitForServer(30_000)
  checks.push(`vm boot: auto.exe run -r vm (cwd=${HOST_DIR}, MCP :${port})`)

  for (const u of UNITS_RUN) {
    for (const [field, want] of Object.entries(u.vm.state ?? {})) {
      await stateIs(field, want)
    }
    if (Object.keys(u.vm.state ?? {}).length) checks.push(`${u.id}: state ${JSON.stringify(u.vm.state)}`)
    if (u.vm.snapshot?.length) {
      const tree = await snapshot()
      for (const needle of u.vm.snapshot) {
        const hit = findFirst(tree, (n) => ownText(n).includes(needle))
        if (!hit) throw new Error(`${u.id}: snapshot 无 "${needle}"`)
      }
      checks.push(`${u.id}: snapshot needles [${u.vm.snapshot.join(', ')}]`)
    }
  }

  console.log('=== bp-gate VM arm PASS ===')
  for (const c of checks) console.log('  ✓ ' + c)
  process.exit(0)
} catch (err) {
  console.error('=== bp-gate VM arm FAIL ===')
  console.error(err.message)
  process.exit(1)
} finally {
  kill()
}
