/// <reference types="vite/client" />
//
// Blueprint catalog, derived from disk via Vite `import.meta.glob` (PLAN-640):
// adding or removing a package under blueprints/ requires zero edits here —
// no generated artifacts, no manifest to sync. (This replaces Plan 343's
// `auto bp export-gallery-meta` outlook: that would have introduced a
// generated-artifact sync surface; `auto bp list` remains the authoritative
// registry view, this module is only the gallery's read of the same disk.)

const specModules = import.meta.glob('../../../blueprints/*/*/spec.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>

const gotchaModules = import.meta.glob('../../../blueprints/*/*/gotchas.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>

const referenceModules = import.meta.glob(
  '../../../blueprints/*/*/reference/*.at',
  { query: '?raw', import: 'default', eager: true },
) as Record<string, string>

export interface BpEntry {
  kind: string
  name: string
  spec: string
  gotchas: string
  references: Record<string, string>
}

/// `../../../blueprints/<kind>/<name>/<file>` -> `{ kind, name, file }`.
function parseKey(path: string): { kind: string; name: string; file: string } {
  const rest = path.slice(path.indexOf('blueprints/') + 'blueprints/'.length)
  const parts = rest.split('/')
  return { kind: parts[0], name: parts[1], file: parts.slice(2).join('/') }
}

const entries = new Map<string, BpEntry>()

for (const [path, spec] of Object.entries(specModules)) {
  const { kind, name } = parseKey(path)
  entries.set(`${kind}/${name}`, { kind, name, spec, gotchas: '', references: {} })
}

for (const [path, raw] of Object.entries(gotchaModules)) {
  const { kind, name } = parseKey(path)
  const entry = entries.get(`${kind}/${name}`)
  if (entry) entry.gotchas = raw
}

for (const [path, raw] of Object.entries(referenceModules)) {
  const { kind, name, file } = parseKey(path)
  const entry = entries.get(`${kind}/${name}`)
  const variant = file.replace(/^reference\//, '').replace(/\.at$/, '')
  if (entry && variant) entry.references[variant] = raw
}

export const bps: BpEntry[] = [...entries.values()].sort(
  (a, b) => a.kind.localeCompare(b.kind) || a.name.localeCompare(b.name),
)

/// Preferred kind ordering (governed by docs/specs/blueprint/contract.md Q5);
/// discovered kinds not listed here append alphabetically.
const kindPreference = [
  'form',
  'navigation',
  'dashboard',
  'data-display',
  'feedback',
  'editor',
  'layout',
  'composite',
]
const discovered = [...new Set(bps.map((b) => b.kind))]
const tail = discovered.filter((k) => !kindPreference.includes(k)).sort()
export const kindOrder = [
  ...kindPreference.filter((k) => discovered.includes(k)),
  ...tail,
]
