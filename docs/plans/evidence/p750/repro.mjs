// p750 repro.mjs — PLAN-750 T-01 仪器化复现（vue 轨草稿结算竞态）
// 用法：node repro.mjs <scenario> [autoExe]
//   scenario: A=双快速输入轮（重入探针） B=matrix 忠实 [4edit]+[5save] C=输入后立即保存
//   autoExe:  缺省主检出 target/debug/auto.exe（M1 失败工件）
// 依赖：jade-edit 只读引用（serveBackend 库 + @playwright/test + gen/front/vue）
// 产出：每场景 API 调用时间线（fetch 钩子）+ .jade/drafts/v1 目录形态 + RTT 采样
import { createRequire } from 'node:module'
import { spawn, execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

const JADE = 'D:/autostack/jade-edit'
const AUTO_EXE = process.env.AUTO_EXE ?? process.argv[3] ?? 'D:/autostack/auto-lang/target/debug/auto.exe'
const SCENARIO = (process.argv[2] ?? 'A').toUpperCase()
const BACK_PORT = Number(process.env.P750_BACK ?? 8919)
const FRONT_PORT = Number(process.env.P750_FRONT ?? 4553)
const WS = `${JADE}/e2e/.runtime/p750-ws-${SCENARIO}`

const require = createRequire(`${JADE}/package.json`)
const { chromium } = require('@playwright/test')
const { serveBackend } = await import(`file:///${JADE}/scripts/serve-back.mjs`)

const log = (...a) => console.log(`[p750-${SCENARIO}]`, ...a)
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

function dumpDrafts(ws) {
  const root = path.join(ws, '.auto-jade', 'drafts', 'v1')
  if (!fs.existsSync(root)) return '(no drafts dir)'
  const out = []
  for (const d of fs.readdirSync(root).sort()) {
    const dir = path.join(root, d)
    const files = fs.readdirSync(dir)
    const parts = files.map((f) => {
      if (!f.endsWith('.meta')) return f
      const meta = fs.readFileSync(path.join(dir, f), 'utf8')
      const ev = /event=(\w+)/.exec(meta)?.[1] ?? '?'
      const erev = /(?:^|[^_])erev=(\d+)/m.exec(meta)?.[1] ?? /settle_erev=(\d+)/.exec(meta)?.[1] ?? '?'
      return `${f}[event=${ev},erev=${erev}]`
    })
    out.push(`  ${d}/: ${parts.join(' ') || '(EMPTY)'}`)
  }
  return out.length ? out.join('\n') : '(v1 empty)'
}

async function rtSample(url, n) {
  const ts = []
  for (let i = 0; i < n; i++) {
    const t0 = performance.now()
    await fetch(url)
    ts.push(performance.now() - t0)
  }
  ts.sort((a, b) => a - b)
  const q = (p) => Math.round(ts[Math.floor((ts.length - 1) * p)])
  return `n=${ts.length} min=${q(0)} p50=${q(0.5)} p90=${q(0.9)} max=${q(1)} (ms)`
}

// ---- 启动后端（fixture 隔离拷贝 + belt）----
const back = await serveBackend({ port: BACK_PORT, workspace: WS })
log(`backend up (exe=${AUTO_EXE}) ws=${WS}`)

// ---- 启动 vite 前端（jade gen/front/vue 现产物）----
const vite = spawn(process.execPath, ['node_modules/vite/bin/vite.js', '--strictPort', '--host', '127.0.0.1'], {
  cwd: `${JADE}/gen/front/vue`,
  env: { ...process.env, AUTO_HTTP_PORT: String(BACK_PORT), AUTO_FRONT_PORT: String(FRONT_PORT), TAURI_ENV: '1' },
  stdio: ['ignore', 'pipe', 'pipe'],
})
let viteOut = ''
vite.stdout.on('data', (d) => (viteOut += d))
vite.stderr.on('data', (d) => (viteOut += d))
for (let i = 0; i < 60; i++) {
  try { await fetch(`http://127.0.0.1:${FRONT_PORT}`); break } catch { await sleep(500) }
}
log('vite up')

// ---- RTT 采样（draft_begin=只读盘扫，可安全重复）----
log('draft_begin RTT:', await rtSample(`http://127.0.0.1:${BACK_PORT}/api/draft_begin`, 20))

// ---- 浏览器 + fetch 钩子 ----
const browser = await chromium.launch()
const ctx = await browser.newContext()
await ctx.addInitScript(`
  window.__p750 = []
  const orig = window.fetch
  window.fetch = async (...args) => {
    const url = String(args[0])
    const t0 = performance.now()
    const res = await orig(...args)
    if (url.includes('/api/')) {
      let body = ''
      try { body = (await res.clone().text()).slice(0, 80) } catch {}
      let req = ''
      try { req = String(args[1] && args[1].body ? args[1].body : '').slice(0, 1200) } catch {}
      window.__p750.push({ url: url.replace(/^https?:\\/\\/[^/]+/, ''), at: Math.round(t0), ms: Math.round(performance.now() - t0), st: res.status, body, req })
    }
    return res
  }
`)
const page = await ctx.newPage()
try {
  await page.goto(`http://127.0.0.1:${FRONT_PORT}`)
  await page.getByText('ready', { exact: true }).first().waitFor({ timeout: 20000 })
  await page.getByText('wiki', { exact: true }).first().click()
  // 开 Hello World（matrix [3 open] 同锚）
  await page.getByText('Hello World', { exact: true }).first().click()
  const content = page.locator('.autodown-editor:visible .autodown-editor-content').first()
  await content.waitFor({ timeout: 15000 })
  await content.click()
  await page.keyboard.press('Control+End')
  const neutralBlur = async () => {
    await page.getByText('EXPLORER', { exact: true }).click()
  }

  if (SCENARIO === 'A') {
    // 两轮快速输入（insertText + blur），轮间零等待——重入窗探针
    // （每轮必须重进编辑器定位焦点：blur 后焦点已移出）
    const round = async (ch) => {
      await content.click()
      await page.keyboard.press('Control+End')
      await page.keyboard.insertText(ch)
      await neutralBlur()
    }
    await round('A')
    await round('B')
    log('typed A/blur + B/blur back-to-back; waiting 3s for drain…')
    await sleep(3000)
  } else if (SCENARIO === 'B') {
    // matrix 忠实 [4 edit]→[5 save]：insertText + poll 渲染 + 保存点击
    await page.keyboard.insertText('p750-B-marker')
    await page.locator('.autodown-editor:visible').first().waitFor({ timeout: 10000 })
    await neutralBlur()
    await page.getByText('未保存', { exact: true }).first().waitFor({ timeout: 10000 })
    await page.locator('button[title="保存"]').click()
    log('insertText + dirty poll + save click; waiting 3s…')
    await sleep(3000)
  } else if (SCENARIO === 'C') {
    // 输入后立即保存（无 poll 间隔）——保存 vs 排水在途竞态探针
    await page.keyboard.insertText('p750-C-marker')
    await neutralBlur()
    await page.locator('button[title="保存"]').click()
    log('insertText + blur + immediate save; waiting 3s…')
    await sleep(3000)
  } else if (SCENARIO === 'D') {
    // 逐键 0 延迟（若首挂载逐键发射存活 → 两 Edit 0-5ms 连发入分配悬窗）
    await page.keyboard.type('XY', { delay: 0 })
    await neutralBlur()
    log('typed XY delay=0 + blur; waiting 3s…')
    await sleep(3000)
  } else if (SCENARIO === 'E') {
    // matrix 忠实逐键 delay=25（负载下窗口>25ms 时的真实触发形态）
    await page.keyboard.type('XY', { delay: 25 })
    await neutralBlur()
    log('typed XY delay=25 + blur; waiting 3s…')
    await sleep(3000)
  }

  const calls = await page.evaluate(() => window.__p750)
  log('=== API timeline (draft/api 视角) ===')
  for (const c of calls.filter((c) => /draft|write_wiki|read_wiki/.test(c.url))) {
    log(`  +${String(c.at).padStart(6)}ms ${c.ms}ms st=${c.st} ${c.url} -> ${JSON.stringify(c.body)}`)
    if (/draft_checkpoint|draft_settle/.test(c.url)) {
      const body = typeof c.req === 'string' ? c.req : ''
      const pick = (k) => {
        const m = RegExp(`"\\\\n?${k}\\\\u0001?":?`).exec(body)
        return m ? '?' : ''
      }
      const doc = /"doc_id":"([^"]*)"/.exec(body)?.[1] ?? ''
      const erev = /"erev":(\d+)/.exec(body)?.[1] ?? ''
      const seq = /"seq":(\d+)/.exec(body)?.[1] ?? ''
      const ev = /"event":"([^"]*)"/.exec(body)?.[1] ?? ''
      const bl = (body.match(/"body":"/) ? `bodyLen~${body.length}` : '')
      log(`      req: doc=${doc} erev=${erev} seq=${seq} event=${ev} ${bl} raw=${body.slice(0, 300)}`)
    }
  }
  // 编辑器 DOM 终态 + 状态栏
  const editorText = await page.evaluate(() => document.querySelector('.autodown-editor-content')?.textContent?.slice(-50) ?? '(none)')
  log('editor tail:', JSON.stringify(editorText))
  const statusText = await page.evaluate(() => {
    const els = [...document.querySelectorAll('div,span')].filter((e) => e.childElementCount === 0)
    return els.map((e) => e.textContent?.trim()).filter((t) => t && t.length < 40).slice(-25)
  })
  log('leaf texts (tail):', JSON.stringify(statusText))
  log('=== drafts after scenario ===')
  log(dumpDrafts(WS))
  const hw = fs.readFileSync(path.join(WS, 'wiki', 'Hello World.ad'), 'utf8')
  log('Hello World disk contains marker:', /p750|AB/.test(hw), '| tail:', JSON.stringify(hw.slice(-30)))
} catch (e) {
  console.error('[p750] FAILED:', e.message)
  process.exitCode = 1
} finally {
  await browser.close().catch(() => {})
  try { execFileSync('taskkill', ['/PID', String(vite.pid), '/T', '/F'], { stdio: 'ignore' }) } catch {}
  await back.stop()
}
