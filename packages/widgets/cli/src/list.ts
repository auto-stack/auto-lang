// `auto-ui list` — print available widgets from the committed registry.
// Plan 331, Phase 4 (Task 4.1).

import { readdirSync } from 'node:fs'
import { resolveRegistryDir } from './paths.js'

export async function list(): Promise<void> {
  const root = resolveRegistryDir()
  let entries: string[] = []
  try {
    entries = readdirSync(root, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name)
      .sort()
  } catch {
    // registry not present (package not built) — print nothing.
  }
  if (entries.length === 0) {
    console.log('(no widgets registered)')
    return
  }
  for (const name of entries) console.log(name)
}
