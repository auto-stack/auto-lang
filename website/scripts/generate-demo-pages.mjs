import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rows = JSON.parse(fs.readFileSync(path.join(root, '.vitepress/theme/data/demos.json'), 'utf8'))
if (rows.length !== 28 || new Set(rows.map(row => row.id)).size !== 28 || new Set(rows.map(row => row.slug)).size !== 28) throw new Error('Expected the frozen 28 unique demo entries')
const outputs = []
for (const locale of ['en', 'zh']) {
  const zh = locale === 'zh'
  const prefix = zh ? '/zh' : ''
  const dir = path.join(root, zh ? 'zh/apps/demos' : 'apps/demos')
  // Preserve shared/bookmarked URLs, without duplicating the introductions.
  for (const row of [null, ...rows]) {
    const target = `${prefix}/apps#${row ? 'demo-' + row.slug : 'system-apps'}`
    const title = row ? row[locale].name : (zh ? '系统应用与 Demo' : 'System applications and demos')
    const componentPath = row ? (zh ? '../../../../' : '../../../') : (zh ? '../../../' : '../../')
    const text = `---
title: ${JSON.stringify(title)}
layout: page
head:
  - - meta
    - name: robots
      content: noindex
  - - link
    - rel: canonical
      href: ${JSON.stringify(target)}
  - - meta
    - http-equiv: refresh
      content: ${JSON.stringify('0;url=' + target)}
---

<script setup>
import AppDemoRedirect from '${componentPath}.vitepress/theme/components/AppDemoRedirect.vue'
</script>

<AppDemoRedirect target="${target}" />
`
    outputs.push([row ? path.join(dir, row.slug, 'index.md') : path.join(dir, 'index.md'), text])
  }
}
const check = process.argv.includes('--check')
for (const [file, text] of outputs) {
  if (check) {
    if (!fs.existsSync(file) || fs.readFileSync(file, 'utf8') !== text) throw new Error(`Generated compatibility route drift: ${file}`)
  } else {
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, text)
  }
}
console.log(`${check ? 'Checked' : 'Generated'} ${outputs.length} bilingual compatibility routes; introductions live in /apps`)
