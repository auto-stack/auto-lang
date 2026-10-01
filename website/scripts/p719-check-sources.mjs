import fs from 'node:fs'
import path from 'node:path'
const root = path.resolve(import.meta.dirname, '../..')
const files = ['articles/autoos-history.md', 'articles/auto-ai-history.md', 'zh/articles/autoos-history.md', 'zh/articles/auto-ai-history.md']
const urls = [...new Set(files.flatMap(f => fs.readFileSync(path.join(root, 'website', f), 'utf8').match(/https:\/\/github\.com\/[^)\s]+/g) || []))].sort()
const receipt = path.join(root, 'docs/reports/p719-public-sources.json')
const prior = fs.existsSync(receipt) ? JSON.parse(fs.readFileSync(receipt, 'utf8')) : null
const recent = prior && Date.now()-Date.parse(prior.checkedAt) < 3600000
const results = recent ? prior.results.filter(r => r.ok && urls.includes(r.url)) : []
const pending = urls.filter(url => !results.some(r => r.url === url))
let next = 0
await Promise.all(Array.from({ length: 2 }, async () => {
  while (next < pending.length) {
    const url = pending[next++]
    let result
    for (let attempt = 0; attempt < 3; attempt++) {
     try {
      const r = await fetch(url, { signal: AbortSignal.timeout(30000) })
      const body = await r.text()
      result = { url, status: r.status, ok: r.ok && !body.includes('<title>Page not found'), checkedAt: new Date().toISOString() }
      if (r.ok || r.status === 404) break
     } catch (error) { result = { url, ok: false, error: String(error) } }
    }
    results.push(result)
  }
}))
results.sort((a,b) => a.url.localeCompare(b.url))
const report = { checkedAt: new Date().toISOString(), results }
fs.writeFileSync(receipt, JSON.stringify(report, null, 2)+'\n')
console.log(JSON.stringify({ total: results.length, failed: results.filter(r => !r.ok) }, null, 2))
if (results.some(r => !r.ok)) process.exitCode = 1
