/**
 * Build the Auto Playground frontend and sync the output to:
 *   - crates/auto-playground/frontend/dist   (served by the Rust backend)
 *
 * The website-side copy (website/public/playground) was retired in Plan 582:
 * /playground is now the VitePress Notes Explorer page, and the old SPA path
 * serves only a meta-refresh redirect (website/public/playground/index.html).
 */

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const root = resolve(__dirname, '..');

const frontendDir = join(root, 'crates', 'auto-playground', 'frontend');
const backendDistDir = join(frontendDir, 'dist');

function run(cmd, args, cwd) {
  const isWindows = process.platform === 'win32';
  const display = `> ${cmd} ${args.join(' ')} (cwd: ${relative(root, cwd)})`;
  console.log(display);

  // On Windows, package managers like bun are often shell scripts, so we need
  // shell mode. When shell is enabled, pass the command as a single string to
  // avoid the DEP0190 deprecation warning.
  const result = isWindows
    ? spawnSync(`${cmd} ${args.join(' ')}`, {
        cwd,
        stdio: 'inherit',
        shell: true,
      })
    : spawnSync(cmd, args, {
        cwd,
        stdio: 'inherit',
      });

  if (result.status !== 0) {
    console.error(`Command failed with exit code ${result.status}`);
    process.exit(result.status ?? 1);
  }
}

function detectPackageManager(cwd) {
  if (existsSync(join(cwd, 'bun.lock'))) return 'bun';
  if (existsSync(join(cwd, 'pnpm-lock.yaml'))) return 'pnpm';
  if (existsSync(join(cwd, 'yarn.lock'))) return 'yarn';
  return 'npm';
}

const pm = detectPackageManager(frontendDir);

// 1. Build the frontend
run(pm, ['run', 'build'], frontendDir);

if (!existsSync(backendDistDir)) {
  console.error(`Build output not found: ${backendDistDir}`);
  process.exit(1);
}

console.log(`\nPlayground frontend built to:`);
console.log(`  - ${relative(root, backendDistDir)}`);
