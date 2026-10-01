// PLAN-718 T-01：截图脚本公共设施（p715-shot / p718-shot 共用）。
// - startPreview/waitAlive：vitepress preview 独占端口启动（Windows taskkill 连树清理）。
// - validateLanding：goto 后断言 pathname 未漂移、locale 与 URL 前缀一致、正文非
//   404（默认主题 NotFound 渲染 .NotFound）。返回失败原因列表。
'use strict'

const { spawn } = require('child_process')
const path = require('path')

function startPreview(port) {
  return new Promise((resolve, reject) => {
    // PLAN-718 T-01：直接 spawn node 而非 shell:true + npx——Windows 下 npx→npm→node
    // 链会让 vitepress 脱离 taskkill /T 的进程树（实测 PID 存活占住管道，脚本永不
    // 退出）；单进程 node 用 proc.kill 即可干净收尾。
    const proc = spawn(
      process.execPath,
      [path.join(__dirname, '..', 'node_modules', 'vitepress', 'bin', 'vitepress.js'),
        'preview', '--port', String(port), '--strictPort'],
      { cwd: path.resolve(__dirname, '..'), stdio: 'pipe' },
    )
    // 兜底连树清理（正常路径 proc.kill 已足够）。
    const killTree = () => {
      try {
        if (process.platform === 'win32') {
          require('child_process').execSync(`taskkill /PID ${proc.pid} /T /F`, { stdio: 'ignore' })
        } else {
          proc.kill('SIGTERM')
        }
      } catch (e) { /* already gone */ }
    }
    process.on('exit', killTree)
    let ready = false
    proc.stdout.on('data', (d) => {
      if (String(d).includes(String(port)) && !ready) { ready = true; resolve(proc) }
    })
    proc.stderr.on('data', (d) => process.stderr.write(d))
    proc.on('exit', (code) => { if (!ready) reject(new Error('preview exited ' + code)) })
    setTimeout(() => { if (!ready) reject(new Error('preview timeout')) }, 60000)
  })
}

async function waitAlive(base) {
  for (let i = 0; i < 60; i++) {
    try { await fetch(base + '/'); return } catch { await new Promise(r => setTimeout(r, 500)) }
  }
  throw new Error('preview not reachable')
}

// URL 期望 locale：'/' 视为 EN，其余按 /zh 前缀判断。
function expectedLocalePrefix(url) {
  return url === '/' ? false : url.startsWith('/zh')
}

function stopPreview(proc) {
  if (!proc) return
  try { proc.kill() } catch (e) { /* already gone */ }
  try {
    if (process.platform === 'win32') {
      require('child_process').execSync(`taskkill /PID ${proc.pid} /T /F`, { stdio: 'ignore' })
    }
  } catch (e) { /* already gone */ }
}

async function validateLanding(page, requestedUrl) {
  const fails = []
  const pathname = await page.evaluate(() => location.pathname)
  const expectZh = expectedLocalePrefix(requestedUrl)
  const actualZh = pathname === '/zh' || pathname.startsWith('/zh/')
  if (actualZh !== expectZh) {
    fails.push(`locale mismatch: requested ${requestedUrl} landed on ${pathname}`)
  }
  // cleanUrls 下 /apps 与 /apps/ 等价；按归一化比较，忽略查询串与 hash。
  const norm = (p) => p.replace(/index\.html$/, '').replace(/\.html$/, '').replace(/\/+$/, '') || '/'
  const want = norm(requestedUrl.split(/[?#]/)[0])
  const got = norm(pathname)
  if (want !== got) {
    fails.push(`pathname drift: requested ${requestedUrl} landed on ${pathname}`)
  }
  const isNotFound = await page.evaluate(() => !!document.querySelector('.NotFound'))
  if (isNotFound) {
    fails.push(`404 body at ${pathname}`)
  }
  return { fails, pathname }
}

module.exports = { startPreview, stopPreview, waitAlive, validateLanding, expectedLocalePrefix }
