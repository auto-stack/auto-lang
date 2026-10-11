import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// --check resolves the same inputs used by the probe, without opening a window.
export function reproContext(defaultPort) {
  const corpus = process.env.P446_CORPUS_ROOT
    ? resolve(process.env.P446_CORPUS_ROOT)
    : fileURLToPath(new URL('../../../test/fixtures/plan446/b3check/auto/', import.meta.url));
  if (!existsSync(resolve(corpus, 'src/front/app.at'))) {
    throw new Error(`Plan 446 corpus not found: ${corpus}`);
  }
  const binary = process.env.AUTO_BIN || 'auto';
  const port = process.argv[2] && process.argv[2] !== '--check' ? process.argv[2] : defaultPort;
  if (process.argv.includes('--check')) {
    console.log(JSON.stringify({ corpus, binary, port }));
    process.exit(0);
  }
  return { corpus, binary, port };
}
