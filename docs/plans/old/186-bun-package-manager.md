# Plan 186: Switch from npm to bun for Vue/Web Projects

## Context

Every newly generated Vue project runs `npm install`, taking 10+ seconds. Bun has a global cache with hard-linking — after the first install, projects sharing the same deps (shadcn-vue, tailwind, vue, etc.) link instantly. Bun is fully compatible with our Vue + Vite + shadcn-vue + Tailwind stack.

## Approach

Create a shared `pkg` module in `auto-man` that auto-detects bun (falls back to npm). Replace all 21 hardcoded `npm`/`npx` references across 6 files.

## Phase 1: Create `crates/auto-man/src/pkg.rs`

New module with:
- `detect()` → returns `Bun` or `Npm` (cached via `OnceLock`)
- `command_exists(cmd)` → checks PATH (`where` on Windows, `which` on Unix)
- `run_command_live(cmd, args, cwd)` → shared helper with Windows `cmd /C` wrapper
- `install(cwd)` → `bun install` or `npm install`
- `run_script(script, args, cwd)` → `bun run dev` or `npm run dev`
- `exec(package, args, cwd)` → `bunx <pkg>` or `npx --yes <pkg>`
- `add_packages(packages, dev, cwd)` → `bun add --dev` or `npm install --save-dev`
- `install_cmd()` / `exec_cmd()` / `display_name()` → string accessors

Register in `crates/auto-man/src/lib.rs` as `pub mod pkg`.

## Phase 2: Update `crates/auto-man/src/vue.rs` (primary)

- Delete local `command_exists()` and `run_command_live()`
- `npm_install()` → `crate::pkg::install()`
- `install_shadcn_components()` → `crate::pkg::exec("shadcn-vue@latest", ...)`
- `npm_build()` → `crate::pkg::run_script("build", ...)`
- `npm_run_dev()` → `crate::pkg::run_script("dev", ...)`
- Update user-facing messages to use `pkg::display_name()`

## Phase 3: Update `crates/auto-man/src/tauri.rs`

- Delete local `run_command_live()`
- `init_tauri()` npm install calls → `crate::pkg::add_packages()`
- `npx tauri init` → `crate::pkg::exec("tauri", ...)`
- `npx tauri dev` → `crate::pkg::exec("tauri", &["dev"], ...)`

## Phase 4: Update `crates/auto-man/src/vscode.rs`

- Replace 4 inline `Command::new("cmd")` blocks with `crate::pkg::install()` / `crate::pkg::run_script()`
- Keep VSCode extension's generated `package.json` scripts as `npm run` (they run in the user's VSCode, not ours)

## Phase 5: Update `crates/auto-man/src/builder/vue.rs`

- `npm run build` → `crate::pkg::run_script("build", ...)`
- `npm run dev` → `crate::pkg::run_script("dev", ...)`

## Phase 6: Update legacy `crates/auto/src/cmd_vue.rs` and `cmd_tauri.rs`

- Duplicate the small `pkg` detection logic (legacy crate can't depend on auto-man)
- Or add `auto-man` as dependency if feasible
- Update `tauri.conf.json` generation: `beforeDevCommand` / `beforeBuildCommand` use `pkg::display_name()`

## Key Decisions

- **No generated script changes**: `"dev": "vite"` works identically with `bun run` and `npm run`
- **`npx --yes` → `bunx`**: bunx auto-confirms, no `--yes` needed (but harmless if present)
- **VSCode extension package.json**: keep `npm run` scripts (user's environment)
- **Fallback**: if bun not found, silently use npm — no config required

## Verification

1. `cargo build` — compiles clean
2. `cargo test -p auto-man` — all tests pass (update any that assert "npm" in output)
3. `auto run` on a Vue project — verify "bun install" in output, dev server starts
4. `auto build` on a Vue project — verify build completes
5. Check that removing bun from PATH falls back to npm gracefully
