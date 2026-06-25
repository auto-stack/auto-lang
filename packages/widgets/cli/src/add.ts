// `auto-ui add <widget>` — copy a widget into the consumer's project and wire
// up reka-ui + Tailwind. Plan 331, Phase 4 (Tasks 4.2 / 4.3 / 4.4).

import {
  existsSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
  mkdirSync,
} from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { resolveRegistryWidget } from './paths.js'

interface AddOptions {
  out?: string | boolean
  install?: string | boolean // negated as --no-install => install === false
  rekaUi?: string | boolean
  styles?: string | boolean // --no-styles => styles === false
}

/** Detect the consumer's package manager (pnpm > bun > npm). */
function detectPackageManager(cwd: string): { cmd: string; addArgs: string[] } {
  const has = (f: string) => existsSync(join(cwd, f))
  if (has('pnpm-lock.yaml')) return { cmd: 'pnpm', addArgs: ['add'] }
  if (has('bun.lockb') || has('bun.lock')) return { cmd: 'bun', addArgs: ['add'] }
  // yarn support
  if (has('yarn.lock')) return { cmd: 'yarn', addArgs: ['add'] }
  return { cmd: 'npm', addArgs: ['install'] }
}

/** Does the consumer's package.json declare a given dependency? */
function hasDependency(cwd: string, name: string): boolean {
  const pj = join(cwd, 'package.json')
  if (!existsSync(pj)) return false
  try {
    const pkg = JSON.parse(readFileSync(pj, 'utf8'))
    return Boolean(
      (pkg.dependencies && pkg.dependencies[name]) ||
        (pkg.devDependencies && pkg.devDependencies[name]) ||
        (pkg.peerDependencies && pkg.peerDependencies[name]),
    )
  } catch {
    return false
  }
}

/** Recursively copy a directory tree. */
function copyDir(src: string, dst: string): void {
  mkdirSync(dst, { recursive: true })
  for (const entry of readdirSync(src, { withFileTypes: true })) {
    const s = join(src, entry.name)
    const d = join(dst, entry.name)
    if (entry.isDirectory()) {
      copyDir(s, d)
    } else {
      let content = readFileSync(s, 'utf8')
      // content rewrites happen in the caller (reka-ui rewrite) — here just copy
      writeFileSync(d, content)
    }
  }
}

/** Find a tailwind config in cwd, if any. */
function findTailwindConfig(cwd: string): string | null {
  const candidates = [
    'tailwind.config.js',
    'tailwind.config.cjs',
    'tailwind.config.mjs',
    'tailwind.config.ts',
  ]
  for (const c of candidates) {
    if (existsSync(join(cwd, c))) return c
  }
  return null
}

/** PascalCase a kebab/lower widget key ('button' -> 'Button'). */
function pascalCase(name: string): string {
  return name
    .split(/[-_]/)
    .map((p) => (p ? p[0].toUpperCase() + p.slice(1) : ''))
    .join('')
}

export async function add(widget: string, opts: AddOptions): Promise<void> {
  const cwd = process.cwd()
  const registryWidget = resolveRegistryWidget(widget)

  // --- Task 4.2: copy the widget files -------------------------------------
  if (!existsSync(registryWidget) || !statSync(registryWidget).isDirectory()) {
    console.error(
      `error: unknown widget '${widget}'. Run 'auto-ui list' for available widgets.`,
    )
    process.exit(1)
  }

  const outRoot =
    typeof opts.out === 'string' ? resolve(cwd, opts.out) : join(cwd, 'src', 'components', 'ui')
  const dest = join(outRoot, widget)

  // Rewrite reka-ui import path if a custom fork is requested (Task 4.3).
  const rekaPkg = typeof opts.rekaUi === 'string' ? opts.rekaUi : null

  mkdirSync(dest, { recursive: true })
  for (const entry of readdirSync(registryWidget, { withFileTypes: true })) {
    const srcPath = join(registryWidget, entry.name)
    const dstPath = join(dest, entry.name)
    if (entry.isDirectory()) {
      copyDir(srcPath, dstPath)
    } else {
      let content = readFileSync(srcPath, 'utf8')
      if (rekaPkg) content = content.replace(/'reka-ui'/g, `'${rekaPkg}'`)
      writeFileSync(dstPath, content)
    }
  }
  console.log(`copied ${widget} -> ${resolve(cwd, dest)}`)

  // --- Task 4.3: auto-install reka-ui --------------------------------------
  const installDisabled = opts.install === false
  if (!installDisabled) {
    const effectivePkg = rekaPkg ?? 'reka-ui'
    if (rekaPkg) {
      // custom fork: always install it (we can't know if it's present by name)
      installPackage(cwd, effectivePkg)
    } else if (!hasDependency(cwd, 'reka-ui')) {
      installPackage(cwd, 'reka-ui')
    } else {
      console.log('reka-ui already present — skipping install')
    }
  } else {
    console.log('--no-install: skipping reka-ui install')
  }

  // --- Task 4.4: Tailwind guidance -----------------------------------------
  if (opts.styles === false) {
    console.log('--no-styles: skipping Tailwind guidance')
    return
  }
  const tailwind = findTailwindConfig(cwd)
  const widgetFile = `${pascalCase(widget)}.vue`
  if (tailwind) {
    console.log(
      `\nTailwind detected (${tailwind}). Make sure your 'content' globs include the copied component, e.g.:\n` +
        `  content: ["./src/components/ui/**/*.{vue,ts}", ...]\n` +
        `Do NOT import '@auto-ui/widgets/styles.css' — you run your own Tailwind.`,
    )
  } else {
    console.log(
      `\nNo Tailwind config detected. For zero-config styling, import the precompiled stylesheet once in your app entry:\n` +
        `  import '@auto-ui/widgets/styles.css'\n` +
        `(Do this instead of running your own Tailwind — pick exactly one path.)`,
    )
  }
  // mention the widget file name for convenience
  void widgetFile
}

function installPackage(cwd: string, pkg: string): void {
  const { cmd, addArgs } = detectPackageManager(cwd)
  console.log(`installing ${pkg} via ${cmd}...`)
  // shell:true so Windows resolves npm.cmd / pnpm.cmd / yarn.cmd.
  const result = spawnSync(cmd, [...addArgs, pkg], { cwd, stdio: 'inherit', shell: true })
  if (result.status !== 0) {
    console.error(
      `warning: ${cmd} ${addArgs} ${pkg} exited with status ${result.status ?? 'null'}${result.signal ? ` (${result.signal})` : ''}`,
    )
  }
}
