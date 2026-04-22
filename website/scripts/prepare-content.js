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
  // If file contains path separators, use it directly as directory
  if (attrs.file && (attrs.file.includes('/') || attrs.file.includes('\\'))) {
    return path.join(bookDir, attrs.file)
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

function copyFile(src, dst, bookDir = null) {
  ensureDir(path.dirname(dst))
  if (src.endsWith('.md')) {
    const content = fs.readFileSync(src, 'utf-8')
    const processed = preprocessMarkdown(content, bookDir)
    fs.writeFileSync(dst, processed, 'utf-8')
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
    if (!name.endsWith('.md')) return
    const relPath = path.relative(DOCS_SRC, fullPath)
    if (!shouldIncludeDoc(relPath)) return

    if (name.endsWith('.cn.md')) {
      const dstPath = path.join(DOCS_DST_ZH, relPath.replace(/\.cn\.md$/, '.md'))
      copyFile(fullPath, dstPath)
      zhFiles.push(relPath.replace(/\.cn\.md$/, '.md'))
    } else {
      const dstPath = path.join(DOCS_DST_EN, relPath)
      copyFile(fullPath, dstPath)
      enFiles.push(relPath)
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

    walkDir(srcDir, (fullPath, name) => {
      if (!name.endsWith('.md')) return
      const relPath = path.relative(srcDir, fullPath)

      if (name.endsWith('.cn.md')) {
        const dstPath = path.join(BOOKS_DST_ZH, book, relPath.replace(/\.cn\.md$/, '.md'))
        copyFile(fullPath, dstPath, srcDir)
        zhFiles[book].push(relPath.replace(/\.cn\.md$/, '.md'))
      } else {
        const dstPath = path.join(BOOKS_DST_EN, book, relPath)
        copyFile(fullPath, dstPath, srcDir)
        enFiles[book].push(relPath)
      }
    })

    // Generate index.md for EN book
    const enSummaryPath = path.join(BOOKS_DST_EN, book, 'SUMMARY.md')
    if (!enFiles[book].includes('index.md') && !enFiles[book].includes('README.md')) {
      generateBookIndex(path.join(BOOKS_DST_EN, book), enSummaryPath)
      enFiles[book].push('index.md')
    }

    // Generate index.md for ZH book
    const zhSummaryPath = path.join(BOOKS_DST_ZH, book, 'SUMMARY.md')
    if (!zhFiles[book].includes('index.md') && !zhFiles[book].includes('README.md')) {
      generateBookIndex(path.join(BOOKS_DST_ZH, book), zhSummaryPath)
      zhFiles[book].push('index.md')
    }

    console.log(`  ${book}: ${enFiles[book].length} EN, ${zhFiles[book].length} ZH`)
  }

  return { en: enFiles, zh: zhFiles }
}

// ------------------------------------------------------------------
// Sidebar Generation — Docs
// ------------------------------------------------------------------

function buildDocsSidebar(files) {
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
          text: title.charAt(0).toUpperCase() + title.slice(1),
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
        text: title.charAt(0).toUpperCase() + title.slice(1),
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

function parseSummary(summaryPath) {
  if (!fs.existsSync(summaryPath)) return null

  const content = fs.readFileSync(summaryPath, 'utf-8')
  const lines = content.split('\n')
  const root = []
  const stack = [{ items: root, depth: -1 }]

  for (const line of lines) {
    const match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
    if (!match) continue

    const depth = match[1].length
    const text = match[2]
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

function generateBookIndex(bookDir, summaryPath) {
  const bookName = path.basename(bookDir)
  const bookTitle = bookName.replace(/-/g, ' ').replace(/^\w/, (c) => c.toUpperCase())
  const indexPath = path.join(bookDir, 'index.md')

  let content = `---\ntitle: ${bookTitle}\n---\n\n# ${bookTitle}\n\n`

  if (fs.existsSync(summaryPath)) {
    const summary = fs.readFileSync(summaryPath, 'utf-8')
    content += '## Table of Contents\n\n'
    const lines = summary.split('\n')
    for (const line of lines) {
      const match = line.match(/^(\s*)-\s*\[([^\]]+)\]\s*\(([^)]+)\)/)
      if (match) {
        const depth = match[1].length
        const text = match[2]
        const link = match[3].replace(/\.md$/, '')
        const indent = '  '.repeat(depth / 2)
        content += `${indent}- [${text}](./${link})\n`
      } else if (line.trim().startsWith('#')) {
        const heading = line.trim().replace(/^#+\s*/, '')
        content += `\n### ${heading}\n\n`
      }
    }
  } else {
    content += 'Chapters will be listed here.\n'
  }

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function generateDocsIndex(docsDir, lang) {
  const indexPath = path.join(docsDir, 'index.md')
  if (fs.existsSync(indexPath)) return

  const content = lang === 'zh'
    ? `---\ntitle: 文档\n---\n\n# 文档\n\n欢迎使用 Auto 语言文档。这里提供从语言规范到高级指南的所有内容。\n\n## 快速链接\n\n- [语言语法](./syntax) — 快速语法参考\n- [语言规范](./language/specification) — 完整语言规范\n- [路线图](./roadmap) — 项目路线图和未来计划\n- [迁移指南](./migration-guide) — 从其他语言迁移\n\n## 章节\n\n### [设计](./design/)\n架构和语言设计文档。\n\n### [语言](./language/)\n语言规范、语法和特性文档。\n\n### [教程](./tutorials/)\n学习 Auto 的逐步指南。\n\n### [指南](./guides/)\n特定用例的实用指南。\n\n### [架构](./architecture/)\n系统架构和内部设计文档。\n\n### [CLI](./cli/)\n命令行接口文档。\n\n### [示例](./examples/)\n示例项目和代码样本。\n`
    : `---\ntitle: Documentation\n---\n\n# Documentation\n\nWelcome to the Auto Language documentation. Here you'll find everything from language specifications to advanced guides.\n\n## Quick Links\n\n- [Language Syntax](./syntax) — Quick syntax reference\n- [Language Specification](./language/specification) — Full language spec\n- [Roadmap](./roadmap) — Project roadmap and future plans\n- [Migration Guide](./migration-guide) — Migrating from other languages\n\n## Sections\n\n### [Design](./design/)\nArchitecture and language design documents.\n\n### [Language](./language/)\nLanguage specification, syntax, and feature documentation.\n\n### [Tutorials](./tutorials/)\nStep-by-step guides for learning Auto.\n\n### [Guides](./guides/)\nPractical guides for specific use cases.\n\n### [Architecture](./architecture/)\nSystem architecture and internal design docs.\n\n### [CLI](./cli/)\nCommand-line interface documentation.\n\n### [Examples](./examples/)\nExample projects and code samples.\n`

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function generateBooksIndex(booksDir, lang) {
  const indexPath = path.join(booksDir, 'index.md')
  if (fs.existsSync(indexPath)) return

  const content = lang === 'zh'
    ? `---\ntitle: 书籍\n---\n\n# 书籍\n\n学习 Auto 的书籍集合，涵盖从初学者教程到高级系统编程的所有内容。\n\n## [The Auto Programming Language](./tapl/)\nAuto 主书 — 全面的语言介绍。\n\n## [Auto vs Rust](./rust/)\n通过与 Rust 比较来学习 Auto。\n\n## [Auto vs TypeScript](./typescript/)\n面向 TypeScript 开发者的 Auto 手册。\n\n## [Auto vs TypeScript DeepDive](./typescript-deepdive/)\n深入比较 Auto 和 TypeScript 的类型系统。\n\n## [Auto vs The Little Book of C](./little-c/)\n通过 C 语言概念温和地介绍 Auto。\n\n## [Auto vs Modern C](./modern-c/)\n使用 Auto 和 C 进行现代系统编程。\n\n## [A Byte of Auto](./byte-of-python/)\n受《A Byte of Python》启发的初学者友好书籍。\n\n## [Think Auto](./think-python/)\n基于《Think Python》的 Auto 计算思维。\n`
    : `---\ntitle: Books\n---\n\n# Books\n\nA collection of books for learning Auto, covering everything from beginner tutorials to advanced systems programming.\n\n## [The Auto Programming Language](./tapl/)\nThe main Auto book — a comprehensive introduction to the language.\n\n## [Auto vs Rust](./rust/)\nLearn Auto by comparing it with Rust.\n\n## [Auto vs TypeScript](./typescript/)\nA handbook for TypeScript developers learning Auto.\n\n## [Auto vs TypeScript DeepDive](./typescript-deepdive/)\nDeep dive into Auto's type system compared to TypeScript.\n\n## [Auto vs The Little Book of C](./little-c/)\nA gentle introduction to Auto through C concepts.\n\n## [Auto vs Modern C](./modern-c/)\nModern systems programming with Auto and C.\n\n## [A Byte of Auto](./byte-of-python/)\nA beginner-friendly book inspired by "A Byte of Python".\n\n## [Think Auto](./think-python/)\nComputational thinking with Auto, based on "Think Python".\n`

  fs.writeFileSync(indexPath, content, 'utf-8')
}

function prefixBookLinks(items, book) {
  return items.map(item => ({
    ...item,
    link: item.link ? `${book}/${item.link}` : undefined,
    items: item.items ? prefixBookLinks(item.items, book) : undefined,
  }))
}

function buildBooksSidebar(bookFiles) {
  const sidebar = []

  for (const book of BOOKS) {
    const files = bookFiles[book] || []
    if (files.length === 0) continue

    const summaryPath = path.join(BOOKS_DST_EN, book, 'SUMMARY.md')
    const items = parseSummary(summaryPath)

    const bookTitle = book
      .replace(/-/g, ' ')
      .replace(/^\w/, (c) => c.toUpperCase())

    // Generate index.md if it doesn't exist
    if (!files.includes('index.md') && !files.includes('README.md')) {
      generateBookIndex(path.join(BOOKS_DST_EN, book), summaryPath)
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
            text: title.charAt(0).toUpperCase() + title.slice(1),
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

  const docsSidebarEn = buildDocsSidebar(docs.en)
  const docsSidebarZh = buildDocsSidebar(docs.zh)
  const booksSidebarEn = buildBooksSidebar(books.en)
  const booksSidebarZh = buildBooksSidebar(books.zh)

  writeSidebarConfig('docs-en', docsSidebarEn)
  writeSidebarConfig('docs-zh', docsSidebarZh)
  writeSidebarConfig('books-en', booksSidebarEn)
  writeSidebarConfig('books-zh', booksSidebarZh)

  console.log('\nDone!')
}

main()
