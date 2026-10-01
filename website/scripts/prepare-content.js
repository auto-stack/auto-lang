/**
 * Content preparation script for Auto Language website.
 *
 * This script copies documentation and books from their source locations
 * into the website directory and generates VitePress sidebar configs.
 */

import fs from 'fs'
import path from 'path'
import { spawnSync } from 'child_process'
import { fileURLToPath } from 'url'
import { DOCS_HUB, BOOKS as LEARNING_BOOKS, localeHref } from '../content/learning-navigation.mjs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const WEBSITE_ROOT = path.resolve(__dirname, '..')
const REPO_ROOT = path.resolve(WEBSITE_ROOT, '..')

// PLAN-718 T-01：worktree 布局（.wt/<group>/auto-lang）下相邻解析会落在
// .wt/<group>/book（不存在）。按 AGENTS.md 跨仓依赖序解析：env 覆盖 → 相邻
// sibling（主检出布局即命中）→ 组目录上溯（worktree 布局落主检出相邻）。
// 候选以 tapl 书目目录存在性验证，全部未命中时返回首个候选保持可读的报错路径。
function resolveBookRoot() {
  const candidates = [
    process.env.AUTO_BOOK_ROOT,
    path.resolve(REPO_ROOT, '..', 'book'),
    path.resolve(REPO_ROOT, '..', '..', '..', 'book'),
  ].filter((c) => !!c)
  for (const c of candidates) {
    if (fs.existsSync(path.join(c, 'tapl'))) return c
  }
  return candidates[0]
}

const BOOK_ROOT = resolveBookRoot()

const DOCS_SRC = path.join(REPO_ROOT, 'docs')
const DOCS_DST_EN = path.join(WEBSITE_ROOT, 'docs')
const DOCS_DST_ZH = path.join(WEBSITE_ROOT, 'zh', 'docs')

const BOOKS_DST_EN = path.join(WEBSITE_ROOT, 'books')
const BOOKS_DST_ZH = path.join(WEBSITE_ROOT, 'zh', 'books')

const SIDEBAR_CONFIG_DIR = path.join(WEBSITE_ROOT, '.vitepress', 'config')

// PLAN-718 T-03：作者源映射。生成页文件路径（相对 website/）→ 真实作者源标记：
//   lang:docs/...   仓内 docs 源文件（github auto-stack/auto-lang）
//   book:<book>/... 相邻 book 仓源文件（gitee auto-stack/book）
//   site:<rel>      website 手工页自身
// 缺失条目 = 纯生成入口（hub/书目 index），由生成 frontmatter editLink:false 关闭。
const AUTHOR_SOURCE = {}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true })
  }
}

function removeDir(dir) {
  if (fs.existsSync(dir)) {
    fs.rmSync(dir, { recursive: true, force: true })
  }
}

function walkDir(dir, callback) {
  if (!fs.existsSync(dir)) return
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      walkDir(fullPath, callback)
    } else {
      callback(fullPath, entry.name)
    }
  }
}

// ------------------------------------------------------------------
// Markdown Preprocessing
// ------------------------------------------------------------------

function escapeInInlineCode(content) {
  // Match inline code spans: `...` or ``...`` etc.
  // This regex handles 1-3 backticks
  return content.replace(/(`+)([^`]|[^`].*?[^`])\1/g, (match) => {
    return match
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
  })
}

// ------------------------------------------------------------------
// Listing → CodeView conversion
// ------------------------------------------------------------------

function parseListingAttrs(tag) {
  const attrs = {}
  const regex = /(\w+)=['"]([^'"]*)['"]/g
  let match
  while ((match = regex.exec(tag)) !== null) {
    attrs[match[1]] = match[2]
  }
  return attrs
}

function resolveListingDir(bookDir, attrs) {
  // Plan 244: Tour mode — file="ch01-hello/01_hello.at" resolves relative
  // to the tour/ subdirectory. bookDir is docs/ root, so prepend "tour/".
  if (attrs.file && (attrs.file.includes('/') || attrs.file.includes('\\'))) {
    // If the path starts with a known top-level dir (design/language/tour/etc),
    // use it directly. Otherwise, if it looks like a tour path (chXX-name/NN_),
    // prepend 'tour/'.
    let fullPath = attrs.file
    if (bookDir && !fullPath.startsWith('tour/') && !fullPath.startsWith('tour' + path.sep)) {
      if (/^ch\d/i.test(fullPath)) {
        fullPath = 'tour/' + fullPath
      }
    }
    // Split into directory and filename
    const parts = fullPath.replace(/\\/g, '/').split('/')
    const fileName = parts.pop()
    const dirPath = path.join(bookDir, parts.join(path.sep))
    // Stash fileName for resolveListingFileName
    attrs._resolvedFileName = fileName + (fileName.endsWith('.at') ? '' : '.at')
    return dirPath
  }

  // If number is provided, derive directory: number "1-1" → listings/ch01/listing-01-01
  if (attrs.number) {
    const num = attrs.number
    const match = num.match(/^([A-Za-z0-9]+)-(\d+)$/)
    if (match) {
      const ch = match[1]
      const chPadded = /^\d+$/.test(ch) ? ch.padStart(2, '0') : ch
      const idx = match[2].padStart(2, '0')
      const listingName = /^\d+$/.test(ch) ? `listing-${chPadded}-${idx}` : `listing-${ch}-${idx}`
      return path.join(bookDir, 'listings', `ch${chPadded}`, listingName)
    }
  }

  return null
}

function resolveListingFileName(attrs) {
  // Plan 244: If resolveListingDir stashed a filename, use it
  if (attrs._resolvedFileName) {
    return attrs._resolvedFileName
  }
  if (attrs['file-name']) {
    return attrs['file-name']
  }
  if (attrs.file && !attrs.file.includes('/') && !attrs.file.includes('\\')) {
    return attrs.file + '.at'
  }
  return 'main.at'
}

function readListingFiles(listingDir, fileName) {
  const baseName = fileName.replace(/\.at$/, '')
  const result = {}

  const autoPath = path.join(listingDir, fileName)
  if (fs.existsSync(autoPath)) {
    result.auto = fs.readFileSync(autoPath, 'utf-8').trimEnd()
  }

  const targets = [
    { ext: '.expected.rs', key: 'rust' },
    { ext: '.expected.c', key: 'c' },
    { ext: '.expected.ts', key: 'typescript' },
    { ext: '.expected.py', key: 'python' },
  ]

  for (const { ext, key } of targets) {
    const p = path.join(listingDir, baseName + ext)
    if (fs.existsSync(p)) {
      result[key] = fs.readFileSync(p, 'utf-8').trimEnd()
    }
  }

  return result
}

function escapeProp(value) {
  return value.replace(/"/g, '&quot;').replace(/\n/g, '&#10;')
}

function listingToCodeView(bookDir, tag) {
  const attrs = parseListingAttrs(tag)
  const listingDir = resolveListingDir(bookDir, attrs)

  if (!listingDir || !fs.existsSync(listingDir)) {
    return `<!-- Listing not found: ${tag.slice(1, -1)} -->`
  }

  const fileName = resolveListingFileName(attrs)
  const files = readListingFiles(listingDir, fileName)

  if (!files.auto) {
    return `<!-- Listing source not found: ${tag.slice(1, -1)} -->`
  }

  // Plan 358 B2: view="scriptship" renders a <ScriptShipView> (Auto VM run +
  // a2r transpile + optional compare) instead of the static <CodeView>.
  // The Rust tab is generated live by the component via /api/trans, so only
  // the .at source is read here (expected.* siblings are ignored).
  if (attrs.view === 'scriptship') {
    const props = [`auto="${escapeProp(files.auto)}"`]
    if (attrs.caption) props.push(`caption="${escapeProp(attrs.caption)}"`)
    if (attrs.compare === 'true') props.push(':compare-run="true"')
    return `<ScriptShipView ${props.join(' ')} />`
  }

  const props = [`auto="${escapeProp(files.auto)}"`]
  if (files.rust) props.push(`rust="${escapeProp(files.rust)}"`)
  if (files.c) props.push(`c="${escapeProp(files.c)}"`)
  if (files.typescript) props.push(`typescript="${escapeProp(files.typescript)}"`)
  if (files.python) props.push(`python="${escapeProp(files.python)}"`)
  if (attrs.caption) props.push(`caption="${escapeProp(attrs.caption)}"`)
  props.push(':runnable="true"')

  return `<CodeView ${props.join(' ')} />`
}

// P581-D1（master 预存红）:docs 文本里的裸泛型/占位符（Map<tableKey…>、<prefix>_pts、
// N<slot> 等）落在表格/正文（非围栏非行内代码）时被 vue 模板编译器解析为未闭合标签，
// 阻断 vitepress build。源头是 crates 侧 schema 描述文本未转义；上游修复前，在此对
// 复制产物统一转义：围栏代码与行内代码由 markdown-it 自行转义（跳过），<Listing>/<Output>
// 标签行交由既有管线（跳过），合法内联 HTML 走允许清单。转义只改 `<`，渲染文本不变。
const SAFE_RAW_TAGS = new Set([
  'a', 'abbr', 'b', 'big', 'blockquote', 'br', 'button', 'canvas', 'center', 'code',
  'details', 'div', 'em', 'figcaption', 'figure', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'hr', 'i', 'img', 'kbd', 'li', 'mark', 'ol', 'p', 'picture', 'pre', 'samp', 'small',
  'span', 'strong', 'style', 'sub', 'summary', 'sup', 'table', 'tbody', 'td', 'tfoot',
  'th', 'thead', 'tr', 'u', 'ul', 'var', 'video', 'audio', 'source',
  'CodeView', 'Output', 'Listing', 'Excalidraw',
])

function escapeDanglingAnglesOutsideCode(line) {
  // 先摘出行内代码段（占位符保护——markdown-it 会自行转义其内容），对剩余
  // 文本转义，再回填。跨行落单的反引号不构成代码段，按普通文本处理。
  const spans = []
  const masked = line.replace(/(`+)([^`]|[^`].*?[^`])\1/g, (m) => {
    spans.push(m)
    return `\u0000${spans.length - 1}\u0000`
  })
  const escaped = masked.replace(/<(\/?)([A-Za-z][A-Za-z0-9_-]*)/g, (m, slash, name) =>
    SAFE_RAW_TAGS.has(name) ? m : `&lt;${slash}${name}`)
  return escaped.replace(/\u0000(\d+)\u0000/g, (_, i) => spans[Number(i)])
}

function escapeRawAngleTags(content) {
  const lines = content.split('\n')
  let inFence = false
  const out = lines.map((line) => {
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence
      return line
    }
    if (inFence) return line
    const t = line.trim()
    if (t.startsWith('<Listing') || t.startsWith('</Listing') || t.startsWith('<Output')) return line
    return escapeDanglingAnglesOutsideCode(line)
  })
  return out.join('\n')
}

function preprocessMarkdown(content, bookDir = null) {
  content = escapeRawAngleTags(content)
  const lines = content.split('\n')
  const result = []
  let i = 0

  while (i < lines.length) {
    const line = lines[i]

    // Handle <Listing ...> tags (also handle malformed "< Listing" with space)
    const trimmed = line.trim()
    if (trimmed.startsWith('<Listing') || trimmed.startsWith('< Listing')) {
      // Normalize malformed tag by removing space after <
      const normalizedTag = trimmed.replace(/^<\s+Listing/, '<Listing')
      const codeView = bookDir ? listingToCodeView(bookDir, normalizedTag) : `<!-- ${normalizedTag.slice(1, -1)} -->`
      result.push(codeView)
      i++

      // Skip everything until </Listing> (code blocks, empty lines, etc.)
      while (i < lines.length && !lines[i].trim().startsWith('</Listing>')) {
        i++
      }
      // Skip the </Listing> line itself
      if (i < lines.length) i++
      continue
    }

    // Handle <Output ...> tags
    if (line.trim().startsWith('<Output')) {
      result.push(`<!-- ${line.trim().slice(1, -1)} -->`)
      i++
      continue
    }

    // Handle standalone </Listing> tags (orphaned, no matching opening tag)
    if (line.trim().startsWith('</Listing>')) {
      i++
      continue
    }

    // Handle standalone </Output> tags
    if (line.trim().startsWith('</Output>')) {
      result.push('<!-- /Output -->')
      i++
      continue
    }

    result.push(line)
    i++
  }

  content = result.join('\n')

  // Escape < and > inside inline code to prevent Vue parser errors
  content = escapeInInlineCode(content)

  // Escape {{ and }} to prevent Vue from parsing them as template interpolations
  content = content.replace(/\{\{/g, '&#123;&#123;')
  content = content.replace(/\}\}/g, '&#125;&#125;')

  // Escape common standalone generic patterns that look like HTML tags
  // but are likely type parameters (only when NOT in code blocks)
  const lines2 = content.split('\n')
  const inCodeBlock = new Array(lines2.length).fill(false)

  // Determine which lines are inside code blocks
  let insideCodeBlock = false
  for (let j = 0; j < lines2.length; j++) {
    const trimmed = lines2[j].trim()
    if (trimmed.startsWith('```')) {
      insideCodeBlock = !insideCodeBlock
    }
    inCodeBlock[j] = insideCodeBlock
  }

  for (let j = 0; j < lines2.length; j++) {
    if (inCodeBlock[j]) continue
    let line = lines2[j]

    // Escape common standalone type parameter patterns like <T>, <K>, <Item>, etc.
    // that are NOT already escaped or inside HTML tags
    line = line.replace(/<([A-Z][a-zA-Z0-9_]*)>/g, '&lt;$1&gt;')

    // Escape <dyn Trait> patterns
    line = line.replace(/<dyn\s+[^>]+>/g, (m) => m.replace(/</g, '&lt;').replace(/>/g, '&gt;'))

    lines2[j] = line
  }

  return lines2.join('\n')
}

const SUMMARY_LINK_FIXES = {
  'ch04-memory-model.md': 'ch04-ownership.md',
  'ch05-types.md': 'ch05-structs.md',
  'ch07-modules.md': 'ch07-packages.md',
  'ch13-closures.md': 'ch13-functional-features.md',
}

function fixSummaryLinks(content) {
  for (const [broken, fixed] of Object.entries(SUMMARY_LINK_FIXES)) {
    content = content.replaceAll(broken, fixed)
  }
  return content
}

function copyFile(src, dst, bookDir = null) {
  ensureDir(path.dirname(dst))
  if (src.endsWith('.md')) {
    let content = fs.readFileSync(src, 'utf-8')
    content = preprocessMarkdown(content, bookDir)
    if (path.basename(src) === 'SUMMARY.md') {
      content = fixSummaryLinks(content)
    }
    fs.writeFileSync(dst, content, 'utf-8')
  } else {
    fs.copyFileSync(src, dst)
  }
}

// ------------------------------------------------------------------
// Copy Docs
// ------------------------------------------------------------------

// Directories to include from docs/
const DOCS_INCLUDE = new Set([
  'design',
  'language',
  'tutorials',
  'guides',
  'architecture',
  'cli',
  'examples',
  'releases',
  'features',
  'tour',  // Plan 244: Auto Language Tour
  'script-to-ship',  // Plan 358: Script-to-Ship workflow tour
  'components',  // Plan 435 P5: schema 生成的组件 API 参考(core.md)
])

// PLAN-718 T-02：根级读者文档白名单。原 shouldIncludeDoc 对无分隔符的根级文件
// 恒 false（split()[0] 是文件名本身），docs/syntax、roadmap、migration-guide 从未
// 发布——旧 docs hub Quick Links 的这些出口全是死链（ignoreDeadLinks 掩盖）。
// 白名单只收读者向文档；其余根级内部工作文档（handoff/status/plan 摘要等）
// 维持不发布。
const ROOT_DOCS_INCLUDE = new Set(['syntax', 'roadmap', 'migration-guide'])

function shouldIncludeDoc(relPath) {
  const parts = relPath.split(path.sep)
  if (parts.length === 1) {
    const base = relPath.replace(/\.cn\.md$/, '').replace(/\.md$/, '')
    return ROOT_DOCS_INCLUDE.has(base)
  }
  return DOCS_INCLUDE.has(parts[0])
}

// PLAN-718 T-03：Windows 下 path.relative 产生反斜杠——AUTHOR_SOURCE 键/值与
// VitePress PageData.filePath（posix）对齐，一律转 posix 斜杠。
const toPosixRel = (p) => p.split(path.sep).join('/')

function prepareDocs() {
  console.log('Preparing docs...')
  removeDir(DOCS_DST_EN)
  removeDir(DOCS_DST_ZH)

  if (!fs.existsSync(DOCS_SRC)) {
    console.warn('  Source docs directory not found:', DOCS_SRC)
    return { en: [], zh: [] }
  }

  const enFiles = []
  const zhFiles = []
  // Plan 359: track ZH paths already filled by a .cn.md so the EN-fallback
  // branch does not clobber the Chinese translation when walkDir visits the
  // sibling .md afterwards (visit order is filesystem-dependent).
  const zhFilledByCn = new Set()

  walkDir(DOCS_SRC, (fullPath, name) => {
    const relPath = path.relative(DOCS_SRC, fullPath)
    if (!shouldIncludeDoc(relPath)) return

    // Plan 244: Copy .at files from tour/ directory (needed for Listing reference).
    // Plan 358: also copy script-to-ship/ .at files (ScriptShipView listings).
    if (name.endsWith('.at') && (relPath.startsWith('tour' + path.sep) || relPath.startsWith('script-to-ship' + path.sep))) {
      const dstPath = path.join(DOCS_DST_EN, relPath)
      copyFile(fullPath, dstPath)
      return
    }

    if (!name.endsWith('.md')) return

    // Plan 244: Pass docs/ root as bookDir for tour/ files so <Listing> resolves.
    // Plan 358: also for script-to-ship/ files (ScriptShipView listings).
    const isTour = relPath.startsWith('tour' + path.sep) || relPath.startsWith('script-to-ship' + path.sep)
    const docBookDir = isTour ? DOCS_SRC : null

    // .cn.md files are the Chinese translation — they do NOT go to EN, and
    // they claim the ZH slot so a later-visited sibling .md (EN fallback)
    // cannot clobber them.
    if (name.endsWith('.cn.md')) {
      const zhRelPath = relPath.replace(/\.cn\.md$/, '.md')
      const zhDstPath = path.join(DOCS_DST_ZH, zhRelPath)
      copyFile(fullPath, zhDstPath, docBookDir)
      zhFiles.push(zhRelPath)
      zhFilledByCn.add(zhRelPath)
      AUTHOR_SOURCE['zh/docs/' + toPosixRel(zhRelPath)] = 'lang:docs/' + toPosixRel(relPath)
      return
    }

    // EN source: always copy to EN.
    const enDstPath = path.join(DOCS_DST_EN, relPath)
    copyFile(fullPath, enDstPath, docBookDir)
    enFiles.push(relPath)
    AUTHOR_SOURCE['docs/' + toPosixRel(relPath)] = 'lang:docs/' + toPosixRel(relPath)

    // EN fallback to ZH — but skip if a .cn.md translation already filled it.
    if (!zhFilledByCn.has(relPath)) {
      const zhDstPath = path.join(DOCS_DST_ZH, relPath)
      copyFile(fullPath, zhDstPath, docBookDir)
      zhFiles.push(relPath)
      AUTHOR_SOURCE['zh/docs/' + toPosixRel(relPath)] = 'lang:docs/' + toPosixRel(relPath)
    }
  })

  console.log(`  Copied ${enFiles.length} EN docs, ${zhFiles.length} ZH docs`)
  return { en: enFiles, zh: zhFiles }
}

// ------------------------------------------------------------------
// Copy Books
// ------------------------------------------------------------------

const BOOKS = [
  'tapl',
  'rust',
  'typescript',
  'typescript-deepdive',
  'little-c',
  'modern-c',
  'byte-of-python',
  'think-python',
]

function prepareBooks() {
  console.log('Preparing books...')
  removeDir(BOOKS_DST_EN)
  removeDir(BOOKS_DST_ZH)

  const enFiles = {}
  const zhFiles = {}

  for (const book of BOOKS) {
    const srcDir = path.join(BOOK_ROOT, book)
    if (!fs.existsSync(srcDir)) {
      console.warn('  Book not found:', srcDir)
      continue
    }

    enFiles[book] = []
    zhFiles[book] = []

    // First pass: identify which files have .cn.md versions
    const hasCnVersion = new Set()
    walkDir(srcDir, (fullPath, name) => {
      if (name.endsWith('.cn.md')) {
        const relPath = path.relative(srcDir, fullPath)
        hasCnVersion.add(relPath.replace(/\.cn\.md$/, '.md'))
      }
    })

    // Second pass: copy files
    walkDir(srcDir, (fullPath, name) => {
      if (!name.endsWith('.md')) return
      const relPath = path.relative(srcDir, fullPath)

      // Always copy to EN
      const enDstPath = path.join(BOOKS_DST_EN, book, relPath)
      copyFile(fullPath, enDstPath, srcDir)
      enFiles[book].push(relPath)
      AUTHOR_SOURCE['books/' + book + '/' + toPosixRel(relPath)] = 'book:' + book + '/' + toPosixRel(relPath)

      if (name.endsWith('.cn.md')) {
        // Chinese version goes to ZH without .cn suffix
        const zhRel = relPath.replace(/\.cn\.md$/, '.md')
        const zhDstPath = path.join(BOOKS_DST_ZH, book, zhRel)
        copyFile(fullPath, zhDstPath, srcDir)
        zhFiles[book].push(zhRel)
        AUTHOR_SOURCE['zh/books/' + book + '/' + toPosixRel(zhRel)] = 'book:' + book + '/' + toPosixRel(relPath)
      } else if (!hasCnVersion.has(relPath)) {
        // Only copy non-Chinese files to ZH if there's no .cn.md version
        const zhDstPath = path.join(BOOKS_DST_ZH, book, relPath)
        copyFile(fullPath, zhDstPath, srcDir)
        zhFiles[book].push(relPath)
        AUTHOR_SOURCE['zh/books/' + book + '/' + toPosixRel(relPath)] = 'book:' + book + '/' + toPosixRel(relPath)
      }
    })

    // Generate index.md for EN book
    const enSummaryPath = path.join(BOOKS_DST_EN, book, 'SUMMARY.md')
    if (!enFiles[book].includes('index.md') && !enFiles[book].includes('README.md')) {
      generateBookIndex(path.join(BOOKS_DST_EN, book), enSummaryPath, 'en')
      enFiles[book].push('index.md')
    }

    // Generate index.md for ZH book
    const zhSummaryPath = path.join(BOOKS_DST_ZH, book, 'SUMMARY.md')
    if (!zhFiles[book].includes('index.md') && !zhFiles[book].includes('README.md')) {
      generateBookIndex(path.join(BOOKS_DST_ZH, book), zhSummaryPath, 'zh')
      zhFiles[book].push('index.md')
    }

    console.log(`  ${book}: ${enFiles[book].length} EN, ${zhFiles[book].length} ZH`)
  }

  return { en: enFiles, zh: zhFiles }
}

// ------------------------------------------------------------------
// Sidebar Generation — Docs
// ------------------------------------------------------------------

const ZH_TITLE_MAP = {
  // Top-level sections
  'Architecture': '架构',
  'Cli': 'CLI',
  'Design': '设计',
  'Examples': '示例',
  'Guides': '指南',
  'Language': '语言',
  'Releases': '发布',
  'Tutorials': '教程',
  'Raw': '原始设计',
  'Spec updates': '规范更新',
  // Raw design docs
  'A2ark': 'A2ark',
  'A2c lvgl analysis': 'A2C LVGL 分析',
  'A2jet': 'A2jet',
  'Abc': 'ABC',
  'Ai native': 'AI 原生',
  'Art': 'Art',
  'Ash coreutils': 'Ash 核心工具',
  'Ash smartcmd design': 'Ash 智能命令设计',
  'Astl': 'ASTL',
  'Atom builder api design': 'Atom 构建器 API 设计',
  'Atom serialize': 'Atom 序列化',
  'Atom': 'Atom',
  'Aura': 'Aura',
  'Auto cache': 'Auto Cache',
  'Auto cli': 'Auto CLI',
  'Auto down': 'Auto Down',
  'Auto flow': 'Auto Flow',
  'Auto mode': 'Auto 模式',
  'Auto vm bigvm': 'Auto VM BigVM',
  'Auto vm mix': 'Auto VM 混合',
  'Autogen': 'Autogen',
  'Autovm autolive': 'AutoVM 自动热更新',
  'Autovm generics': 'AutoVM 泛型',
  'Autovm streaming': 'AutoVM 流式',
  'Autovm task msg': 'AutoVM 任务消息',
  'Autovm tokio': 'AutoVM Tokio',
  'Bit operations': '位运算',
  'C': 'C',
  'Compile time execution': '编译期执行',
  'Containers': '容器',
  'Data structures': '数据结构',
  'Design token system': '设计令牌系统',
  'Dot notation': '点符号',
  'Enhanced main': '增强型 main',
  'Error system': '错误系统',
  'Exceptionals': '异常',
  'Extending atom': '扩展 Atom',
  'Frontend backend communication': '前后端通信',
  'Functions': '函数',
  'Generic constraints': '泛型约束',
  'Http server stdlib': 'HTTP 服务器标准库',
  'Incremental compilation': '增量编译',
  'May type': 'May 类型',
  'Mcu hot reloading': 'MCU 热重载',
  'Memory': '内存',
  'Microvm atom': 'MicroVM Atom',
  'New memory': '新内存模型',
  'OOP': '面向对象',
  'Organizations': '组织',
  'Os': '操作系统',
  'Param passing default': '参数传递默认值',
  'Potential keywords': '潜在关键字',
  'Prune': '剪枝',
  'Result type': 'Result 类型',
  'Scenario': '场景',
  'Shared': '共享',
  'Stdlib organization': '标准库组织',
  'Storages': '存储',
  'Task msg': '任务消息',
  'Type inference': '类型推断',
  'Types': '类型',
  'Typestore unification design': '类型存储统一设计',
  'Unified enum': '统一枚举',
  'Union': '联合类型',
  'Value access': '值访问',
  'Vue router': 'Vue 路由',
  // Design section
  'Type system': '类型系统',
  'Error handling': '错误处理',
  'Memory ownership': '内存所有权',
  'Vm runtime': 'VM 运行时',
  'Code generation': '代码生成',
  'Ui systems': 'UI 系统',
  'Compiler': '编译器',
  'Language syntax': '语言语法',
  'Shell tools': 'Shell 工具',
  'Vm debugging': 'VM 调试',
  // Examples
  'Mixed mode project': '混合模式项目',
  // Guides
  'Autocache guide': 'Autocache 指南',
  'Ffi usage guide': 'FFI 使用指南',
  'Migration guide': '迁移指南',
  'Mode selection guide': '模式选择指南',
  // Language
  'Specification': '语言规范',
  'Batch 01 metadata': '批次 01：元数据',
  'Batch 02 lexical': '批次 02：词法',
  'Batch 03 types': '批次 03：类型',
  'Batch 04 expressions': '批次 04：表达式',
  'Batch 05 statements': '批次 05：语句',
  'Batch 06 functions': '批次 06：函数',
  'Batch 07 type defs': '批次 07：类型定义',
  'Batch 08 specs': '批次 08：规范',
  'Batch 09 generics closures option': '批次 09：泛型、闭包、Option',
  'Batch 10 concurrency': '批次 10：并发',
  'Batch 11 comptime ownership modules': '批次 11：编译期、所有权、模块',
  'Batch 12 ui routing cleanup': '批次 12：UI、路由、清理',
  // Releases
  'V0.1': 'v0.1',
  'V0.2': 'v0.2',
  'V0.3': 'v0.3',
  // Tutorials
  'Array return types': '数组返回类型',
  'Atom api guide': 'Atom API 指南',
  'Atom api guide.cn': 'Atom API 指南（中文）',
  'Autogen tutorial': 'Autogen 教程',
  'Autogen tutorial.cn': 'Autogen 教程（中文）',
  'Ext statement': 'ext 语句',
  'For loop guide': 'for 循环指南',
  'Method calls': '方法调用',
  'Stdlib organization': '标准库组织',
  // Features
  'Features': '功能特性',
  'Actor concurrency': 'Actor 并发',
  'Ai native design': 'AI 原生设计',
  'Autovm interpreter': 'AutoVM 解释器',
  'Comptime metaprogramming': '编译期元编程',
  'Memory safety': '内存安全',
  'Multi target transpiler': '多目标转译器',
  // Misc
  'Autocache': 'Autocache',
  'Bpbe': 'BPBE',
  'Autocache cli': 'Autocache CLI',
  // Book titles
  'Tapl': 'Auto 编程语言',
  'Rust': 'Auto版Rust Book',
  'Typescript': 'Auto版TypeScript Handbook',
  'Typescript deepdive': 'Auto版TypeScript DeepDive',
  'Little c': 'Auto版The Little Book of C',
  'Modern c': 'Auto版Modern C',
  'Byte of python': 'Auto版A Byte of Python',
  'Think python': 'Auto版Think Python',
  // Common book chapters
  'Introduction': '简介',
  'Getting Started': '入门',
  'Getting started': '入门',
  'Variables & Operators': '变量与运算符',
  'Functions & Control Flow': '函数与控制流',
  'Collections & Nodes': '集合与节点',
  'Project: Guessing Game': '项目：猜数字游戏',
  'Types & `let`': '类型与 let',
  'Enums & Pattern Matching': '枚举与模式匹配',
  'OOP Reshaped': '重塑 OOP',
  'Error Handling': '错误处理',
  'Packages & Modules': '包与模块',
  'References & Pointers': '引用与指针',
  'Memory & Ownership': '内存与所有权',
  'Project: File Processor': '项目：文件处理器',
  'Actor Concurrency': 'Actor 并发',
  'Async with `~T`': '异步与 ~T',
  'Smart Casts & Flow Typing': '智能转换与流类型',
  'Testing': '测试',
  'Closures & Iterators': '闭包与迭代器',
  'Comptime & Metaprogramming': '编译期与元编程',
  'Standard Library Tour': '标准库概览',
  'Project: Multi-user Chat Server': '项目：多用户聊天服务器',
  'Appendix A: Keyword Reference': '附录 A：关键字参考',
  'Appendix B: Operator Table': '附录 B：运算符表',
  'Appendix C: Transpiler Quick-Ref': '附录 C：转译器速查',
  'Appendix D: Standard Library Index': '附录 D：标准库索引',
  'About python': '关于 Python',
  'Advanced Features': '高级特性',
  'An I/O Project: Building a Command Line Tool': 'I/O 项目：构建命令行工具',
  'Appendix': '附录',
  'Async Programming with `~T`': '异步编程与 ~T',
  'Atomics': '原子操作',
  'Basic values': '基本值',
  'Basics': '基础',
  'C library': 'C 标准库',
  'Classes': '类',
  'Classes functions': '类与函数',
  'Classes methods': '类与方法',
  'Classes objects': '类与对象',
  'Common Collections': '常用集合',
  'Common Programming Concepts': '常用编程概念',
  'Compilation': '编译',
  'Compiler': '编译器',
  'Conditionals': '条件语句',
  'Control flow': '控制流',
  'Control flow variations': '控制流变体',
  'Creating Types from Types': '从类型创建类型',
  'Debugging': '调试',
  'Derived types': '派生类型',
  'Design patterns': '设计模式',
  'Dictionaries': '字典',
  'Discriminated unions': '可辨识联合',
  'Enums and Pattern Matching': '枚举与模式匹配',
  'Errors': '错误',
  'Everyday Types': '日常类型',
  'Exceptions': '异常',
  'Expressions': '表达式',
  'Extras': '额外内容',
  'Final Project: Building a Multithreaded Web Server': '最终项目：构建多线程 Web 服务器',
  'Final thoughts': '最终思考',
  'First steps': '第一步',
  'Generics, Specs, and AutoFree': '泛型、规格与 AutoFree',
  'Index types': '索引类型',
  'Inheritance': '继承',
  'Input output': '输入输出',
  'Installation': '安装',
  'Interfaces': '接口',
  'Interfaces enums': '接口与枚举',
  'Io files': 'IO 文件',
  'Io processing': 'IO 处理',
  'Iteration': '迭代',
  'Language basics': '语言基础',
  'Lists': '列表',
  'Macros': '宏',
  'Memory model': '内存模型',
  'Mixins errors': 'Mixins 与错误',
  'Modern features': '现代特性',
  'Modules': '模块',
  'More': '更多',
  'More About automan': '更多关于 automan',
  'More on Functions': '更多关于函数',
  'Narrowing': '收窄',
  'Object Types': '对象类型',
  'Object-Oriented Patterns in Auto': 'Auto 中的面向对象模式',
  'Oop': 'OOP',
  'Operators expressions': '运算符与表达式',
  'Organization': '组织',
  'Packages and Modules': '包与模块',
  'Patterns and Matching': '模式与匹配',
  'Performance': '性能',
  'Pointers': '指针',
  'Portable modern': '可移植与现代',
  'Preface': '前言',
  'Problem solving': '问题解决',
  'Program failure': '程序失败',
  'Program structure': '程序结构',
  'Programming a Guessing Game': '编程：猜数字游戏',
  'Project modules': '项目与模块',
  'Real projects': '实际项目',
  'References and Pointers': '引用与指针',
  'Return values': '返回值',
  'Stdlib': '标准库',
  'Storage': '存储',
  'Strings': '字符串',
  'Structuring data': '数据结构',
  'Style': '风格',
  'System programming': '系统编程',
  'Text analysis': '文本分析',
  'Thinking': '思考',
  'Threads': '线程',
  'Tuples': '元组',
  'Type compatibility': '类型兼容性',
  'Type generic': '类型泛型',
  'Type guards': '类型守卫',
  'Type Operators': '类型运算符',
  'Understanding Auto\'s Memory Model': '理解 Auto 的内存模型',
  'Understanding Errors': '理解错误',
  'Using `type` to Structure Related Data': '使用 type 组织相关数据',
  'Variables': '变量',
  'What next': '下一步',
  'Why types': '为什么需要类型',
  'Writing Automated Tests': '编写自动化测试',
  // Chinese originals (keep as-is)
  'About python.cn': '关于 Python（中文）',
  'Atomics.cn': '原子操作（中文）',
  'Basic values.cn': '基本值（中文）',
  'Basics.cn': '基础（中文）',
  'Classes functions.cn': '类与函数（中文）',
  'Classes methods.cn': '类与方法（中文）',
  'Classes objects.cn': '类与对象（中文）',
  'Compilation.cn': '编译（中文）',
  'Compiler.cn': '编译器（中文）',
  'Conditionals.cn': '条件语句（中文）',
  'Control flow variations.cn': '控制流变体（中文）',
  'Control flow.cn': '控制流（中文）',
  'Data structures.cn': '数据结构（中文）',
  'Debugging.cn': '调试（中文）',
  'Derived types.cn': '派生类型（中文）',
  'Design patterns.cn': '设计模式（中文）',
  'Dictionaries.cn': '字典（中文）',
  'Discriminated unions.cn': '可辨识联合（中文）',
  'Errors.cn': '错误（中文）',
  'Exceptions.cn': '异常（中文）',
  'Expressions.cn': '表达式（中文）',
  'Extras.cn': '额外内容（中文）',
  'Final thoughts.cn': '最终思考（中文）',
  'First steps.cn': '第一步（中文）',
  'Functions.cn': '函数（中文）',
  'Generics.cn': '泛型（中文）',
  'Getting started.cn': '入门（中文）',
  'Index types.cn': '索引类型（中文）',
  'Inheritance.cn': '继承（中文）',
  'Input output.cn': '输入输出（中文）',
  'Installation.cn': '安装（中文）',
  'Interfaces enums.cn': '接口与枚举（中文）',
  'Interfaces.cn': '接口（中文）',
  'Io files.cn': 'IO 文件（中文）',
  'Io processing.cn': 'IO 处理（中文）',
  'Iteration.cn': '迭代（中文）',
  'Language basics.cn': '语言基础（中文）',
  'Lists.cn': '列表（中文）',
  'Macros.cn': '宏（中文）',
  'Memory model.cn': '内存模型（中文）',
  'Memory.cn': '内存（中文）',
  'Mixins errors.cn': 'Mixins 与错误（中文）',
  'Modules.cn': '模块（中文）',
  'Oop.cn': 'OOP（中文）',
  'Operators expressions.cn': '运算符与表达式（中文）',
  'Organization.cn': '组织（中文）',
  'Pointers.cn': '指针（中文）',
  'Preface.cn': '前言（中文）',
  'Problem solving.cn': '问题解决（中文）',
  'Program failure.cn': '程序失败（中文）',
  'Program structure.cn': '程序结构（中文）',
  'Project modules.cn': '项目与模块（中文）',
  'Real projects.cn': '实际项目（中文）',
  'Return values.cn': '返回值（中文）',
  'Stdlib.cn': '标准库（中文）',
  'Storage.cn': '存储（中文）',
  'Strings.cn': '字符串（中文）',
  'Structuring data.cn': '数据结构（中文）',
  'Style.cn': '风格（中文）',
  'System programming.cn': '系统编程（中文）',
  'Text analysis.cn': '文本分析（中文）',
  'Thinking.cn': '思考（中文）',
  'Threads.cn': '线程（中文）',
  'Tuples.cn': '元组（中文）',
  'Type compatibility.cn': '类型兼容性（中文）',
  'Type generic.cn': '类型泛型（中文）',
  'Type guards.cn': '类型守卫（中文）',
  'Type system.cn': '类型系统（中文）',
  'Variables.cn': '变量（中文）',
  'What next.cn': '下一步（中文）',
  'Why types.cn': '为什么需要类型（中文）',
  // SUMMARY headings
  'The Auto Programming Language': 'Auto 编程语言',
  'Phase 1 — Auto as Script': '第一阶段 — Auto 作为脚本',
  'Phase 2 — Auto as System': '第二阶段 — Auto 作为系统',
  'Phase 3 — Auto as AIOS': '第三阶段 — Auto 作为 AIOS',
  'Appendices': '附录',
}

function buildDocsSidebar(files, lang = 'en') {
  const tree = {}

  for (const file of files) {
    const parts = file.split(path.sep)
    const fileName = parts.pop()
    let current = tree

    for (const part of parts) {
      if (!current[part]) current[part] = {}
      current = current[part]
    }

    current[fileName] = file
  }

  function translateTitle(title) {
    if (lang === 'zh' && ZH_TITLE_MAP[title]) {
      return ZH_TITLE_MAP[title]
    }
    return title
  }

  function toSidebarItems(node, prefix = '') {
    const items = []
    const dirs = []
    const leafs = []

    for (const [key, value] of Object.entries(node)) {
      if (typeof value === 'string') {
        const name = key.replace(/\.md$/, '')
        const title = name
          .replace(/-/g, ' ')
          .replace(/_/g, ' ')
          .replace(/^ch\d+[-\s]/, '')
          .replace(/^\d+[-\s]/, '')
        leafs.push({
          text: translateTitle(title.charAt(0).toUpperCase() + title.slice(1)),
          link: prefix + name,
        })
      } else {
        dirs.push({ key, value })
      }
    }

    for (const { key, value } of dirs) {
      const title = key
        .replace(/-/g, ' ')
        .replace(/_/g, ' ')
      items.push({
        text: translateTitle(title.charAt(0).toUpperCase() + title.slice(1)),
        collapsed: true,
        items: toSidebarItems(value, prefix + key + '/'),
      })
    }

    items.push(...leafs)
    return items
  }

  const tree2 = toSidebarItems(tree)
  // PLAN-718 T-03：入门路径置顶（真实内容：Auto Tour 第一章）；完整参考目录
  // 保持不动。链接相对 docs base（'/docs/'）。
  tree2.unshift({
    text: lang === 'zh' ? '开始使用' : 'Start here',
    link: 'tour/ch01-hello',
  })
  return tree2
}

// ------------------------------------------------------------------
// Sidebar Generation — Books (from SUMMARY.md)
// ------------------------------------------------------------------

function translateTitle(title, lang) {
  if (lang === 'zh' && ZH_TITLE_MAP[title]) {
    return ZH_TITLE_MAP[title]
  }
  return title
}

function parseSummary(summaryPath, lang = 'en') {
  if (!fs.existsSync(summaryPath)) return null

  const content = fs.readFileSync(summaryPath, 'utf-8')
  const lines = content.split('\n')
  const root = []
  const stack = [{ items: root, depth: -1 }]

  for (const line of lines) {
    // PLAN-718 T-06：mdbook SUMMARY 的顶层章节也允许裸 [Text](target.md) 形式
    // （无列表标记，如 tapl 的 Introduction / rust 的 title-page）——此前被跳过，
    // 导致首章不在侧栏、prev/next 缺失（tapl ch00 无 next 实测）。
    let match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
    if (!match) {
      const plain = line.match(/^\[([^\]]+)\]\s*\(([^)]+)\)/)
      if (plain) match = [line, '', plain[1], plain[2]]
    }
    if (!match) continue

    const depth = match[1].length
    const text = translateTitle(match[2], lang)
    const link = match[3].replace(/\.md$/, '')
    const item = { text, link }

    while (stack.length > 1 && stack[stack.length - 1].depth >= depth) {
      stack.pop()
    }

    const parent = stack[stack.length - 1]
    if (!parent.items) parent.items = []
    parent.items.push(item)
    stack.push({ ...item, depth })
  }

  return root
}

function generateBookIndex(bookDir, summaryPath, lang = 'en') {
  const bookName = path.basename(bookDir)
  const bookTitle = translateTitle(bookName.replace(/-/g, ' ').replace(/^\w/, (c) => c.toUpperCase()), lang)
  const indexPath = path.join(bookDir, 'index.md')

  const tocHeading = lang === 'zh' ? '目录' : 'Table of Contents'
  const fallbackText = lang === 'zh' ? '章节列表将在此处显示。' : 'Chapters will be listed here.'
  // editLink:false：书目首页为生成入口（PLAN-718 T-03）。
  let content = `---\ntitle: ${bookTitle}\neditLink: false\n---\n\n# ${bookTitle}\n\n`

  if (fs.existsSync(summaryPath)) {
    const summary = fs.readFileSync(summaryPath, 'utf-8')
    content += `## ${tocHeading}\n\n`
    const lines = summary.split('\n')
    for (const line of lines) {
      // 与 parseSummary 同口径：列表项与裸顶层链接都收（PLAN-718 T-06）。
      let match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
      if (!match) {
        const plain = line.match(/^\[([^\]]+)\]\s*\(([^)]+)\)/)
        if (plain) match = [line, '', plain[1], plain[2]]
      }
      if (match) {
        const depth = match[1].length
        const text = translateTitle(match[2], lang)
        const link = match[3].replace(/\.md$/, '')
        const indent = '  '.repeat(depth / 2)
        content += `${indent}- [${text}](./${link})\n`
      } else if (line.trim().startsWith('#')) {
        const heading = translateTitle(line.trim().replace(/^#+\s*/, ''), lang)
        content += `\n### ${heading}\n\n`
      }
    }
  } else {
    content += `${fallbackText}\n`
  }

  fs.writeFileSync(indexPath, content, 'utf-8')
}

// ------------------------------------------------------------------
// Learning hub pages（PLAN-718 T-02）
// ------------------------------------------------------------------

// 学习导航 href → 生成页/公共资产文件映射校验：入口必须落在真实内容上，
// 任何失效引用在 prepare 阶段直接失败（不静默、不依赖 ignoreDeadLinks）。
function hubTargetExists(href) {
  const clean = href.split('#')[0].split('?')[0]
  // .html（共享 SPA public/ui/*/index.html）双语同 URL，不加 zh 前缀（与
  // theme/data/navigation.ts sharedSpa 同口径），仅按 public 资产校验一次。
  const variants = clean.endsWith('.html') ? [clean] : [clean, localeHref(clean, true)]
  for (const p of variants) {
    if (p.endsWith('.html')) {
      if (!fs.existsSync(path.join(WEBSITE_ROOT, 'public', p))) return `missing public asset: ${p}`
      continue
    }
    let rel
    if (p === '/' || p === '') rel = 'index.md'
    else if (p.endsWith('/')) rel = p.slice(1) + 'index.md'
    else rel = p.slice(1) + '.md'
    if (!fs.existsSync(path.join(WEBSITE_ROOT, rel))) return `no generated page for ${p}`
  }
  return null
}

function collectHubHrefs() {
  const hrefs = []
  for (const card of [...DOCS_HUB.primary, ...DOCS_HUB.cards]) hrefs.push(card.href)
  for (const group of DOCS_HUB.tasks) for (const item of group.items) hrefs.push(item.href)
  for (const book of LEARNING_BOOKS) hrefs.push(book.href, book.entry)
  return hrefs
}

function validateHubLinks() {
  const fails = []
  for (const href of collectHubHrefs()) {
    const err = hubTargetExists(href)
    if (err) fails.push(err)
  }
  if (fails.length) {
    throw new Error(`learning-navigation hrefs not resolvable:\n  ${fails.join('\n  ')}`)
  }
  console.log(`  Learning hub links validated: ${collectHubHrefs().length} hrefs all resolve`)
}

// PLAN-718 T-06：搜索索引桥接。VitePress local search 索引由 md.render(markdown 源)
// 构建（dist node chunk 实测），<LearningHub> 组件文本不进索引——hub 页在组件后
// 追加"全部入口"纯 markdown 清单：进入静态 HTML 与搜索索引（AC-01）。
// 尝试过 sr-only <div> 包裹 markdown——markdown-it 的 HTML 块在空行截断，闭合标签
// 游离导致 Vue 编译失败（实测），故为可见的紧凑清单段。
function hubSearchBridgeDocs(lang) {
  const zh = lang === 'zh'
  const t = (pair) => (zh ? pair.zh : pair.en)
  const lines = [`## ${zh ? '全部入口' : 'All entries'}`, '']
  for (const card of [...DOCS_HUB.primary, ...DOCS_HUB.cards]) {
    lines.push(`- [${t(card.title)}](${localeHref(card.href, zh)}) — ${t(card.desc)}`)
  }
  for (const group of DOCS_HUB.tasks) {
    lines.push(`- ${t(group.title)}`)
    for (const item of group.items) lines.push(`  - [${t(item.label)}](${localeHref(item.href, zh)})`)
  }
  return lines.join('\n')
}

function hubSearchBridgeBooks(lang) {
  const zh = lang === 'zh'
  const t = (pair) => (zh ? pair.zh : pair.en)
  const lines = [`## ${zh ? '全部入口' : 'All entries'}`, '']
  for (const book of LEARNING_BOOKS) {
    lines.push(`- [${t(book.title)}](${localeHref(book.href, zh)}) — ${t(book.blurb)} ${t(book.audience)}`)
    lines.push(`  - [${zh ? '第一章' : 'First chapter'}](${localeHref(book.entry, zh)})`)
  }
  return lines.join('\n')
}

function generateDocsIndex(docsDir, lang) {
  ensureDir(docsDir)
  const zh = lang === 'zh'
  const t = (pair) => (zh ? pair.zh : pair.en)
  const indexPath = path.join(docsDir, 'index.md')
  const title = zh ? '文档' : 'Documentation'

  // editLink:false：纯生成入口不显示虚假编辑链接（PLAN-718 T-03）。
  // next:false：hub 不是章节序列的一环，取消无语义的"下一页 Autocache"。
  let content = `---\ntitle: ${title}\neditLink: false\nnext: false\n---\n\n# ${title}\n\n`
  content += `${t(DOCS_HUB.intro)}\n\n`
  content += `<LearningHub kind="docs" />\n\n`
  content += hubSearchBridgeDocs(lang) + '\n'

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function generateBooksIndex(booksDir, lang) {
  ensureDir(booksDir)
  const zh = lang === 'zh'
  const indexPath = path.join(booksDir, 'index.md')
  const title = zh ? '教程' : 'Tutorials'

  let content = `---\ntitle: ${title}\neditLink: false\nnext: false\n---\n\n# ${title}\n\n`
  content += `${zh ? '八本教程覆盖从零基础到系统编程——主书优先，也可按你已有的语言背景选择。' : 'Eight tutorials cover everything from first steps to systems programming — start with the main book, or pick by a language you already know.'}\n\n`
  content += `<LearningHub kind="books" />\n\n`
  content += hubSearchBridgeBooks(lang) + '\n'

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function prefixBookLinks(items, book) {
  return items.map(item => ({
    ...item,
    link: item.link ? `${book}/${item.link}` : undefined,
    items: item.items ? prefixBookLinks(item.items, book) : undefined,
  }))
}

// PLAN-718 T-03：返回 { sidebar, byBook }——sidebar 为 books hub 的全书列表
// （每书折叠组，链接带书前缀，base=/books/）；byBook[id] 为该书章节树
// （链接相对书根，配合 en/zh 配置的 '/books/<id>/' 最长前缀侧栏，使上一章/
// 下一章与侧栏只覆盖当前书，消除跨书 prev/next）。
function buildBooksSidebar(bookFiles, lang = 'en') {
  const sidebar = []
  const byBook = {}
  const booksDst = lang === 'zh' ? BOOKS_DST_ZH : BOOKS_DST_EN

  for (const book of BOOKS) {
    const files = bookFiles[book] || []
    if (files.length === 0) continue

    const summaryPath = path.join(booksDst, book, 'SUMMARY.md')
    const items = parseSummary(summaryPath, lang)

    const bookTitle = translateTitle(book
      .replace(/-/g, ' ')
      .replace(/^\w/, (c) => c.toUpperCase()), lang)

    // Generate index.md if it doesn't exist
    if (!files.includes('index.md') && !files.includes('README.md')) {
      generateBookIndex(path.join(booksDst, book), summaryPath, lang)
      if (!files.includes('index.md')) files.push('index.md')
    }

    if (items && items.length > 0) {
      byBook[book] = items
      sidebar.push({
        text: bookTitle,
        link: `${book}/`,
        collapsed: book !== 'tapl',
        items: prefixBookLinks(items, book),
      })
    } else {
      // Fallback: list all markdown files. PLAN-718 T-03：EN 侧栏排除 *.cn.md
      // （中文重复条目泄漏）；页面仍存在于 EN 树，仅不再进导航（ZH 侧经
      // /zh/books 覆盖，目的地不丢）。
      const leafs = files
        .filter((f) => f.endsWith('.md') && !f.endsWith('SUMMARY.md') && f !== 'index.md')
        .filter((f) => !(lang === 'en' && f.endsWith('.cn.md')))
        .map((f) => {
          const name = path.basename(f, '.md')
          const title = name
            .replace(/-/g, ' ')
            .replace(/^ch\d+[-\s]/, '')
          return {
            text: translateTitle(title.charAt(0).toUpperCase() + title.slice(1), lang),
            link: `${book}/${name}`,
          }
        })

      byBook[book] = leafs.map(({ link, ...rest }) => ({
        ...rest,
        link: link.slice(book.length + 1),
      }))
      sidebar.push({
        text: bookTitle,
        link: `${book}/`,
        collapsed: book !== 'tapl',
        items: leafs,
      })
    }
  }

  return { sidebar, byBook }
}

// ------------------------------------------------------------------
// Write sidebar config files
// ------------------------------------------------------------------

function toValidIdentifier(name) {
  return name
    .split(/[-_]/)
    .map((part, i) => (i === 0 ? part : part.charAt(0).toUpperCase() + part.slice(1)))
    .join('')
}

function writeSidebarConfig(name, sidebar) {
  const filePath = path.join(SIDEBAR_CONFIG_DIR, `sidebar-${name}.ts`)
  const varName = 'sidebar' + name.split(/[-_]/).map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join('')
  const content = `import type { DefaultTheme } from 'vitepress'

export const ${varName}: DefaultTheme.SidebarItem[] = ${JSON.stringify(sidebar, null, 2)}
`
  fs.writeFileSync(filePath, content, 'utf-8')
  console.log(`  Generated sidebar config: ${filePath}`)
}

// PLAN-718 T-03：books 侧栏双导出——hub 全书列表 + 每书章节树。
function writeBooksSidebarConfigs(lang, sidebar, byBook) {
  const name = `books-${lang}`
  const filePath = path.join(SIDEBAR_CONFIG_DIR, `sidebar-${name}.ts`)
  const varName = 'sidebarBooks' + (lang === 'zh' ? 'Zh' : 'En')
  const content = `import type { DefaultTheme } from 'vitepress'

export const ${varName}: DefaultTheme.SidebarItem[] = ${JSON.stringify(sidebar, null, 2)}

export const ${varName}ByBook: Record<string, DefaultTheme.SidebarItem[]> = ${JSON.stringify(byBook, null, 2)}
`
  fs.writeFileSync(filePath, content, 'utf-8')
  console.log(`  Generated sidebar config: ${filePath}`)
}

// PLAN-718 T-03：site 手工页作者源映射（website 根与子目录的 .md，排除生成树）。
function collectSiteAuthorSource() {
  const GENERATED = new Set(['docs', 'books', 'node_modules', '.vitepress', 'public', 'scripts', 'tests', 'content', 'test-results', 'playwright-report'])
  const walk = (dir, rel) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const relPath = rel ? rel + '/' + entry.name : entry.name
      if (entry.isDirectory()) {
        if (GENERATED.has(entry.name)) continue
        walk(path.join(dir, entry.name), relPath)
      } else if (entry.name.endsWith('.md')) {
        AUTHOR_SOURCE[relPath.split(path.sep).join('/')] = 'site:' + relPath.split(path.sep).join('/')
      }
    }
  }
  walk(WEBSITE_ROOT, '')
}

function writeAuthorSource() {
  const filePath = path.join(SIDEBAR_CONFIG_DIR, 'author-source.ts')
  const keys = Object.keys(AUTHOR_SOURCE).sort()
  const lines = keys.map((k) => `  ${JSON.stringify(k)}: ${JSON.stringify(AUTHOR_SOURCE[k])},`)
  // 注意：VitePress 会把 themeConfig 里的函数序列化到 app bundle（deserializeFunctions
  // 重新求值），闭包导入全部丢失——所以 pattern 必须自包含：映射与 URL 常量都
  // 内联在函数体内（实测 ReferenceError: authorSource is not defined 的教训）。
  const content = `// PLAN-718 T-03：生成文件——勿手改（owner=prepare-content.js）。
// 生成页 → 作者源映射的 editLink pattern。lang:docs/… → github
// auto-stack/auto-lang（分支 master）；book:… → gitee auto-stack/book（分支
// master）；site:… → 仓内 website/。生成入口页不在表中（frontmatter editLink:false）。
export const editLinkPattern = (page: { filePath: string }): string => {
  const map: Record<string, string> = {
${lines.join('\n')}
  }
  const src = map[page.filePath]
  if (!src) return ''
  if (src.startsWith('lang:')) return 'https://github.com/auto-stack/auto-lang/edit/master/' + src.slice(5)
  if (src.startsWith('book:')) return 'https://gitee.com/auto-stack/book/edit/master/' + src.slice(5)
  if (src.startsWith('site:')) return 'https://github.com/auto-stack/auto-lang/edit/master/website/' + src.slice(5)
  return ''
}
`
  fs.writeFileSync(filePath, content, 'utf-8')
  console.log(`  Generated author-source map: ${filePath} (${keys.length} entries)`)
}

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------

function main() {
  console.log('=== Auto Language Website Content Preparation ===\n')

  ensureDir(SIDEBAR_CONFIG_DIR)

  const docs = prepareDocs()
  const books = prepareBooks()

  // Generate index pages for docs and books
  generateDocsIndex(DOCS_DST_EN, 'en')
  generateDocsIndex(DOCS_DST_ZH, 'zh')
  generateBooksIndex(BOOKS_DST_EN, 'en')
  generateBooksIndex(BOOKS_DST_ZH, 'zh')

  // PLAN-718 T-02：学习入口数据校验——所有 href 必须落在真实生成页上
  validateHubLinks()

  console.log('\nGenerating sidebars...')

  const docsSidebarEn = buildDocsSidebar(docs.en, 'en')
  const docsSidebarZh = buildDocsSidebar(docs.zh, 'zh')
  const booksEn = buildBooksSidebar(books.en, 'en')
  const booksZh = buildBooksSidebar(books.zh, 'zh')

  writeSidebarConfig('docs-en', docsSidebarEn)
  writeSidebarConfig('docs-zh', docsSidebarZh)
  writeBooksSidebarConfigs('en', booksEn.sidebar, booksEn.byBook)
  writeBooksSidebarConfigs('zh', booksZh.sidebar, booksZh.byBook)

  // PLAN-718 T-03：作者源映射（docs/books 拷贝 + site 手工页）。
  collectSiteAuthorSource()
  writeAuthorSource()

  // Plan 581: books/docs 物化完成后生成 Playground Notes manifest（dev/build/deploy
  // 三态自动触发——本脚本位于三者构建链最前端）。book 仓缺失时采集脚本内部跳过
  // books 源并打 warning，不失败；manifest 为 gitignore 生成物，每次重建。
  console.log('\nGenerating playground notes manifest...')
  const notesScript = path.join(REPO_ROOT, 'scripts', 'build-playground-notes.mjs')
  const notesResult = spawnSync(process.execPath, [notesScript], { stdio: 'inherit' })
  if (notesResult.status !== 0) {
    console.warn('  playground notes manifest generation FAILED')
  }

  console.log('\nDone!')
}

main()
