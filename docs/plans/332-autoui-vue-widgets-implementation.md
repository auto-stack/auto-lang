# @auto-ui/widgets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
> **Design:** [331-autoui-vue-widgets-npm-library-design.md](331-autoui-vue-widgets-npm-library-design.md) — read it first; this plan is its execution.
> **Repo rules (CLAUDE.md):** Develop in a dedicated worktree. Run `cargo build -p auto` after any codegen/CLI change. Run `cargo test` after VM/codegen changes. Plan files use `NNN-name.md`.

**Goal:** Ship `@auto-ui/widgets` v0.1 to npm — 12 AutoUI-generated, self-contained Vue 3 SFC primitives, installed via `npx @auto-ui/widgets add <widget>` (shadcn-style copy).

**Architecture:** Add a third `VueMode::Library` to the existing `VueGenerator` that emits one standalone `.vue` per widget (reka-ui import + Tailwind classes, no `@/components/ui/*` imports). A new `auto ui build --target vue` subcommand walks the `WidgetRegistry` and writes those `.vue` files into `packages/widgets/registry/`. A Node/TS CLI (`npx @auto-ui/widgets add`) copies them into a user's project and auto-installs `reka-ui`. A pre-built `dist/styles.css` gives zero-config users styling.

**Tech Stack:** Rust (codegen + CLI), Vue 3 + reka-ui + Tailwind (runtime), Node/TypeScript (consumer-facing CLI), npm (publish).

---

## Pre-flight: Worktree

Per CLAUDE.md, all plan work happens in an isolated worktree, NOT on `master`.

```bash
git worktree add ../auto-lang-332 plan-332/autoui-vue-widgets
cd ../auto-lang-332
```

All subsequent tasks execute there. Merge back to `master` only after the full plan builds and tests pass.

---

## Phase 1 — Codegen: self-contained `VueMode::Library`

**Context from exploration:**
- `crates/auto-lang/src/ui_gen/vue.rs` — `VueGenerator` (struct ~L768), `VueMode` enum (~L757: `Plain`/`Shadcn`), `map_tag()` (~L2942), `generate_sfc()` (~L1133), `generate_shadcn_imports()` (~L7409). Tests at ~L7760.
- `crates/auto-lang/src/ui_gen/widget/registry.rs` — `WidgetRegistry`; `spec.rs` — `WidgetSpec`/`BackendMapping` (component/import/props/events/extra_components).

The current generator emits ONE SFC per `AuraWidget` (a whole app/page) that imports shared shadcn-vue primitives. The library needs the inverse: ONE SFC per PRIMITIVE. So we add a new entry point that takes a primitive name + its `BackendMapping` and emits a standalone component, driven by a new mode.

### Task 1.1: Add `VueMode::Library` variant

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (VueMode enum ~L757, `is_shadcn`/mode helpers)

**Step 1: Write the failing test** (append near existing mode tests ~L7866)

```rust
#[test]
fn test_library_mode_constructor() {
    let gen = VueGenerator::new_library();
    assert!(gen.is_library());
    assert!(!gen.is_shadcn());
}
```

**Step 2: Run — verify it fails**

`cargo test -p auto-lang -- vue::test_library_mode_constructor`
Expected: FAIL — `new_library` / `is_library` not found.

**Step 3: Implement**

- Add `Library` to `pub enum VueMode { Plain, Shadcn, Library }`.
- Add `pub fn new_library() -> Self { Self::with_mode(VueMode::Library) }`.
- Add `pub fn is_library(&self) -> bool { matches!(self.mode, VueMode::Library) }`.

**Step 4: Run — verify pass**

`cargo test -p auto-lang -- vue::test_library_mode_constructor` → PASS.
Then `cargo build -p auto-lang` → clean.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/vue.rs
git commit -m "feat(ui_gen): add VueMode::Library for self-contained widget output"
```

---

### Task 1.2: Per-widget standalone SFC entry point

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` (add method on `VueGenerator`)
- Modify: `crates/auto-lang/src/ui_gen/mod.rs` / `lib.rs` — re-export if needed

**Goal:** A function `generate_widget_sfc(&mut self, name: &str) -> GenResult<String>` that produces a complete standalone `.vue` for one primitive (e.g. `button`), importing from `reka-ui` (never `@/components/ui/*`).

**Step 1: Write the failing test**

```rust
#[test]
fn test_library_button_sfc_is_self_contained() {
    let mut gen = VueGenerator::new_library();
    let sfc = gen.generate_widget_sfc("button").unwrap();
    assert!(sfc.contains("<template>"), "has template");
    assert!(sfc.contains("<script setup"), "has script setup");
    assert!(!sfc.contains("@/components/ui/"), "must NOT import shadcn-vue");
    assert!(sfc.contains("reka-ui"), "uses reka-ui as backend");
}
```

**Step 2: Run — verify it fails** (`no method generate_widget_sfc`).

**Step 3: Implement (minimal)**

Add a placeholder that returns a hard-coded button SFC string to make the test pass (we generalize in Task 1.4). The button template:

```rust
pub fn generate_widget_sfc(&mut self, name: &str) -> GenResult<String> {
    // Phase 1.2: button only; generalized in 1.4 via registry lookup.
    debug_assert_eq!(name, "button"); // removed in 1.4
    Ok(r#"<script setup lang="ts">
import { computed } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '../utils'
import { buttonVariants } from './variants'
import type { ButtonVariants } from './variants'

const props = withDefaults(defineProps<{
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: string
  as?: string
  asChild?: boolean
}>(), { variant: 'default', size: 'default', as: 'button' })
</script>

<template>
  <Primitive :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)">
    <slot />
  </Primitive>
</template>
"#.to_string())
}
```

**Step 4: Run — verify pass.** `cargo test -p auto-lang -- vue::test_library_button_sfc_is_self_contained` → PASS.

**Step 5: Commit** `feat(ui_gen): generate_widget_sfc entry point (button stub)`

---

### Task 1.3: Companion per-widget files (variants.ts, utils.ts)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — add `generate_widget_support_files(&self, name: &str) -> Vec<(String, String)>` returning `(relative_path, content)` pairs (e.g. `variants.ts`, `index.ts`).

The button SFC above imports `./variants` and `../utils`. The generator must also emit these alongside each widget so the copied component is self-contained.

**Step 1: Failing test** — assert `generate_widget_support_files("button")` returns entries whose paths include `variants.ts` and `index.ts`, and `index.ts` re-exports `Button`.

**Step 2: Verify fail.**

**Step 3: Implement** — emit:
- `index.ts`: `export { default as Button } from './Button.vue'`
- `variants.ts`: the `buttonVariants` cva recipe + `ButtonVariants` type (use `class-variance-authority`). Derive the Tailwind class strings from the same shadcn-vue reference recipe.

**Step 4: Verify pass.**

**Step 5: Commit** `feat(ui_gen): emit per-widget support files (variants/index)`

---

### Task 1.4: Generalize via registry lookup

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs` + `spec.rs`
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — `generate_widget_sfc` now reads the `WidgetSpec` for `name` and dispatches to a per-widget template fn.

**Goal:** Replace the `debug_assert_eq!(name,"button")` stub with a match over a new method `widget_template(&self, name: &str) -> WidgetTemplate` that returns the script/template/support content per widget. For 1.4, implement `button` + `input` + `label` (three) to prove the pattern generalizes; remaining widgets land in Phase 5.

**Step 1: Failing test** — `generate_widget_sfc("input")` and `generate_widget_sfc("label")` each produce self-contained SFCs (no `@/components/ui/`, reka-ui or native input).

**Step 2: Verify fail.**

**Step 3: Implement** — add a `WidgetTemplate` struct `{ script: String, template: String, support_files: Vec<(String,String)> }` and a `fn library_template(name) -> Option<WidgetTemplate>` table. Wire `generate_widget_sfc` to look it up, returning a clear error (`unknown widget: {name}`) if absent.

**Step 4: Verify pass** — all three widget tests green; `cargo test -p auto-lang -- vue`.

**Step 5: Commit** `feat(ui_gen): registry-driven library templates (button/input/label)`

---

## Phase 2 — `auto ui build` subcommand

**Context:** CLI is clap-derived in `crates/auto/src/main.rs` (enum ~L339, match ~L1107). Pattern to follow: `crates/auto/src/cmd_vue.rs`.

### Task 2.1: Add `Ui` command to clap enum

**Files:**
- Modify: `crates/auto/src/main.rs` (~L339 add variant; ~L1107 add match arm)

**Step 1:** Add to `Commands` enum:

```rust
#[command(about = "AutoUI widget library commands")]
Ui {
    #[command(subcommand)]
    action: UiAction,
},
```

and a `UiAction` enum: `Build { target: String, out: String, widgets: Vec<String> }` (target defaults `vue`, out defaults `packages/widgets/registry`).

**Step 2:** Add match arm that calls `cmd_ui::build(action)`.

**Step 3:** `cargo build -p auto` → clean. `auto ui --help` prints the subcommand.

**Step 4: Commit** `feat(cli): add 'auto ui' subcommand scaffold`

---

### Task 2.2: Create `crates/auto/src/cmd_ui.rs` build action

**Files:**
- Create: `crates/auto/src/cmd_ui.rs`
- Modify: `crates/auto/src/main.rs` — `mod cmd_ui;`

**Goal:** `build(action)` instantiates `VueGenerator::new_library()`, iterates the requested widgets (or all registered), calls `generate_widget_sfc` + `generate_widget_support_files`, writes `<out>/<widget>/Widget.vue` (+ support files).

**Step 1: Manual integration test** (no harness yet) — create `tmp/ui_build_test/` and run:

```bash
cargo build -p auto
./target/debug/auto ui build --target vue --out tmp/ui_build_test --widgets button,input,label
ls tmp/ui_build_test   # expect: button/ input/ label/
cat tmp/ui_build_test/button/Button.vue   # expect self-contained SFC
```

**Step 2: Implement `cmd_ui::build`** — for each widget: `fs::create_dir_all`, write SFC + support files. Print a summary (`wrote N widgets to <out>`). Errors propagate with context.

**Step 3:** Re-run Step 1 commands → all pass; verify no `@/components/ui/` in output.

**Step 4:** Add a Rust integration test `crates/auto/tests/cmd_ui.rs` that spawns the binary (`std::process::Command`) with `--out <tempdir>` and asserts the expected files exist and are self-contained. Use `assert_cmd` if already a dev-dep; else use `std::process`.

**Step 5: Commit** `feat(cli): 'auto ui build' writes self-contained .vue widgets`

---

## Phase 3 — Package scaffold (`packages/widgets/`)

### Task 3.1: Directory + package.json

**Files:**
- Create: `packages/widgets/package.json`, `packages/widgets/LICENSE`, `packages/widgets/NOTICES`, `packages/widgets/README.md`, `packages/widgets/.gitignore`, `packages/widgets/cli/tsconfig.json`

**Step 1:** `packages/widgets/package.json`:

```jsonc
{
  "name": "@auto-ui/widgets",
  "version": "0.1.0",
  "description": "AutoUI-generated Vue 3 component primitives (reka-ui + Tailwind).",
  "type": "module",
  "license": "MIT",
  "bin": { "auto-ui": "./cli/dist/index.js" },
  "files": ["registry", "dist", "cli/dist", "README.md", "LICENSE", "NOTICES"],
  "exports": {
    "./styles.css": "./dist/styles.css",
    "./registry/*": "./registry/*"
  },
  "peerDependencies": { "vue": "^3.4.0", "reka-ui": "^2.0.0" },
  "peerDependenciesMeta": { "reka-ui": { "optional": true } },
  "devDependencies": {
    "tailwindcss": "^3.4.0",
    "typescript": "^5.3.0"
  }
}
```

**Step 2:** `LICENSE` = copy repo root MIT (Soutek Co. Ltd.).

**Step 3:** `NOTICES` — list reka-ui, shadcn-vue, Tailwind CSS, each with copyright + "MIT". Add note: "Visual layer of generated components is derived from shadcn-vue (MIT)."

**Step 4:** `README.md` — install/usage (add command), the Tailwind two-path note (import styles.css OR bring your own), Credits.

**Step 5:** `.gitignore` — `node_modules/`, `cli/dist/`. (Note: `registry/` and `dist/styles.css` ARE committed — they're the published artifacts, generated but tracked so publishes are reproducible without a build on the consumer side.)

**Step 6:** Commit `feat(packages): scaffold @auto-ui/widgets package (license/notices/readme)`

---

### Task 3.2: Attribution header on generated `.vue`

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs` — `generate_widget_sfc` prepends a header comment to every emitted SFC.

**Step 1: Failing test** — assert SFC starts with `<!-- Generated by AutoUI` and contains `shadcn-vue (MIT)`.

**Step 2–4:** Implement, verify, commit `feat(ui_gen): attribution header on generated widgets`.

---

## Phase 4 — Node/TS CLI (`add` / `list`)

**Goal:** `npx @auto-ui/widgets add <widget>` copies `registry/<widget>/*` into the user's `src/components/ui/<widget>/`, auto-installs `reka-ui`, prompts for Tailwind path. `list` prints available widgets.

### Task 4.1: CLI skeleton + `list`

**Files:**
- Create: `packages/widgets/cli/src/index.ts`, `packages/widgets/cli/src/list.ts`, `packages/widgets/cli/package-build.json` (scripts)

**Step 1:** `list.ts` — reads `registry/` dir names, prints them.

**Step 2:** Minimal `index.ts` argv parser (no dep; or add `commander` as devDep) dispatching `list`/`add`.

**Step 3:** Build: `tsc` → `cli/dist/index.js`. Verify `node cli/dist/index.js list` prints widget names (after Phase 5 populates registry; for now prints empty/placeholder).

**Step 4: Commit** `feat(cli): @auto-ui/widgets list command`.

---

### Task 4.2: `add` — copy component files

**Files:**
- Create: `packages/widgets/cli/src/add.ts`

**Step 1:** `add <widget>`:
1. Resolve package `registry/<widget>/` (via `import.meta.url`).
2. Resolve user dest: `<cwd>/src/components/ui/<widget>/` (allow `--out`).
3. Copy all files. Error if widget unknown (suggest `list`).

**Step 2:** Manual test in `tmp/`: `mkdir tmp/consumer && cd tmp/consumer && node ../../cli/dist/index.js add button` → `src/components/ui/button/Button.vue` exists.

**Step 3: Commit** `feat(cli): add command copies widget into consumer project`.

---

### Task 4.3: `add` — auto-install reka-ui

**Files:**
- Modify: `packages/widgets/cli/src/add.ts`

**Step 1:** Detect pkg manager (reuse logic mirroring `auto-man::pkg`: prefer pnpm>bun>npm, or read `packageManager` field). If `reka-ui` absent from consumer deps, run install (`pnpm add reka-ui`, etc.). Respect `--no-install`. Support `--reka-ui <pkg>` → rewrite the `from 'reka-ui'` import path in copied files.

**Step 2:** Manual test — fresh `tmp/consumer2`, run `add button`, confirm `reka-ui` added to its package.json. Test `--no-install` skips. Test `--reka-ui @my/fork` rewrites import.

**Step 3: Commit** `feat(cli): add auto-installs reka-ui (with --no-install / --reka-ui overrides)`.

---

### Task 4.4: `add` — Tailwind prompt + `--no-styles`

**Files:**
- Modify: `packages/widgets/cli/src/add.ts`

**Step 1:** Detect `tailwind.config.*` in consumer:
- Present → print reminder to include the copied `.vue` path in `content` (or attempt to patch it; minimal v1 = just print the reminder).
- Absent → print guidance: `import '@auto-ui/widgets/styles.css'` (the zero-config path).
- `--no-styles` → skip the styles guidance entirely.

**Step 2:** Manual test both branches.

**Step 3: Commit** `feat(cli): add tailwind guidance + --no-styles flag`.

---

## Phase 5 — Remaining 9 widgets

Task 1.4 shipped button/input/label. Remaining v1 set: `textarea`, `checkbox`, `switch`, `card`, `separator`, `badge`, `dialog`, `tabs`. (12 total: those 3 + these 8 + `input` already done = wait, recount: v1 list = button, input, textarea, checkbox, switch, label, card, separator, badge, dialog, tabs = 11. Adjust to 12 by adding `avatar` or keep 11 — confirm count with design; default to the 11 named in design §6 + add `avatar` for 12.)

For EACH widget, one task following the **Widget Template Pattern**:

### Widget Template Pattern (repeat per widget)

**Files:** Modify `crates/auto-lang/src/ui_gen/vue.rs` `library_template` table.

**Step 1: Failing test**

```rust
#[test]
fn test_library_<widget>_sfc() {
    let mut gen = VueGenerator::new_library();
    let sfc = gen.generate_widget_sfc("<widget>").unwrap();
    assert!(!sfc.contains("@/components/ui/"));
    assert!(sfc.contains("<template>") && sfc.contains("<script setup"));
    // widget-specific: e.g. dialog uses reka-ui DialogRoot/DialogTrigger
}
```

**Step 2:** Verify fail (`unknown widget: <widget>`).

**Step 3:** Add `library_template` entry — script (reka-ui imports + props), template (Tailwind classes per shadcn-vue reference recipe), support files (variants/index). Attribution header auto-prepended.

**Step 4:** Verify pass + `cargo test -p auto-lang -- vue::test_library`.

**Step 5:** Regenerate + visual check: `auto ui build --target vue --out tmp/ui_build_test --widgets <widget>`; open the SFC, eyeball correctness.

**Step 6:** Commit `feat(ui_gen): <widget> library template`.

**Widgets (one commit each):** `textarea`, `checkbox`, `switch`, `card`, `separator`, `badge`, `dialog`, `tabs`, `avatar`.

After all widgets: commit regenerated `packages/widgets/registry/` content.

---

## Phase 6 — Pre-built `dist/styles.css`

**Goal:** Zero-config users `import '@auto-ui/widgets/styles.css'` and get all widget styling.

### Task 6.1: Tailwind build over registry

**Files:**
- Create: `packages/widgets/tailwind.config.cjs`, `packages/widgets/build-styles.js` (or `.cjs`)

**Step 1:** `tailwind.config.cjs` — `content: ['./registry/**/*.vue', './registry/**/*.ts']`, default theme (shadcn slate base color CSS variables), `plugins: [require('tailwindcss-animate')]`.

**Step 2:** `build-styles.js` — shells `npx tailwindcss -i ./src/input.css -o ./dist/styles.css --minify` (input.css = `@tailwind` directives + the shadcn CSS variable `:root`/`.dark` block).

**Step 3:** Run `node build-styles.js` → `dist/styles.css` exists, non-empty, contains button classes.

**Step 4:** Wire into package `scripts.build`: `"build:css": "node build-styles.js"`.

**Step 5:** Commit the generated `dist/styles.css` + config.

---

## Phase 7 — Dogfood: rename + consume in gallery

### Task 7.1: Rename `examples/component-gallery` → `examples/gallery`

**Files:** `git mv examples/component-gallery examples/gallery`; global search-replace the path in docs/scripts (`grep -r component-gallery`).

**Step 1:** `git mv`, then `grep -rn "component-gallery" --include=*.md --include=*.rs --include=*.json --include=*.ts .` and update references.

**Step 2:** Verify gallery still runs (`cd examples/gallery/vue && pnpm install && pnpm dev`) — build green.

**Step 3:** Commit `chore: rename component-gallery -> gallery`.

---

### Task 7.2: Gallery consumes `@auto-ui/widgets`

**Files:** Modify `examples/gallery/vue/package.json` (add `"@auto-ui/widgets": "file:../../packages/widgets"`), then replace gallery's hand-maintained `src/components/ui/<widget>` for the v1 set with copies from the package via `npx`.

**Step 1:** Add local file dep. `pnpm install`.

**Step 2:** For each v1 widget, `pnpm exec auto-ui add <widget>` (or `node ../../packages/widgets/cli/dist/index.js add <widget>`), overwriting the gallery's shadcn-vue version. Confirm pages still render.

**Step 3:** Run gallery dev server, visually verify each v1 widget page (button/input/.../dialog/tabs) looks correct.

**Step 4:** Commit `feat(gallery): consume @auto-ui/widgets (dogfood)`. This is the end-to-end validation that the package works in a real Vue project.

---

## Phase 8 — Publish readiness

### Task 8.1: Dry-run publish

**Step 1:** From `packages/widgets/`: `npm pack` → inspect the tarball contents (`tar tzf *.tgz`). Confirm ONLY `registry/`, `dist/`, `cli/dist/`, README, LICENSE, NOTICES are included — no source `cli/src/`, no `node_modules`.

**Step 2:** `npm publish --dry-run` → confirm no errors, peer deps correct, `bin` resolves.

**Step 3:** Document the publish procedure in `packages/widgets/README.md` (section "Maintainer release"): bump version → `pnpm build:css` → regenerate registry (`auto ui build`) → `npm publish`.

**Step 4:** Commit `docs(packages): publish procedure + verified npm pack contents`.

(DO NOT run a real `npm publish` without explicit user approval — it's an irreversible outward action.)

---

## Definition of Done (v0.1)

- [ ] `auto ui build --target vue` generates all v1 widgets as self-contained SFCs (no `@/components/ui/` imports).
- [ ] `packages/widgets/registry/` populated; `dist/styles.css` builds.
- [ ] `npx @auto-ui/widgets list` / `add <widget>` work (copy + reka-ui auto-install + tailwind guidance).
- [ ] `examples/gallery` consumes the package and renders all v1 widgets correctly.
- [ ] `npm pack` / `npm publish --dry-run` clean; `files` allowlist tight; LICENSE + NOTICES present; attribution headers on every `.vue`.
- [ ] All `cargo test -p auto-lang -- vue` green; `cargo build -p auto` green.
- [ ] Worktree branch merged to `master` after green build + tests.

---

## Risks (carried from design §11)

- **Codegen refactor scope** → mitigated by Phase 1's incremental TDD (stub → generalize → registry-driven) and shipping 3 widgets before the rest.
- **Tailwind pre-built vs user Tailwind conflict** → CLI two-path prompt (Phase 4.4).
- **reka-ui version drift** → peer range + CLI installs matching version.
- **Active-session git safety** → all work in the plan-332 worktree; never touch `master` directly.
