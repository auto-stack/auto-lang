// serve-dist.mjs — 静态 dist 服务（jade gallery serve-dist 同形；SPA 回退
// index.html）。gate vue 臂 playwright 前置服务。
import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.json': 'application/json',
}

export function startDistServer(distDir, port) {
  const server = http.createServer((req, res) => {
    const urlPath = decodeURIComponent((req.url ?? '/').split('?')[0])
    let file = path.join(distDir, urlPath)
    if (!fs.existsSync(file) || fs.statSync(file).isDirectory()) {
      file = path.join(distDir, 'index.html')
    }
    const ext = path.extname(file)
    res.writeHead(200, { 'Content-Type': MIME[ext] ?? 'application/octet-stream' })
    fs.createReadStream(file).pipe(res)
  })
  return new Promise((resolve) => server.listen(port, '127.0.0.1', () => resolve(server)))
}
