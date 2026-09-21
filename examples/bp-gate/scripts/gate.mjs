#!/usr/bin/env node
// gate.mjs — bp-gate 蓝图级双端 gate 编排（PLAN-075 T-03；jade gallery
// gate.mjs 双臂模式复刻 + R-B 沙箱裁定）。
//
//   node scripts/gate.mjs [--arm vue|vm|all] [--update-snapshots] [--keep-sandbox]
//
// vue 臂：沙箱准备（复制 blueprints+host → 仓库外临时目录）→ auto build
//         --gen-only（junction 只落沙箱）→ G-7 补件（vite-env.d.ts +
//         auto-sources.ts，074-sink-mode G-7 环境缺口）→ pnpm install →
//         vue-tsc → vite build → serve dist → playwright（断言+截图基线，
//         --update-snapshots 透传刷新）。
// vm 臂：node scripts/vm-probe.mjs（沙箱 host 内 auto run -r vm 单 boot，
//         MCP autoui_state/snapshot 断言，units.mjs 配置驱动）。
// 汇总每臂 PASS/FAIL，任一红即 exit 1。沙箱退出路径保证清理（deps 下
// junction 先 rmdir 摘链再整树删除——rmSync 不穿透）。

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const GATE = path.resolve(here, '..')
const REPO = path.resolve(GATE, '../..')
const AUTO_EXE = process.env.AUTO_EXE ?? path.join(REPO, 'target/debug/auto.exe')

const args = process.argv.slice(2)
const argOf = (name) => {
  const i = args.indexOf(name)
  return i >= 0 ? args[i + 1] : undefined
}
const has = (name) => args.includes(name)
const ARM = argOf('--arm') ?? 'all'
const UPDATE = has('--update-snapshots')
const KEEP = has('--keep-sandbox')

const run = (cmd, cmdArgs, label, cwd) => {
  console.log(`\n=== gate[${label}]: ${cmd} ${cmdArgs.join(' ')}`)
  const r = spawnSync(cmd, cmdArgs, { cwd: cwd ?? GATE, stdio: 'inherit', shell: process.platform === 'win32' })
  if (r.error) throw r.error
  return r.status === 0
}

function copyDir(src, dst) {
  fs.mkdirSync(dst, { recursive: true })
  for (const e of fs.readdirSync(src, { withFileTypes: true })) {
    if (e.name === 'node_modules' || e.name === 'deps' || e.name === 'gen') continue
    const from = path.join(src, e.name)
    const to = path.join(dst, e.name)
    if (e.isDirectory()) copyDir(from, to)
    else fs.copyFileSync(from, to)
  }
}

// 沙箱：host 落 sandbox/a/b/host（三级深度对齐 pac.at 的 ../../../blueprints）
function makeSandbox() {
  const sandbox = fs.mkdtempSync(path.join(os.tmpdir(), 'bp-gate-'))
  copyDir(path.join(REPO, 'blueprints'), path.join(sandbox, 'blueprints'))
  copyDir(path.join(GATE, 'host'), path.join(sandbox, 'a', 'b', 'host'))
  return sandbox
}

function removeSandbox(sandbox) {
  // deps/ 下是 auto build 物化的 junction——先摘链接本身（rmdir 不穿透），
  // 再整树删除。VM 进程退出与清理存在竞态（taskkill 异步，EPERM 实测），
  // 带短重试。
  const deps = path.join(sandbox, 'a', 'b', 'host', 'deps')
  if (fs.existsSync(deps)) {
    for (const name of fs.readdirSync(deps)) {
      try { fs.rmdirSync(path.join(deps, name)) } catch { /* 非链接走整树删除 */ }
    }
  }
  for (let i = 0; i < 3; i++) {
    try {
      fs.rmSync(sandbox, { recursive: true, force: true })
      return
    } catch (e) {
      if (i === 2) throw e
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 800)
    }
  }
}

// G-7 环境件（074-sink-mode）：auto build gen 腿不落两文件，vue-tsc 必炸；
// gate 侧补最小 shim（auto run 会在真运行时改写 auto-sources）。
function patchG7(genVueDir) {
  fs.writeFileSync(
    path.join(genVueDir, 'src', 'vite-env.d.ts'),
    '/// <reference types="vite/client" />\n',
  )
  fs.writeFileSync(
    path.join(genVueDir, 'src', 'auto-sources.ts'),
    '// G-7 env shim — rewritten by `auto run` (content-hash debounced).\nexport const AUTO_SOURCES: Record<string, string> = {}\n',
  )
}

const results = {}
let sandbox
try {
  if (!fs.existsSync(AUTO_EXE)) {
    console.error(`auto.exe not found: ${AUTO_EXE} — run \`cargo build -p auto\` first`)
    process.exit(1)
  }
  sandbox = makeSandbox()
  const host = path.join(sandbox, 'a', 'b', 'host')
  console.log(`sandbox: ${sandbox}`)

  if (ARM === 'all' || ARM === 'vue') {
    const genOk = run(AUTO_EXE, ['build', '--gen-only'], 'vue:gen', host)
    let tscOk = false
    let viteOk = false
    let pwOk = false
    if (genOk) {
      const genVue = path.join(host, 'gen', 'front', 'vue')
      patchG7(genVue)
      // PLAN-676：gen-only 现已跳过 npm 步（log: "npm steps skipped"）——
      // 头注流程的 pnpm install 步在代码里补回，否则 vue-tsc 无 bin 可寻。
      const instOk = run('pnpm', ['install', '--prefer-offline'], 'vue:install', genVue)
      tscOk = instOk && run('pnpm', ['exec', 'vue-tsc'], 'vue:tsc', genVue)
      viteOk = tscOk && run('pnpm', ['exec', 'vite', 'build'], 'vue:vite', genVue)
      if (viteOk) {
        const pwArgs = ['exec', 'playwright', 'test']
        if (UPDATE) pwArgs.push('--update-snapshots')
        const env = { ...process.env, BP_GATE_DIST: path.join(genVue, 'dist') }
        const r = spawnSync('pnpm', pwArgs, { cwd: GATE, env, stdio: 'inherit', shell: process.platform === 'win32' })
        pwOk = r.status === 0
      }
    }
    results['vue'] = genOk && tscOk && viteOk && pwOk
  }
  if (ARM === 'all' || ARM === 'vm') {
    const env = { ...process.env, BP_GATE_SANDBOX_HOST: host, AUTO_EXE }
    const r = spawnSync('node', ['scripts/vm-probe.mjs'], { cwd: GATE, env, stdio: 'inherit', shell: process.platform === 'win32' })
    results['vm'] = r.status === 0
  }
} finally {
  if (sandbox && !KEEP) {
    try { removeSandbox(sandbox) } catch (e) { console.error(`sandbox cleanup failed: ${e.message}`) }
  } else if (sandbox) {
    console.log(`sandbox kept: ${sandbox}`)
  }
}

console.log('\n=== bp-gate summary ===')
let failed = false
for (const [arm, ok] of Object.entries(results)) {
  console.log(`  ${ok ? '✓' : '✗'} ${arm}`)
  if (!ok) failed = true
}
process.exit(failed ? 1 : 0)
