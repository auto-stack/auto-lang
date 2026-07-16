/**
 * Build the Auto Playground frontend and sync the output to both:
 *   - crates/auto-playground/frontend/dist   (served by the Rust backend)
 *   - website/public/playground              (deployed as a standalone page)
 *
 * This ensures the backend-served playground and the website playground are
 * always built from the same source and stay in sync.
 */

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, statSync, mkdirSync, copyFileSync, rmSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const root = resolve(__dirname, '..');

const frontendDir = join(root, 'crates', 'auto-playground', 'frontend');
const backendDistDir = join(frontendDir, 'dist');
const websitePlaygroundDir = join(root, 'website', 'public', 'playground');

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

function cleanDir(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    rmSync(full, { recursive: true, force: true });
  }
}

function copyDir(src, dst) {
  mkdirSync(dst, { recursive: true });
  for (const entry of readdirSync(src)) {
    const srcPath = join(src, entry);
    const dstPath = join(dst, entry);
    const stat = statSync(srcPath);
    if (stat.isDirectory()) {
      copyDir(srcPath, dstPath);
    } else {
      copyFileSync(srcPath, dstPath);
    }
  }
}

const pm = detectPackageManager(frontendDir);

// 1. Build the frontend
run(pm, ['run', 'build'], frontendDir);

if (!existsSync(backendDistDir)) {
  console.error(`Build output not found: ${backendDistDir}`);
  process.exit(1);
}

// 2. Sync to website/public/playground
cleanDir(websitePlaygroundDir);
copyDir(backendDistDir, websitePlaygroundDir);

console.log(`\nSynced playground build to:`);
console.log(`  - ${relative(root, backendDistDir)}`);
console.log(`  - ${relative(root, websitePlaygroundDir)}`);
