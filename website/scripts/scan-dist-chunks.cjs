// PLAN-718 T-01：dist chunk 依赖图扫描（按内容标记定位编辑器/重组件所在 chunk）。
// 用法：node scripts/scan-dist-chunks.cjs [distDir]
// 输出含标记的 chunk 清单；--editor 输出编辑器 chunk 文件名（供 e2e 网络断言）。
'use strict'
const fs = require('fs')
const path = require('path')

const distArgIdx = process.argv.indexOf('--dist')
const dist = distArgIdx > -1 ? process.argv[distArgIdx + 1] : path.resolve(__dirname, '..', '.vitepress', 'dist')
const assets = path.join(dist, 'assets')
const MARKERS = {
  'cm-editor': 'codemirror-editor',
  'EditorState': 'codemirror-state',
  'SnippetRunner': 'snippet-runner',
  'CodeView': 'codeview',
  'ScriptShipView': 'scriptship',
  'NotesExplorer': 'notes-explorer',
  'AutoPlayground': 'autoplayground',
  'AutoFence': 'autofence',
  'HomeDemo': 'homedemo',
}

function scan(dir) {
  const out = []
  for (const f of fs.readdirSync(dir)) {
    const p = path.join(dir, f)
    if (fs.statSync(p).isDirectory()) {
      out.push(...scan(p))
      continue
    }
    if (!f.endsWith('.js')) continue
    const s = fs.readFileSync(p, 'utf8')
    const marks = Object.entries(MARKERS).filter(([needle]) => s.includes(needle)).map(([, tag]) => tag)
    // 排除匹配：CodeView 字样出现在其他组件的引用处也会命中；体积与 .lean 去重可见。
    out.push({ file: path.relative(assets, p).split(path.sep).join('/'), size: fs.statSync(p).size, marks })
  }
  return out
}

const all = scan(assets)
const hits = all.filter((h) => h.marks.length)
hits.sort((a, b) => b.size - a.size)
for (const h of hits.slice(0, 25)) console.log(String(h.size).padStart(9), h.file, '[' + h.marks.join('+') + ']')
console.log('chunks total:', all.length, 'with markers:', hits.length)

if (process.argv.includes('--editor')) {
  const editorChunks = hits.filter((h) => h.marks.includes('codemirror-editor') || h.marks.includes('codemirror-state'))
  for (const h of editorChunks) console.log('EDITOR-CHUNK', h.file)
}
