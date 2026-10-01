import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rows = JSON.parse(fs.readFileSync(path.join(root, '.vitepress/theme/data/demos.json'), 'utf8'))
if (rows.length !== 28 || new Set(rows.map(row => row.id)).size !== 28 || new Set(rows.map(row => row.slug)).size !== 28) throw new Error('Expected the frozen 28 unique demo entries')
const attr = value => String(value).replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;')
const outputs = []
for (const locale of ['en', 'zh']) {
  const zh = locale === 'zh'
  const prefix = zh ? '/zh' : ''
  const dir = path.join(root, zh ? 'zh/apps/demos' : 'apps/demos')
  const componentPath = zh ? '../../../' : '../../'
  outputs.push([path.join(dir, 'index.md'), `---
title: ${zh ? '系统应用与 Demo' : 'System applications and demos'}
description: ${zh ? '28 个 Auto 应用与示例的真实界面、用途、操作和运行条件。' : 'Real captures, purpose, operations, and runtime requirements for 28 Auto applications and examples.'}
layout: page
---

<script setup>
import DemoCatalog from '${componentPath}.vitepress/theme/components/DemoCatalog.vue'
</script>

<DemoCatalog />
`])
  for (const row of rows) {
    const c = row[locale]
    const source = `https://github.com/auto-stack/${row.sourceRepo}/tree/master/${row.sourcePath === '.' ? '' : row.sourcePath}`
    const text = `---
title: ${JSON.stringify(c.name)}
description: ${JSON.stringify(c.summary)}
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '${zh ? '../../../../' : '../../../'}.vitepress/theme/components/EvidenceImage.vue'
</script>

# ${c.name}

${c.summary}

[${zh ? '全部应用' : 'All applications'}](${prefix}/apps) · [${zh ? '28 个系统应用与示例' : '28 system apps and examples'}](${prefix}/apps/demos/)

## ${zh ? '先看界面' : 'The interface'}

<EvidenceImage src="${row.image.src}" alt="${attr(c.name + (zh ? '真实运行界面' : ' actual running interface'))}" caption="${attr(c.caption)}" :width="${row.image.width}" :height="${row.image.height}" :framed="false" loading="eager" zoom-label="${zh ? '放大图片' : 'Enlarge image'}" close-label="${zh ? '关闭' : 'Close'}" original-label="${zh ? '查看原图' : 'View original'}" />

${zh ? '截图形态' : 'Capture runtime'}：**${row.image.runtime}** · ${row.image.date}

## ${zh ? '一次典型操作' : 'A typical workflow'}

${c.steps.map((step, index) => `${index + 1}. ${step}`).join('\n')}

## ${zh ? '运行条件与当前范围' : 'Runtime requirements and current scope'}

${c.requirements}

${c.state}

${zh ? '这个介绍页使用静态文字和截图。网页虚拟桌面与应用在线体验安排在 v0.5.1；当前 v0.5 页面不运行应用后台。截图记录具体画面，典型流程帮助理解操作，并不替代全部功能的版本验收。' : 'This introduction uses static text and screenshots. The web virtual desktop and online application experiences are planned for v0.5.1; this v0.5 page does not run an application backend. A capture records one scene, and the workflow explains operations rather than certifying every feature.'}

## ${zh ? '源码与相关介绍' : 'Source and related introductions'}

${zh ? '源码工程' : 'Source project'}：[${row.sourceRepo}/${row.sourcePath}](${source})

[${zh ? 'AutoOS 虚拟桌面' : 'AutoOS virtual desktop'}](${prefix}/autoos/) · [AutoUI](${prefix}/ui) · [${zh ? '返回应用目录' : 'Back to the application catalog'}](${prefix}/apps/demos/)

${zh ? '介绍资料时点' : 'Descriptions as of'}：**2026-10-01**。${zh ? '新拍图与留存图的日期、运行形态分别记录。' : 'New and retained images have their capture dates and runtimes recorded separately.'}
`
    outputs.push([path.join(dir, row.slug, 'index.md'), text])
  }
}
const check = process.argv.includes('--check')
for (const [file, text] of outputs) {
  if (check) {
    if (!fs.existsSync(file) || fs.readFileSync(file, 'utf8') !== text) throw new Error(`Generated content drift: ${file}`)
  } else {
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, text)
  }
}
console.log(`${check ? 'Checked' : 'Generated'} ${outputs.length} bilingual catalog and introduction pages`)
