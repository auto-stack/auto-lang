# Auto UI → Vue Generator Contracts

These are the generator's assumptions about .at code. Violating them causes
unexpected behavior in generated Vue code. Plan 361 validators (R001-R007)
enforce most of these at build time.

---

## C1: Component Instance Identity

**Contract**: Within the same template, each use of the same component tag
receives a unique, stable key.

**What this means**:
- ✅ You CAN use the same component in multiple v-if branches (e.g., two
  `AutoDownEditor` instances). The generator assigns them distinct keys
  (`AutoDownEditor-1`, `AutoDownEditor-2`).
- ⚠️ If both instances need **state continuity** (editor content should
  survive the branch switch), use a **single instance + prop toggle** instead.
  See [editor-integration pattern](../patterns/editor-integration.md).

**Violation symptom**: Component state lost after v-if switch, Tiptap editor
blank, child component unmount errors, "view is not available".

**Validated by**: R001 (duplicate-component-key), R007 (autodown-dual-instance)

---

## C2: Store Dependency Propagation

**Contract**: `use store: Name` declarations are extracted and passed to the
generator, which emits `import { useNameStore }` and
`const store = reactive(useNameStore())`.

**What this means**:
- ✅ In a widget, write `use store: NotesStore` then use `store.X` directly.
- ⚠️ store_deps extraction depends on parsing `use store:` statements.
  Alternative syntax (e.g., `use back.store: X`) is NOT recognized.

**Violation symptom**: Generated `.vue` has `store is not defined` runtime error.

**Validated by**: R002 (store-usage-without-import)

---

## C3: Third-Party Component CSS Dependencies

**Contract**: When an npm dependency with CSS side-effects is used, the
generator auto-imports its stylesheet.

**Currently auto-handled**:
- `@autodown/editor` → `import '@autodown/editor/style.css'`

**When adding new CSS-bearing dependencies**: update `generate_main_ts` in
`auto-man/vue.rs` with the corresponding injection logic.

**Validated by**: R003 (autodown-css-missing)

---

## C4: Handler Reference Consistency

**Contract**: Template `onclick: .X` references must have a corresponding
`on { .X -> ... }` block. The generator emits an empty function stub with
`// TODO: handler not defined` for unmatched references.

**What this means**:
- ✅ Declare in `msg Msg`, define in `on`, reference in `view`.
- ⚠️ View-only references without `on` blocks produce empty handler stubs
  (clicks have no effect).

**Validated by**: R004 (undefined-handler)

---

## C5: Dark Mode Detection

**Contract**: The generator detects dark mode by looking for a handler with
the exact name `.ToggleDarkMode` (with leading dot). If found, it injects
`:class="{ dark: ... }"` on the root element.

**What this means**:
- ✅ Define `.ToggleDarkMode -> { ... }` in the `on` block.
- ⚠️ Handler name MUST be exactly `ToggleDarkMode`. Other names
  (`ToggleTheme`, `ToggleDark`) are NOT recognized.

---

## C6: List Rendering and Keys

**Contract**: `for x in .items` loops auto-generate `:key` bindings on list
items using the loop variable.

**What this means**:
- ✅ List item keys are automatic — no manual key needed.
- ⚠️ If list items are **stateful components** (e.g., editors), note
  switching destroys/recreates them. This can trigger third-party library
  unmount issues. See [editor-integration pattern](../patterns/editor-integration.md).

**Validated by**: R006 (v-for-without-key)

---

## C7: Same-Component Key Uniqueness (Plan 360 lesson)

**Contract**: Each use of the same component tag in a single template
gets a unique key via the generator's `widget_key_counter`.

**What this means**:
- ✅ Two `AutoDownEditor` instances in different v-if branches get distinct keys.
- ⚠️ If both instances need state continuity, use single instance + prop toggle.
- ⚠️ Custom keys are NOT supported in .at — keys are fully generator-managed.

**Violation symptom**: v-if switch loses component state, Tiptap blank,
`view is not available`.

**Regression tests**: T1 (note switching), T5 (Edit/Save/Cancel)

---

## C8: CSS Variable Scoping (Plan 360 lesson) ⚠️ CRITICAL

**Contract**: Any subsystem that modifies CSS variables (e.g., `--primary`)
must write to the SAME DOM element where Tailwind/shadcn's CSS rules
(e.g., `.dark { --primary: ... }`) apply.

**Background**: The generator applies `.dark` class to `#app > div` (root
component). But CSS variable inheritance means a child element's variable
definition **overrides** the parent's. If the accent system writes `--primary`
to `<html>` but `.dark` rules are on `#app > div`, the child's `.dark` value
wins → accent is invisible in dark mode.

**What this means**:
- ✅ CSS-variable-modifying code must consider "is the target element the
  same one that the consumer's class is on?"
- ✅ When unsure, write to **all possible target elements** simultaneously
- ⚠️ When switching dark/light, clean up old-mode inline residue

**Violation symptom**: Light-mode accent works, dark-mode accent shows default
color instead.

**Regression tests**: **T12-DARK** (critical regression), C-DARK-1 (contract verification)

**Historical lesson**: During Plan 360 implementation, `applyAccent` only wrote
to `<html>`, causing dark-mode accent failure. The fix writes to both `<html>`
and `.dark` elements, plus cleans up residue.

---

## C9: Accent Color System (Plan 360)

**Contract**: When a store declares `accent_color` state, the generator
auto-injects the full accent system:
- `ACCENT_PALETTES` data (5-color HSL)
- `applyAccent(name, isDark)` function (sets CSS vars + localStorage)
- `SetAccent` handler auto-linked to applyAccent
- `ToggleDarkMode` handler auto-linked to applyAccent (dark lightness +4% compensation)
- Module-level bootstrap (restore from localStorage)
- `accent_names` getter (for palette UI rendering)

**What this means**:
- ✅ User only declares `var accent_color str = "indigo"` + `.SetAccent(name)` handler
- ✅ Dark mode lightness auto-compensated
- ⚠️ Subject to C8 constraint — applyAccent must cover `.dark` element

**Regression tests**: T12-LIGHT, T12-DARK, T12-ROUNDTRIP
