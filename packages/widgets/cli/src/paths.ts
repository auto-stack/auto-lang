// Path resolution helpers. Plan 331, Phase 4.
//
// When compiled, the CLI ships at packages/widgets/cli/dist/*.js. The committed
// registry lives at packages/widgets/registry/, i.e. ../../registry relative to
// the dist dir. `import.meta.url` lets us locate it regardless of the consumer's
// cwd.

import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

/** Directory of the running compiled CLI module (cli/dist). */
function distDir(): string {
  // import.meta.url is available under ESM (package.json has "type": "module").
  return dirname(fileURLToPath(import.meta.url))
}

/** Absolute path to the committed registry root. */
export function resolveRegistryDir(): string {
  // dist -> cli/dist, so registry is two levels up then into registry/.
  return resolve(distDir(), '..', '..', 'registry')
}

/** Absolute path to a widget inside the committed registry. */
export function resolveRegistryWidget(widget: string): string {
  return resolve(resolveRegistryDir(), widget)
}
