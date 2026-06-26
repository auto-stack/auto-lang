#!/usr/bin/env node
// @auto-ui/widgets CLI — shadcn-style copy of AutoUI-generated Vue widgets.
// Plan 331, Phase 4. Minimal argv parsing, no runtime dependencies.

import { list } from './list.js'
import { add } from './add.js'

export interface ParsedArgs {
  values: Record<string, string | boolean>
  positional: string[]
}

/** Tiny argv parser: `--flag`, `--opt value`, `--opt=value`, positional. */
function parseArgs(argv: string[]): ParsedArgs {
  const values: Record<string, string | boolean> = {}
  const positional: string[] = []
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a.startsWith('--')) {
      const body = a.slice(2)
      const eq = body.indexOf('=')
      if (eq >= 0) {
        values[body.slice(0, eq)] = body.slice(eq + 1)
      } else if (i + 1 < argv.length && !argv[i + 1].startsWith('--')) {
        values[body] = argv[++i]
      } else {
        values[body] = true
      }
    } else {
      positional.push(a)
    }
  }
  return { values, positional }
}

function printHelp(): void {
  console.log(`@auto-ui/widgets — AutoUI-generated Vue 3 widget primitives

Usage:
  auto-ui list
  auto-ui add <widget> [options]

Commands:
  list                  List available widgets
  add <widget>          Copy a widget into src/components/ui/<widget>/

Options for 'add':
  --out <dir>           Destination root (default: src/components/ui)
  --no-install          Don't auto-install reka-ui
  --reka-ui <pkg>       Use a custom reka-ui fork (rewrites imports)
  --no-styles           Skip Tailwind / precompiled-CSS guidance
  -h, --help            Show this help`)
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2))
  if (args.values.help || args.values.h) {
    printHelp()
    return
  }
  const command = args.positional[0]
  switch (command) {
    case 'list':
      await list()
      break
    case 'add': {
      const widget = args.positional[1]
      if (!widget) {
        console.error("error: missing widget name. Run 'auto-ui list'.")
        process.exit(1)
      }
      const v = args.values
      await add(widget, {
        out: typeof v.out === 'string' ? v.out : undefined,
        // `--no-install` sets `no-install: true`; map to install=false.
        install: v['no-install'] === true ? false : undefined,
        rekaUi: typeof v['reka-ui'] === 'string' ? v['reka-ui'] : undefined,
        // `--no-styles` sets `no-styles: true`; map to styles=false.
        styles: v['no-styles'] === true ? false : undefined,
      })
      break
    }
    case undefined:
    case 'help':
      printHelp()
      break
    default:
      console.error(`error: unknown command '${command}'`)
      printHelp()
      process.exit(1)
  }
}

main().catch((e) => {
  console.error(e instanceof Error ? e.message : String(e))
  process.exit(1)
})
