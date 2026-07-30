# Known Pitfalls

Anti-patterns discovered during 015-notes development. Each entry describes
the mistake, why it breaks, and the correct approach.

---

## P1: Dual AutoDownEditor Mode Switching

**Anti-pattern**:
```auto
if .editing == true {
    autodown_editor { content: .edit_body, can_edit: true }
}
if .editing == false {
    autodown_editor { content: .note.body, can_edit: false }
}
```

**Problem**: Two instances switching between branches causes Tiptap's
`onUnmounted` to access a destroyed `editor.view.dom`, producing
"The editor view is not available" and breaking mode switching.

**Fix**: The generator now assigns distinct keys to both instances (C7),
and CodeBlockMenu has `isDestroyed` protection. However, prefer
**single instance + canEdit prop toggle**.

**Related contracts**: C1, C7

---

## P2: Fixed Keys Causing State Loss

**Anti-pattern**: Assigning a fixed key `:key="'MyEditor'"` to a stateful
component, expecting it to survive note switching.

**Problem**: Fixed keys mean the component is NEVER destroyed. When props
change, Vue patches instead of remounting. For components that need
re-initialization on content change (Tiptap), patching is insufficient.

**Correct approach**: Let the generator manage keys automatically. If you
need instance persistence, use a prop-toggle architecture instead.

**Related contracts**: C1, C7

---

## P3: store_deps Lost in Multi-Path Generation

**Anti-pattern**: Assuming `use store: X` always propagates correctly.

**Problem**: Before Plan 361, the generator had three code paths. Some
paths dropped `store_deps`, producing `.vue` files without store imports.

**Fix**: Plan 361 converged all paths to a single `generate_component_from_file`.
When adding new generation paths, MUST pass `store_deps`.

**Related contracts**: C2

---

## P4: Imprecise Dark Mode Handler Name

**Anti-pattern**: Using `.ToggleTheme` or `.ToggleDark` instead of `.ToggleDarkMode`.

**Problem**: The generator matches the EXACT string ".ToggleDarkMode" to
detect dark mode support. Any other name is silently ignored.

**Fix**: Always use `ToggleDarkMode` as the handler name. If customization
is needed, update the generator's recognition logic.

**Related contracts**: C5

---

## P5: AutoDownEditor Without CSS Import

**Anti-pattern**: Declaring `@autodown/editor` in pac.at's `npm_deps` and
assuming CSS loads automatically.

**Problem**: Before Plan 360, the generator did not auto-inject the CSS import.
Without it, the editor's boundary `+` button (opacity:0 baseline) becomes
permanently visible.

**Fix**: Plan 360 auto-injects `import '@autodown/editor/style.css'` in
`generate_main_ts` when `npm_deps` includes `@autodown/editor`. When adding
new CSS-bearing dependencies, update `generate_main_ts`.

**Related contracts**: C3
