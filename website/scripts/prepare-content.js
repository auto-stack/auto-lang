/**
 * Content preparation script for Auto Language website.
 *
 * This script copies documentation and books from their source locations
 * into the website directory and generates VitePress sidebar configs.
 */

import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const WEBSITE_ROOT = path.resolve(__dirname, '..')
const REPO_ROOT = path.resolve(WEBSITE_ROOT, '..')
const BOOK_ROOT = path.resolve(REPO_ROOT, '..', 'book')

const DOCS_SRC = path.join(REPO_ROOT, 'docs')
const DOCS_DST_EN = path.join(WEBSITE_ROOT, 'docs')
const DOCS_DST_ZH = path.join(WEBSITE_ROOT, 'zh', 'docs')

const BOOKS_DST_EN = path.join(WEBSITE_ROOT, 'books')
const BOOKS_DST_ZH = path.join(WEBSITE_ROOT, 'zh', 'books')

const SIDEBAR_CONFIG_DIR = path.join(WEBSITE_ROOT, '.vitepress', 'config')

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

function preprocessMarkdown(content, bookDir = null) {
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
])

function shouldIncludeDoc(relPath) {
  const topDir = relPath.split(path.sep)[0]
  // Include root-level files and whitelisted directories
  return !topDir || DOCS_INCLUDE.has(topDir)
}

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

  walkDir(DOCS_SRC, (fullPath, name) => {
    const relPath = path.relative(DOCS_SRC, fullPath)
    if (!shouldIncludeDoc(relPath)) return

    // Plan 244: Copy .at files from tour/ directory (needed for Listing reference)
    if (name.endsWith('.at') && relPath.startsWith('tour' + path.sep)) {
      const dstPath = path.join(DOCS_DST_EN, relPath)
      copyFile(fullPath, dstPath)
      return
    }

    if (!name.endsWith('.md')) return

    // Always copy to EN
    // Plan 244: Pass docs/ root as bookDir for tour/ files so <Listing> resolves.
    // Plan 358: also for script-to-ship/ files (ScriptShipView listings).
    const isTour = relPath.startsWith('tour' + path.sep) || relPath.startsWith('script-to-ship' + path.sep)
    const docBookDir = isTour ? DOCS_SRC : null
    const enDstPath = path.join(DOCS_DST_EN, relPath)
    copyFile(fullPath, enDstPath, docBookDir)
    enFiles.push(relPath)

    if (name.endsWith('.cn.md')) {
      // Chinese version goes to ZH without .cn prefix
      const zhDstPath = path.join(DOCS_DST_ZH, relPath.replace(/\.cn\.md$/, '.md'))
      copyFile(fullPath, zhDstPath, docBookDir)
      zhFiles.push(relPath.replace(/\.cn\.md$/, '.md'))
    } else {
      // Non-Chinese files also go to ZH as fallback
      const zhDstPath = path.join(DOCS_DST_ZH, relPath)
      copyFile(fullPath, zhDstPath, docBookDir)
      zhFiles.push(relPath)
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

      if (name.endsWith('.cn.md')) {
        // Chinese version goes to ZH without .cn suffix
        const zhDstPath = path.join(BOOKS_DST_ZH, book, relPath.replace(/\.cn\.md$/, '.md'))
        copyFile(fullPath, zhDstPath, srcDir)
        zhFiles[book].push(relPath.replace(/\.cn\.md$/, '.md'))
      } else if (!hasCnVersion.has(relPath)) {
        // Only copy non-Chinese files to ZH if there's no .cn.md version
        const zhDstPath = path.join(BOOKS_DST_ZH, book, relPath)
        copyFile(fullPath, zhDstPath, srcDir)
        zhFiles[book].push(relPath)
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

  return toSidebarItems(tree)
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
    const match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
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

  let content = `---\ntitle: ${bookTitle}\n---\n\n# ${bookTitle}\n\n`

  if (fs.existsSync(summaryPath)) {
    const summary = fs.readFileSync(summaryPath, 'utf-8')
    content += `## ${tocHeading}\n\n`
    const lines = summary.split('\n')
    for (const line of lines) {
      const match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
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

function generateDocsIndex(docsDir, lang) {
  ensureDir(docsDir)
  const indexPath = path.join(docsDir, 'index.md')
  if (fs.existsSync(indexPath)) return

  const content = lang === 'zh'
    ? `---\ntitle: 文档\n---\n\n# 文档\n\n欢迎使用 Auto 语言文档。这里提供从语言规范到高级指南的所有内容。\n\n## 快速链接\n\n- [语言语法](./syntax) — 快速语法参考\n- [语言规范](./language/specification) — 完整语言规范\n- [路线图](./roadmap) — 项目路线图和未来计划\n- [迁移指南](./migration-guide) — 从其他语言迁移\n\n## 章节\n\n### [设计](./design/)\n架构和语言设计文档。\n\n### [语言](./language/)\n语言规范、语法和特性文档。\n\n### [教程](./tutorials/)\n学习 Auto 的逐步指南。\n\n### [指南](./guides/)\n特定用例的实用指南。\n\n### [架构](./architecture/)\n系统架构和内部设计文档。\n\n### [CLI](./cli/)\n命令行接口文档。\n\n### [示例](./examples/)\n示例项目和代码样本。\n`
    : `---\ntitle: Documentation\n---\n\n# Documentation\n\nWelcome to the Auto Language documentation. Here you'll find everything from language specifications to advanced guides.\n\n## Quick Links\n\n- [Language Syntax](./syntax) — Quick syntax reference\n- [Language Specification](./language/specification) — Full language spec\n- [Roadmap](./roadmap) — Project roadmap and future plans\n- [Migration Guide](./migration-guide) — Migrating from other languages\n\n## Sections\n\n### [Design](./design/)\nArchitecture and language design documents.\n\n### [Language](./language/)\nLanguage specification, syntax, and feature documentation.\n\n### [Tutorials](./tutorials/)\nStep-by-step guides for learning Auto.\n\n### [Guides](./guides/)\nPractical guides for specific use cases.\n\n### [Architecture](./architecture/)\nSystem architecture and internal design docs.\n\n### [CLI](./cli/)\nCommand-line interface documentation.\n\n### [Examples](./examples/)\nExample projects and code samples.\n`

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function generateBooksIndex(booksDir, lang) {
  ensureDir(booksDir)
  const indexPath = path.join(booksDir, 'index.md')
  if (fs.existsSync(indexPath)) return

  const content = lang === 'zh'
    ? `---\ntitle: 教程\n---\n\n# 教程\n\n学习 Auto 的教程集合，涵盖从初学者教程到高级系统编程的所有内容。\n\n## [Auto 编程语言](./tapl/)\nAuto 主书 — 全面的语言介绍。\n\n## [Auto版Rust Book](./rust/)\n通过与 Rust 比较来学习 Auto。\n\n## [Auto版TypeScript Handbook](./typescript/)\n面向 TypeScript 开发者的 Auto 手册。\n\n## [Auto版TypeScript DeepDive](./typescript-deepdive/)\n深入比较 Auto 和 TypeScript 的类型系统。\n\n## [Auto版The Little Book of C](./little-c/)\n通过 C 语言概念温和地介绍 Auto。\n\n## [Auto版Modern C](./modern-c/)\n使用 Auto 和 C 进行现代系统编程。\n\n## [Auto版A Byte of Python](./byte-of-python/)\n受《A Byte of Python》启发的初学者友好教程。\n\n## [Auto版Think Python](./think-python/)\n基于《Think Python》的 Auto 计算思维。\n`
    : `---\ntitle: Tutorials\n---\n\n# Tutorials\n\nA collection of tutorials for learning Auto, covering everything from beginner tutorials to advanced systems programming.\n\n## [The Auto Programming Language](./tapl/)\nThe main Auto tutorial — a comprehensive introduction to the language.\n\n## [Auto vs Rust](./rust/)\nLearn Auto by comparing it with Rust.\n\n## [Auto vs TypeScript](./typescript/)\nA handbook for TypeScript developers learning Auto.\n\n## [Auto vs TypeScript DeepDive](./typescript-deepdive/)\nDeep dive into Auto's type system compared to TypeScript.\n\n## [Auto vs The Little Book of C](./little-c/)\nA gentle introduction to Auto through C concepts.\n\n## [Auto vs Modern C](./modern-c/)\nModern systems programming with Auto and C.\n\n## [A Byte of Auto](./byte-of-python/)\nA beginner-friendly tutorial inspired by "A Byte of Python".\n\n## [Think Auto](./think-python/)\nComputational thinking with Auto, based on "Think Python".\n`

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function prefixBookLinks(items, book) {
  return items.map(item => ({
    ...item,
    link: item.link ? `${book}/${item.link}` : undefined,
    items: item.items ? prefixBookLinks(item.items, book) : undefined,
  }))
}

function buildBooksSidebar(bookFiles, lang = 'en') {
  const sidebar = []
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
      sidebar.push({
        text: bookTitle,
        link: `${book}/`,
        collapsed: book !== 'tapl',
        items: prefixBookLinks(items, book),
      })
    } else {
      // Fallback: list all markdown files
      const leafs = files
        .filter((f) => f.endsWith('.md') && !f.endsWith('SUMMARY.md') && f !== 'index.md')
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

      sidebar.push({
        text: bookTitle,
        link: `${book}/`,
        collapsed: book !== 'tapl',
        items: leafs,
      })
    }
  }

  return sidebar
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

  console.log('\nGenerating sidebars...')

  const docsSidebarEn = buildDocsSidebar(docs.en, 'en')
  const docsSidebarZh = buildDocsSidebar(docs.zh, 'zh')
  const booksSidebarEn = buildBooksSidebar(books.en, 'en')
  const booksSidebarZh = buildBooksSidebar(books.zh, 'zh')

  writeSidebarConfig('docs-en', docsSidebarEn)
  writeSidebarConfig('docs-zh', docsSidebarZh)
  writeSidebarConfig('books-en', booksSidebarEn)
  writeSidebarConfig('books-zh', booksSidebarZh)

  console.log('\nDone!')
}

main()
