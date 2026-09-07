#!/usr/bin/env node
/**
 * build-playground-notes.mjs — Playground Notes manifest 生成器（Plan 581，Playground 设计 §5）。
 *
 * 从仓内语料源确定性采集生成 website/public/playground-data/notes.json（schema v1：
 * groups/notes；不写 builtAt，保证 byte-identical）：
 *   - vm golden    crates/auto-lang/test/vm/NN_* 目录（.at 与同名 .expected.out 配对；
 *                  无 .expected.out 时回退 .expected.result——result 为终值、out 为 stdout，
 *                  二者皆为 golden 期望，任一存在即收录）
 *   - aavm corpus  crates/auto-lang/test/vm/aavm2/corpus_ 目录（每目录一组）
 *   - books        website/books/<book>/ch*.md 的 ```auto 围栏（.cn.md 不重复采；prepare-content
 *                  物化后运行；book 仓缺失时 warning 跳过，不失败）
 *   - playground-demo examples/playground-demo/（单文件直采 + main.at 项目目录）
 *   - parity       默认不采（Plan 581 T1 裁定：后置 582，见 scratch/p581/parity-survey.md）
 *
 * 用法：
 *   node scripts/build-playground-notes.mjs           # 生成 manifest
 *   node scripts/build-playground-notes.mjs --check   # 幂等 + 计数断言（CI 防采集回归）
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = path.resolve(__dirname, '..')
const VM_BASE = path.join(REPO_ROOT, 'crates/auto-lang/test/vm')
const DEMO_DIR = path.join(REPO_ROOT, 'examples/playground-demo')
const BOOKS_DIR = path.join(REPO_ROOT, 'website/books')
const OUT_DIR = path.join(REPO_ROOT, 'website/public/playground-data')
const OUT_FILE = path.join(OUT_DIR, 'notes.json')

// ── 通用工具 ──────────────────────────────────────────────────────────

function toPosix(p) {
  return p.split(path.sep).join('/')
}

function relRoot(p) {
  return toPosix(path.relative(REPO_ROOT, p))
}

function readIfExists(p) {
  try {
    return fs.readFileSync(p, 'utf8')
  } catch {
    return null
  }
}

function listDir(p) {
  try {
    return fs.readdirSync(p, { withFileTypes: true })
  } catch {
    return []
  }
}

/** 含 import/use 顶层声明的片段不可独立运行（多文件依赖，展示-only）。 */
function isStandalone(code) {
  return !/^[ \t]*(use|import)[ \t]/m.test(code)
}

/** 与 auto-playground examples.rs 的 display_name_from_stem 同风格（"01-hello" → "Hello"）。 */
function displayFromStem(stem) {
  const rest = stem.includes('-') ? stem.slice(stem.indexOf('-') + 1) : stem
  return rest
    .split('_')
    .filter(Boolean)
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(' ')
}

/** case 名（"001_for_range" / 无号 stem）→ 标题（"For Range"）；无号回退原 stem。 */
function titleFromCase(name) {
  const m = name.match(/^\d+_(.+)$/)
  if (!m) return name
  return displayFromStem(m[1])
}

function tagsFromCase(name) {
  const m = name.match(/^\d+_(.+)$/)
  const src = m ? m[1] : name
  return src.split(/[_-]+/).filter(Boolean).map((w) => w.toLowerCase())
}

// ── vm golden ─────────────────────────────────────────────────────────

// 目录号中文映射（Playground 设计 §5.1：组名取目录语义段；未命中回退目录名）。
const VM_GROUP_TITLES = {
  '01_basics': '基础',
  '02_bit_ops': '位运算',
  '03_variables': '变量',
  '04_control_flow': '控制流',
  '05_loops': '循环',
  '06_arrays': '数组',
  '07_objects': '对象',
  '08_strings': '字符串',
  '09_functions': '函数',
  '10_types': '类型',
  '11_compound_ops': '复合运算',
  '12_type_coercion': '类型强转',
  '13_collections': '集合',
  '14_borrow': '借用',
  '15_nested_mutation': '嵌套修改',
  '16_option_result': 'Option / Result',
  '17_cffi': 'C FFI',
  '17_modules': '模块',
  '18_ffi': 'FFI',
  '19_rust_std': 'Rust 标准库',
  '20_permission': '权限',
  '20_rust_ffi': 'Rust FFI',
  '21_conv': '类型转换',
  '22_generator': '生成器',
  '23_actor': 'Actor',
  '24_generics': '泛型',
  '25_method_u64': 'u64 方法',
  '26_str_method_on_heap': '堆字符串方法',
  '27_function_reference': '函数引用',
  '28_enum_methods': '枚举方法',
  '29_list_shims': '列表内置',
  '99_bootstrap': '自举',
  '99_idiom2': '惯用法 II',
  '99_idiom_probe': '惯用法探针',
  '99_misc': '杂项',
  '99_plan230': 'Plan 230',
  '99_plan231': 'Plan 231',
  '99_py_dispatch': 'Python 分派',
  '99_script_err': '脚本错误',
  '99_slice': '切片',
  '99_spec_dispatch': 'Spec 分派',
  '99_str_natives': '字符串原生',
}

/** 递归收集目录下全部 .at 文件（相对路径）。 */
function collectAtFiles(dir, acc = []) {
  for (const entry of listDir(dir)) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) collectAtFiles(full, acc)
    else if (entry.isFile() && entry.name.endsWith('.at')) acc.push(full)
  }
  return acc
}

function collectVmGolden() {
  const groups = []
  for (const entry of listDir(VM_BASE)) {
    if (!entry.isDirectory() || !/^\d\d_/.test(entry.name)) continue
    const groupDir = path.join(VM_BASE, entry.name)
    const semantic = entry.name.replace(/^\d+_/, '')
    const groupId = `vm-${semantic}`
    const notes = []
    for (const atPath of collectAtFiles(groupDir)) {
      const stem = atPath.slice(0, -'.at'.length)
      // 期望文件：.expected.out（stdout）优先，回退 .expected.result（终值）。
      const expectedPath = fs.existsSync(`${stem}.expected.out`)
        ? `${stem}.expected.out`
        : fs.existsSync(`${stem}.expected.result`)
          ? `${stem}.expected.result`
          : null
      if (!expectedPath) continue // 无配对者跳过（Playground 设计 §5.1）
      const code = readIfExists(atPath)
      const expectedOutput = readIfExists(expectedPath)
      if (code === null || expectedOutput === null) continue
      const caseName = path.basename(path.dirname(atPath))
      // case 目录名与 .at 同名时用目录号语义段，否则（平铺/多文件 case）用文件 stem。
      const noteName = caseName === path.basename(stem) ? caseName : path.basename(stem)
      notes.push({
        id: `${groupId}/${noteName}`,
        title: titleFromCase(noteName),
        sourceType: 'vm-golden',
        sourcePath: relRoot(atPath),
        kind: 'single',
        standalone: isStandalone(code),
        code,
        files: null,
        expectedOutput,
        description: null,
        tags: tagsFromCase(noteName),
      })
    }
    if (notes.length === 0) continue
    groups.push({
      id: groupId,
      title: VM_GROUP_TITLES[entry.name] ?? entry.name,
      order: parseInt(entry.name, 10),
      source: relRoot(groupDir),
      notes,
    })
  }
  return groups
}

// ── playground-demo ───────────────────────────────────────────────────

function collectDemo() {
  const notes = []
  const entries = listDir(DEMO_DIR)
  // 单文件直采（与 examples.rs load_from_dir 同序：按文件名排序）。
  const sorted = [...entries].sort((a, b) => a.name.localeCompare(b.name))
  for (const entry of sorted) {
    if (!entry.isFile() || !entry.name.endsWith('.at')) continue
    const full = path.join(DEMO_DIR, entry.name)
    const code = readIfExists(full)
    if (code === null) continue
    const stem = entry.name.slice(0, -'.at'.length)
    notes.push({
      id: `demo/${stem}`,
      title: displayFromStem(stem),
      sourceType: 'demo',
      sourcePath: relRoot(full),
      kind: 'single',
      standalone: isStandalone(code),
      code,
      files: null,
      expectedOutput: null,
      description: null,
      tags: tagsFromCase(stem),
    })
  }
  // 项目目录：含 main.at 者采全部 .at 文件（main.at 先、其余按文件名序——同 examples.rs）。
  for (const entry of sorted) {
    if (!entry.isDirectory()) continue
    const projectDir = path.join(DEMO_DIR, entry.name)
    const mainSrc = readIfExists(path.join(projectDir, 'main.at'))
    if (mainSrc === null) continue
    const files = [{ path: 'main.at', content: mainSrc }]
    const others = []
    for (const f of listDir(projectDir)) {
      if (!f.isFile() || !f.name.endsWith('.at') || f.name === 'main.at') continue
      const content = readIfExists(path.join(projectDir, f.name))
      if (content !== null) others.push({ path: f.name, content })
    }
    others.sort((a, b) => a.path.localeCompare(b.path))
    files.push(...others)
    notes.push({
      id: `demo/${entry.name}`,
      title: displayFromStem(entry.name),
      sourceType: 'demo',
      sourcePath: relRoot(path.join(projectDir, 'main.at')),
      kind: 'project',
      standalone: true,
      code: null,
      files,
      expectedOutput: null,
      description: null,
      tags: tagsFromCase(entry.name),
    })
  }
  if (notes.length === 0) return []
  return [
    {
      id: 'demo',
      title: 'Demo 示例',
      order: 400,
      source: relRoot(DEMO_DIR),
      notes,
    },
  ]
}

// ── aavm corpus ───────────────────────────────────────────────────────

// 每目录一组（Playground 设计 §5.1：m1 基础 / m2 … / a2r）。
const AAVM_GROUPS = [
  { dir: 'corpus_m1', id: 'aavm-m1', title: 'AAVM M1 基础' },
  { dir: 'corpus_m2', id: 'aavm-m2', title: 'AAVM M2 进阶' },
  { dir: 'corpus_m3', id: 'aavm-m3', title: 'AAVM M3' },
  { dir: 'corpus_m4', id: 'aavm-m4', title: 'AAVM M4 综合' },
  { dir: 'corpus_use', id: 'aavm-use', title: 'AAVM Use 语料' },
  { dir: 'corpus_a2r', id: 'aavm-a2r', title: 'AAVM A2R 语料' },
]

function collectAavm() {
  const groups = []
  AAVM_GROUPS.forEach((spec, i) => {
    const corpusDir = path.join(VM_BASE, 'aavm2', spec.dir)
    const notes = []
    for (const atPath of collectAtFiles(corpusDir)) {
      const code = readIfExists(atPath)
      if (code === null) continue
      // 嵌套 case 目录（corpus_use 的 main/db、corpus_a2r 的 gNN 子目录）：
      // 目录名与 stem 同名时折叠为目录名，否则用相对路径（保 id 唯一且稳定）。
      const rel = toPosix(path.relative(corpusDir, atPath))
      const stem = rel.slice(0, -'.at'.length)
      const parts = stem.split('/')
      const noteName =
        parts.length >= 2 && parts[parts.length - 1] === parts[parts.length - 2]
          ? parts.slice(0, -1).join('/')
          : stem
      notes.push({
        id: `${spec.id}/${noteName}`,
        title: parts[parts.length - 1],
        sourceType: 'aavm-corpus',
        sourcePath: relRoot(atPath),
        kind: 'single',
        standalone: isStandalone(code),
        code,
        files: null,
        // 设计 §5.2：expectedOutput 仅 vm-golden（corpus_a2r 的 .expected.out 不入）。
        expectedOutput: null,
        description: null,
        tags: parts.filter((p) => p !== '.').map((p) => p.toLowerCase()),
      })
    }
    if (notes.length === 0) return
    notes.sort((a, b) => a.id.localeCompare(b.id))
    groups.push({ id: spec.id, title: spec.title, order: 200 + i, source: relRoot(corpusDir), notes })
  })
  return groups
}

// ── books（```auto 围栏；.cn.md 不重复采，英文版为源）─────────────────

/** 逐行提取裸 ```auto 围栏（带属性变体如 ```auto,ignore,does_not_compile 不采——明示不可编译示例）。 */
function extractAutoFences(md) {
  const fences = []
  const lines = md.split('\n')
  let inFence = false
  let buf = []
  for (const line of lines) {
    if (!inFence) {
      if (/^```auto\s*$/.test(line)) {
        inFence = true
        buf = []
      }
    } else if (/^```\s*$/.test(line)) {
      fences.push(buf.join('\n'))
      inFence = false
      buf = []
    } else {
      buf.push(line)
    }
  }
  return fences // 未闭合尾部忽略
}

function collectBooks() {
  if (!fs.existsSync(BOOKS_DIR)) {
    console.warn('[build-playground-notes] books 未物化（website/books 缺失）——跳过 books 源。')
    return []
  }
  const bookDirs = listDir(BOOKS_DIR)
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort((a, b) => a.localeCompare(b))
  if (bookDirs.length === 0) {
    console.warn('[build-playground-notes] website/books 为空（book 仓缺失或未物化）——跳过 books 源。')
    return []
  }
  const groups = []
  bookDirs.forEach((bookName, i) => {
    const bookDir = path.join(BOOKS_DIR, bookName)
    // 仅 ch*.md；.cn.md 不重复采（Playground 设计 §5.1）。
    const chapterFiles = listDir(bookDir)
      .filter((e) => e.isFile() && /^ch.*\.md$/.test(e.name) && !e.name.endsWith('.cn.md'))
      .map((e) => e.name)
      .sort((a, b) => a.localeCompare(b))
    const notes = []
    for (const fileName of chapterFiles) {
      const md = readIfExists(path.join(bookDir, fileName))
      if (md === null) continue
      const stem = fileName.slice(0, -'.md'.length)
      extractAutoFences(md).forEach((code, idx) => {
        if (!code.trim()) return
        notes.push({
          id: `book-${bookName}/${stem}-${String(idx + 1).padStart(2, '0')}`,
          title: `${stem} · 块 ${String(idx + 1).padStart(2, '0')}`,
          sourceType: 'book',
          sourcePath: relRoot(path.join(bookDir, fileName)),
          kind: 'fence',
          standalone: isStandalone(code),
          code,
          files: null,
          expectedOutput: null,
          description: null,
          tags: ['book', bookName],
        })
      })
    }
    if (notes.length === 0) return
    notes.sort((a, b) => a.id.localeCompare(b.id))
    groups.push({ id: `book-${bookName}`, title: bookName, order: 300 + i, source: relRoot(bookDir), notes })
  })
  return groups
}

// ── manifest 组装（确定性：groups 按 order/id 排序、notes 按 id 排序、不写时间戳）──

function buildManifest() {
  const groups = [...collectVmGolden(), ...collectAavm(), ...collectBooks(), ...collectDemo()]
  for (const g of groups) g.notes.sort((a, b) => a.id.localeCompare(b.id))
  groups.sort((a, b) => a.order - b.order || a.id.localeCompare(b.id))
  return { version: 1, groups }
}

// ── main ──────────────────────────────────────────────────────────────

function countByPrefix(manifest, p) {
  return manifest.groups
    .filter((g) => g.id.startsWith(p))
    .reduce((s, g) => s + g.notes.length, 0)
}

/**
 * --check：内存二次生成幂等比对 + 计数断言（非磁盘 diff——books 为 gitignore 生成物）。
 * 任一失败非零退出（CI 防采集规则回归）。计数基线见 Plan 581 待澄清④（demo 29→28 修正）。
 */
function runCheck() {
  const failures = []
  const first = buildManifest()
  const second = buildManifest()
  if (JSON.stringify(first) !== JSON.stringify(second)) {
    failures.push('幂等失败：两次内存生成不一致')
  }
  const vmNotes = first.groups
    .filter((g) => g.id.startsWith('vm-'))
    .flatMap((g) => g.notes)
  if (vmNotes.length < 300) failures.push(`vm-golden ${vmNotes.length} < 300`)
  const vmNoExpected = vmNotes.filter((n) => typeof n.expectedOutput !== 'string' || n.expectedOutput === '')
  if (vmNoExpected.length > 0) failures.push(`vm-golden ${vmNoExpected.length} 条缺 expectedOutput（如 ${vmNoExpected[0].id}）`)
  const aavmCount = countByPrefix(first, 'aavm-')
  if (aavmCount < 158) failures.push(`aavm ${aavmCount} < 158`)
  const demoNotes = first.groups.find((g) => g.id === 'demo')?.notes ?? []
  if (demoNotes.length < 28) failures.push(`demo ${demoNotes.length} < 28`)
  const demoProjects = demoNotes.filter((n) => n.kind === 'project').length
  if (demoProjects < 4) failures.push(`demo kind=project ${demoProjects} < 4`)
  const bookCount = countByPrefix(first, 'book-')
  if (bookCount === 0 && first.groups.some((g) => g.id.startsWith('book-'))) {
    failures.push('book 组存在但 0 笔记')
  }
  if (failures.length > 0) {
    console.error(`[build-playground-notes] --check FAILED:\n  - ${failures.join('\n  - ')}`)
    process.exit(1)
  }
  console.log(
    `[build-playground-notes] --check OK: vm=${vmNotes.length} aavm=${aavmCount} demo=${demoNotes.length}(project=${demoProjects}) book=${bookCount}${bookCount === 0 ? '（未物化，跳过）' : ''}`
  )
}

function writeManifest(manifest) {
  fs.mkdirSync(OUT_DIR, { recursive: true })
  fs.writeFileSync(OUT_FILE, JSON.stringify(manifest, null, 2) + '\n', 'utf8')
}

if (process.argv.includes('--check')) {
  runCheck()
} else {
  const manifest = buildManifest()
  writeManifest(manifest)
  const lines = [`manifest → ${relRoot(OUT_FILE)}`]
  for (const g of manifest.groups) lines.push(`  ${g.id.padEnd(22)} ${String(g.notes.length).padStart(4)}  ${g.title}`)
  lines.push(
    `vm total: ${countByPrefix(manifest, 'vm-')}, aavm total: ${countByPrefix(manifest, 'aavm-')}, book total: ${countByPrefix(manifest, 'book-')}, demo total: ${countByPrefix(manifest, 'demo')}`
  )
  console.log(lines.join('\n'))
}
