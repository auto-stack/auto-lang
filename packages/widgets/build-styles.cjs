// Build script: scan registry/ for utility classes and emit a minified,
// self-contained dist/styles.css (Plan 331, Phase 6).
//
//   node build-styles.cjs
//
// Produces dist/styles.css suitable for `import '@auto-ui/widgets/styles.css'`.
// CommonJS (.cjs) because package.json sets "type": "module".

const { spawnSync } = require('node:child_process')
const fs = require('node:fs')
const path = require('node:path')

const root = __dirname
const outDir = path.join(root, 'dist')
fs.mkdirSync(outDir, { recursive: true })

const input = path.join(root, 'src', 'input.css')
const output = path.join(outDir, 'styles.css')

// Use the local tailwindcss CLI (devDependency) to avoid network lookups.
const bin = process.platform === 'win32'
  ? path.join(root, 'node_modules', '.bin', 'tailwindcss.cmd')
  : path.join(root, 'node_modules', '.bin', 'tailwindcss')

const result = spawnSync(
  bin,
  ['-i', input, '-o', output, '--minify', '--config', path.join(root, 'tailwind.config.cjs')],
  { stdio: 'inherit', shell: true, cwd: root },
)

if (result.status !== 0) {
  console.error(`tailwindcss exited with status ${result.status}`)
  process.exit(result.status ?? 1)
}

const size = fs.statSync(output).size
console.log(`wrote ${path.relative(process.cwd(), output)} (${size} bytes)`)
