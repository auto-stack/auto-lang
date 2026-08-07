# 036 — Codegen Warnings (Plan 012 Batch A)

Six compiler gaps used to produce silently-wrong Vue output. Batch A made
each one either compile correctly or fail loudly through a unified warning
channel (`crates/auto-lang/src/ui_gen/validators.rs`). This example
exercises all six in one app.

| Gap | Was | Now |
| --- | --- | --- |
| 30 | stray `,` between view children → junk empty `<div />` spacers | child skipped, **R008 Warning** printed |
| 20 | dynamic `class: .cls` on a plain element dropped | emits `:class="cls"` (`class: if …` → `:class` ternary) |
| 44 | computed referencing another computed → bare ref (always truthy) | emits `is_expanded.value` |
| 45 | `expose { .Open }` on a parameterized handler → handler not generated, `defineExpose({ Open })` resolved to `window.open` at runtime | handler generated and exposed; **R009** guards residual cases |
| 19 | `.remove`/`.contains` mapped to `.splice`/`.includes` on ANY receiver (store facades included) | type-gated: proven arrays/strings map, facades pass through with an **R010 Info** note |
| 47 | `.x != null` → `!== undefined` (misses `null`) | loose `x.value != null` / `== null` |
| — | `class:` on schema-registered Form elements routed through the shadcn attrs path (`label`, `select`) silently dropped | static `class` kept, dynamic exprs emit `:class`; unrenderable values raise **R011 Warning** |

## Warning channel

- Warnings surface on `VueGenerator.last_validation_warnings` /
  `GeneratedComponent.validation_warnings`, and are printed once per
  (file, rule, widget, message) during `auto build`.
- Severity: `Error` / `Warning` block in strict mode, `Info` never blocks.
- `auto build --strict` escalates any blocking warning to a hard build
  failure.

## Build

```sh
auto build           # succeeds; prints [R008] and [R010] warnings
auto build --strict  # FAILS on the deliberate stray comma (R008)
```

## Verify

- Build output shows `[R008 Warning] App` (stray comma) and
  `[R010 Info] App` (facade `.remove` passthrough) — each printed once.
- `gen/front/vue/src/App.vue`:
  - no `<div />` between the comma-separated children,
  - `<span :class="cls">`,
  - `body_text` computed uses `is_expanded.value`,
  - `has_draft` computed uses `draft_name.value != null`,
  - `Del` handler keeps `items.value.splice(i, 1)`,
  - `FacadeDel` handler calls `recentFiles.remove(i)` unchanged,
  - `<label class="slider-row">` (static class kept on the shadcn path),
  - the second `<label>` carries `:class="cls"` (dynamic class kept),
  - the `<Select>` root keeps `class="control-row"`.
- `gen/front/vue/src/components/EntryOpener.vue` contains
  `function Open(entry` and `defineExpose({ Open })`.
